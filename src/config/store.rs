use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::{Duration, SystemTime};

use super::parser::parse_config_content;
use super::settings::{SettingsMap, build_settings};
use super::value::ConfigValue;
use super::writer::serialize;
use crate::error::{Result, WrCliError};

/// Config store inspired by Go's Viper.
///
/// Reads four sources in ascending priority order:
/// 1. Programmatic defaults ([`Config::set_default`])
/// 2. Config file ([`Config::read_in_config`])
/// 3. Environment variables ([`Config::automatic_env`], [`Config::bind_env`])
/// 4. CLI flag overrides (injected automatically after flag parsing)
///
/// Supports dot-notation keys for nested access: `"database.host"`.
///
/// # Supported formats
///
/// | Format | Feature flag    |
/// |------|-----------------|
/// | TOML | `toml-config`   |
/// | JSON | `json-config`   |
/// | YAML | `yaml-config`   |
///
/// # Example
/// ```no_run
/// use wrcli::Config;
///
/// let mut cfg = Config::new()
///     .set_config_name("myapp")
///     .set_config_type("toml")
///     .add_config_path(".")
///     .set_default("server.port", 8080i64)
///     .automatic_env()
///     .set_env_prefix("MYAPP");
///
/// cfg.read_in_config().ok();
/// let port = cfg.get_int("server.port").unwrap_or(8080);
/// ```
#[derive(Debug, Clone)]
pub struct Config {
    // Priority layer 1 (lowest): programmatic defaults
    defaults: HashMap<String, ConfigValue>,
    // Priority layer 2: config file values
    file_values: HashMap<String, ConfigValue>,
    // Priority layer 3: CLI flag overrides
    flag_values: HashMap<String, ConfigValue>,
    // Priority layer 4 (highest): explicit values set via `set()`
    explicit_values: HashMap<String, ConfigValue>,
    // alias -> canonical key
    aliases: HashMap<String, String>,

    config_name: Option<String>,
    config_type: Option<String>,
    config_paths: Vec<PathBuf>,
    config_file: Option<PathBuf>,
    loaded_file: Option<PathBuf>,

    env_prefix: Option<String>,
    env_prefix_upper: Option<String>,
    auto_env: bool,
    allow_empty_env: bool,
    env_key_replacer: Option<Vec<(String, String)>>,
    explicit_env_bindings: HashMap<String, String>,

    key_delim: char,

    on_change: Option<ChangeCallback>,
    watch_interval: Duration,
}

/// Config change callback (shared with the watcher thread).
#[derive(Clone)]
struct ChangeCallback(Arc<dyn Fn(&Config) + Send + Sync>);

impl std::fmt::Debug for ChangeCallback {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ChangeCallback")
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            defaults: HashMap::new(),
            file_values: HashMap::new(),
            flag_values: HashMap::new(),
            explicit_values: HashMap::new(),
            aliases: HashMap::new(),
            config_name: None,
            config_type: None,
            config_paths: Vec::new(),
            config_file: None,
            loaded_file: None,
            env_prefix: None,
            env_prefix_upper: None,
            auto_env: false,
            allow_empty_env: true,
            env_key_replacer: None,
            explicit_env_bindings: HashMap::new(),
            key_delim: '.',
            on_change: None,
            watch_interval: Duration::from_secs(1),
        }
    }
}

/// The layer where [`Config::resolve`] found the value.
enum Layer<'a> {
    Explicit(&'a ConfigValue),
    Flag(&'a ConfigValue),
    Env(String),
    File(&'a ConfigValue),
    Default(&'a ConfigValue),
}

impl Config {
    pub fn new() -> Self {
        Default::default()
    }

    // ── File settings ─────────────────────────────────────────────────────────

    /// Base config file name without extension (e.g. `"config"`, `"myapp"`).
    pub fn set_config_name(mut self, name: &str) -> Self {
        self.config_name = Some(name.to_owned());
        self
    }

    /// Config file format: `"toml"`, `"json"`, `"yaml"` / `"yml"`.
    pub fn set_config_type(mut self, t: &str) -> Self {
        self.config_type = Some(t.to_owned());
        self
    }

    /// Adds a directory to search for config files. Supports `~` and `$VAR` expansion.
    pub fn add_config_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.config_paths.push(path.into());
        self
    }

    /// Explicitly sets a single config file path. Supports `~` and `$VAR` expansion.
    ///
    /// Loads directly from this path regardless of config name/type/search paths.
    /// The format is inferred automatically from the extension (`.toml`/`.json`/`.yaml`/`.yml`).
    pub fn set_config_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.config_file = Some(path.into());
        self
    }

    /// Loads the config file from the first matching path.
    ///
    /// Returns [`WrCliError::ConfigFileNotFound`] if no file is found.
    /// Use `.read_in_config().ok()` to ignore a missing file.
    pub fn read_in_config(&mut self) -> Result<()> {
        if let Some(file) = self.config_file.clone() {
            return self.read_explicit_file(file);
        }
        let name = self
            .config_name
            .as_deref()
            .ok_or_else(|| WrCliError::ConfigFileNotFound {
                name: "<not set>".to_owned(),
                paths: vec![],
            })?;

        // If no format was specified, try every supported extension candidate in order.
        let extensions: Vec<String> = match self.config_type.as_deref() {
            Some(t) => vec![t.to_owned()],
            None => supported_extensions(),
        };
        let paths = self.search_paths();

        for ext in &extensions {
            let filename = format!("{}.{}", name, ext);
            for path in &paths {
                let expanded = expand_path(path);
                let full = expanded.join(&filename);
                log::debug!("searching for config file: {}", full.display());
                if full.exists() {
                    log::debug!("found config file: {}", full.display());
                    let content = std::fs::read_to_string(&full)?;
                    self.file_values =
                        parse_config_content(&content, ext, &full.display().to_string())?;
                    self.loaded_file = Some(full);
                    return Ok(());
                }
            }
        }

        Err(WrCliError::ConfigFileNotFound {
            name: format!("{}.{}", name, extensions.join("|")),
            paths: paths.iter().map(|p| p.display().to_string()).collect(),
        })
    }

    fn read_explicit_file(&mut self, file: PathBuf) -> Result<()> {
        let expanded = expand_path(&file);
        let ext = expanded
            .extension()
            .and_then(|e| e.to_str())
            .map(str::to_lowercase)
            .unwrap_or_default();
        let content = std::fs::read_to_string(&expanded)?;
        self.file_values = parse_config_content(&content, &ext, &expanded.display().to_string())?;
        self.loaded_file = Some(expanded);
        Ok(())
    }

    /// List of user-added search paths plus the standard locations appended.
    ///
    /// Viper style: user paths -> `$XDG_CONFIG_HOME/<name>` or `~/.config/<name>`
    /// -> `~/.<name>` -> current directory.
    fn search_paths(&self) -> Vec<PathBuf> {
        let mut paths = self.config_paths.clone();
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("USERPROFILE").map(PathBuf::from));
        if let Some(name) = &self.config_name
            && let Some(home) = &home
        {
            let xdg = std::env::var_os("XDG_CONFIG_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| home.join(".config"));
            paths.push(xdg.join(name));
            paths.push(home.join(format!(".{}", name)));
        }
        paths.push(PathBuf::from("."));
        paths
    }

    // ── Defaults ─────────────────────────────────────────────────────────────

    /// Sets a programmatic default (lowest priority).
    pub fn set_default(mut self, key: &str, val: impl Into<ConfigValue>) -> Self {
        let key = self.canonical_key(key);
        self.defaults.insert(key, val.into());
        self
    }

    /// Sets an explicit value (highest priority, Viper's `Set`).
    pub fn set(mut self, key: &str, val: impl Into<ConfigValue>) -> Self {
        let key = self.canonical_key(key);
        self.explicit_values.insert(key, val.into());
        self
    }

    /// Registers `key`'s value to be returned when looked up via `alias` (Viper's `RegisterAlias`).
    pub fn register_alias(mut self, alias: &str, key: &str) -> Self {
        let alias = self.canonical_key(alias);
        let key = self.canonical_key(key);
        self.aliases.insert(alias, key);
        self
    }

    /// Changes the nested key delimiter (default `'.'`, Viper's `SetKeyDelimiter`).
    pub fn set_key_delimiter(mut self, delim: char) -> Self {
        self.key_delim = delim;
        self
    }

    /// Sets the replacement pairs applied when generating env variable names (Viper's `SetEnvKeyReplacer`).
    ///
    /// Applied in order to the uppercased key. Defaults to `.`/`-` -> `_`.
    pub fn set_env_key_replacer(mut self, pairs: &[(&str, &str)]) -> Self {
        self.env_key_replacer = Some(
            pairs
                .iter()
                .map(|(from, to)| ((*from).to_owned(), (*to).to_owned()))
                .collect(),
        );
        self
    }

    /// Sets whether empty env variables count as values (Viper's `AllowEmptyEnv`).
    ///
    /// Defaults to `true` (empty values are used). Viper's default behavior (empty = unset) is `false`.
    pub fn allow_empty_env(mut self, allow: bool) -> Self {
        self.allow_empty_env = allow;
        self
    }

    // ── Environment variables ────────────────────────────────────────────────

    /// Automatically looks up an env variable for every key queried.
    ///
    /// The env variable name is derived from the key: uppercase + `.` -> `_`.
    /// With a prefix set: `PREFIX_KEY_SUBKEY`.
    pub fn automatic_env(mut self) -> Self {
        self.auto_env = true;
        self
    }

    /// Adds a prefix to automatic env variable lookup (e.g. `"MYAPP"`).
    pub fn set_env_prefix(mut self, prefix: &str) -> Self {
        self.env_prefix_upper = Some(prefix.to_uppercase());
        self.env_prefix = Some(prefix.to_owned());
        self
    }

    /// Explicitly binds a config key to a specific env variable.
    pub fn bind_env(mut self, key: &str, env_var: &str) -> Self {
        let key = self.canonical_key(key);
        self.explicit_env_bindings.insert(key, env_var.to_owned());
        self
    }

    // ── Internal: flag binding ────────────────────────────────────────────────

    /// Injects a CLI flag value into the highest-priority layer.
    ///
    /// Called automatically by the command execution engine for flags the user actually passed.
    pub(crate) fn bind_flag_value(&mut self, key: &str, val: ConfigValue) {
        self.flag_values.insert(key.to_owned(), val);
    }

    // ── Getter ───────────────────────────────────────────────────────────────

    /// Internal (dot-notation) key reflecting aliases and the custom delimiter.
    fn canonical_key(&self, key: &str) -> String {
        let mut key = if self.key_delim == '.' {
            key.to_owned()
        } else {
            key.replace(self.key_delim, ".")
        };
        let mut hops = 0;
        while let Some(target) = self.aliases.get(&key) {
            key = target.clone();
            hops += 1;
            if hops >= 32 {
                log::warn!("alias cycle detected near '{}'", key);
                break;
            }
        }
        key
    }

    /// Finds `key` in the priority layers and returns it. Only env is looked up dynamically, so it holds an owned string.
    fn resolve(&self, key: &str) -> Option<Layer<'_>> {
        let key = self.canonical_key(key);
        if let Some(v) = self.explicit_values.get(&key) {
            return Some(Layer::Explicit(v));
        }
        if let Some(v) = self.flag_values.get(&key) {
            return Some(Layer::Flag(v));
        }
        if let Some(v) = self.env_lookup(&key) {
            return Some(Layer::Env(v));
        }
        if let Some(v) = self.file_values.get(&key) {
            return Some(Layer::File(v));
        }
        self.defaults.get(&key).map(Layer::Default)
    }

    /// `true` if any layer has a value (including defaults, same as Viper).
    pub fn is_set(&self, key: &str) -> bool {
        self.resolve(key).is_some()
    }

    /// Raw [`ConfigValue`] lookup. Priority: Set > CLI flags > environment > config file > defaults.
    ///
    /// Environment variables are always strings, so they are returned wrapped in [`ConfigValue::String`].
    pub fn get(&self, key: &str) -> Option<ConfigValue> {
        match self.resolve(key)? {
            Layer::Env(v) => Some(ConfigValue::String(v)),
            Layer::Explicit(v) | Layer::Flag(v) | Layer::File(v) | Layer::Default(v) => {
                Some(v.clone())
            }
        }
    }

    /// Looks up the [`ConfigValue`] reference from the stored layers (Set/flags/file/defaults).
    /// The environment layer is a dynamic lookup, so it is not included.
    pub fn get_ref(&self, key: &str) -> Option<&ConfigValue> {
        let key = self.canonical_key(key);
        self.explicit_values
            .get(&key)
            .or_else(|| self.flag_values.get(&key))
            .or_else(|| self.file_values.get(&key))
            .or_else(|| self.defaults.get(&key))
    }

    /// Looks up the value as `String` (number/bool values are coerced to strings).
    pub fn get_string(&self, key: &str) -> Option<String> {
        match self.resolve(key)? {
            Layer::Env(v) => Some(v),
            Layer::Explicit(v) | Layer::Flag(v) | Layer::File(v) | Layer::Default(v) => {
                v.to_string_coerce()
            }
        }
    }

    /// Looks up the value as `i64` (parses a string if needed).
    pub fn get_int(&self, key: &str) -> Option<i64> {
        match self.resolve(key)? {
            Layer::Env(v) => v.parse().ok(),
            Layer::Explicit(v) | Layer::Flag(v) | Layer::File(v) | Layer::Default(v) => {
                v.to_int_coerce()
            }
        }
    }

    /// Looks up the value as `i64`. Same as [`Config::get_int`] (Viper naming compatibility).
    pub fn get_int64(&self, key: &str) -> Option<i64> {
        self.get_int(key)
    }

    /// Looks up the value as `u64` (negative values are `None`).
    pub fn get_uint(&self, key: &str) -> Option<u64> {
        match self.resolve(key)? {
            Layer::Env(v) => v.parse().ok(),
            Layer::Explicit(v) | Layer::Flag(v) | Layer::File(v) | Layer::Default(v) => {
                v.to_uint_coerce()
            }
        }
    }

    /// Looks up the value as `bool` (`true/false/1/0/yes/no` accepted).
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        match self.resolve(key)? {
            Layer::Env(v) => match v.as_str() {
                "true" | "1" | "yes" => Some(true),
                "false" | "0" | "no" => Some(false),
                _ => None,
            },
            Layer::Explicit(v) | Layer::Flag(v) | Layer::File(v) | Layer::Default(v) => {
                v.to_bool_coerce()
            }
        }
    }

    /// Looks up the value as `f64` (parses a string if needed).
    pub fn get_float(&self, key: &str) -> Option<f64> {
        match self.resolve(key)? {
            Layer::Env(v) => v.parse().ok(),
            Layer::Explicit(v) | Layer::Flag(v) | Layer::File(v) | Layer::Default(v) => {
                v.to_float_coerce()
            }
        }
    }

    /// Looks up the value as `Vec<String>`. Environment variables are split on commas (`,`) into an array.
    pub fn get_string_vec(&self, key: &str) -> Option<Vec<String>> {
        match self.resolve(key)? {
            Layer::Env(v) => Some(
                v.split(',')
                    .map(|s| s.trim().to_owned())
                    .filter(|s| !s.is_empty())
                    .collect(),
            ),
            Layer::Explicit(v) | Layer::Flag(v) | Layer::File(v) | Layer::Default(v) => v
                .as_array()
                .map(|arr| arr.iter().filter_map(|v| v.to_string_coerce()).collect()),
        }
    }

    /// Looks up the value as `Vec<String>`. Same as [`Config::get_string_vec`] (Viper naming compatibility).
    pub fn get_string_slice(&self, key: &str) -> Option<Vec<String>> {
        self.get_string_vec(key)
    }

    /// Looks up the value as [`std::time::Duration`].
    ///
    /// Strings are parsed Go-style (`"1h30m"`, `"250ms"`); numbers are interpreted as seconds.
    pub fn get_duration(&self, key: &str) -> Option<Duration> {
        match self.resolve(key)? {
            Layer::Env(v) => ConfigValue::String(v).to_duration_coerce(),
            Layer::Explicit(v) | Layer::Flag(v) | Layer::File(v) | Layer::Default(v) => {
                v.to_duration_coerce()
            }
        }
    }

    /// Looks up the value as [`std::time::SystemTime`].
    ///
    /// Numbers are interpreted as Unix epoch seconds; strings as RFC3339 or Unix seconds.
    pub fn get_time(&self, key: &str) -> Option<SystemTime> {
        match self.resolve(key)? {
            Layer::Env(v) => ConfigValue::String(v).to_time_coerce(),
            Layer::Explicit(v) | Layer::Flag(v) | Layer::File(v) | Layer::Default(v) => {
                v.to_time_coerce()
            }
        }
    }

    /// Looks up the value as a byte count (`u64`). Supports 1024-based units such as `"1KB"`, `"1.5MB"`, `"2GiB"`.
    pub fn get_size_in_bytes(&self, key: &str) -> Option<u64> {
        match self.resolve(key)? {
            Layer::Env(v) => ConfigValue::String(v).to_size_in_bytes_coerce(),
            Layer::Explicit(v) | Layer::Flag(v) | Layer::File(v) | Layer::Default(v) => {
                v.to_size_in_bytes_coerce()
            }
        }
    }

    // ── Enumeration & subtrees ────────────────────────────────────────────────

    /// Returns the keys from all layers, sorted (Viper's `AllKeys`).
    pub fn all_keys(&self) -> Vec<String> {
        let mut keys: BTreeSet<String> = BTreeSet::new();
        keys.extend(self.defaults.keys().cloned());
        keys.extend(self.file_values.keys().cloned());
        keys.extend(self.flag_values.keys().cloned());
        keys.extend(self.explicit_values.keys().cloned());
        keys.extend(self.explicit_env_bindings.keys().cloned());
        keys.into_iter().collect()
    }

    /// Nested settings tree merging all layers (Viper's `AllSettings`).
    ///
    /// The automatic env layer is not included because its keys cannot be enumerated.
    pub fn all_settings(&self) -> SettingsMap {
        build_settings(
            self.all_keys()
                .into_iter()
                .filter_map(|k| self.get(&k).map(|v| (k, v))),
        )
    }

    /// Relative key -> value list directly under `key` (subtree).
    fn subtree(&self, key: &str) -> Vec<(String, ConfigValue)> {
        let prefix = format!("{}.", self.canonical_key(key));
        self.all_keys()
            .into_iter()
            .filter_map(|k| {
                k.strip_prefix(&prefix)
                    .and_then(|rest| self.get(&k).map(|v| (rest.to_owned(), v)))
            })
            .collect()
    }

    /// Returns the values under `key` as a nested tree (Viper's `GetStringMap`).
    pub fn get_string_map(&self, key: &str) -> Option<SettingsMap> {
        let entries = self.subtree(key);
        (!entries.is_empty()).then(|| build_settings(entries))
    }

    /// Returns the values under `key` as a `String` map (Viper's `GetStringMapString`).
    pub fn get_string_map_string(&self, key: &str) -> Option<BTreeMap<String, String>> {
        let entries = self.subtree(key);
        if entries.is_empty() {
            return None;
        }
        Some(
            entries
                .into_iter()
                .filter_map(|(k, v)| v.to_string_coerce().map(|s| (k, s)))
                .collect(),
        )
    }

    /// Returns the array values under `key` as a `Vec<String>` map (Viper's `GetStringMapStringSlice`).
    pub fn get_string_map_string_slice(&self, key: &str) -> Option<BTreeMap<String, Vec<String>>> {
        let entries = self.subtree(key);
        if entries.is_empty() {
            return None;
        }
        Some(
            entries
                .into_iter()
                .filter_map(|(k, v)| {
                    v.as_array()
                        .map(|arr| (k, arr.iter().filter_map(|x| x.to_string_coerce()).collect()))
                })
                .collect(),
        )
    }

    /// Returns a new `Config` containing only the subtree under `key` (Viper's `Sub`).
    ///
    /// Copies the stored layers and env settings. Automatic env lookup is relative to the key,
    /// so the env variable names may differ from the original.
    pub fn sub(&self, key: &str) -> Config {
        let prefix = format!("{}.", self.canonical_key(key));
        let strip = |k: &str| k.strip_prefix(&prefix).map(str::to_owned);
        let collect = |layer: &HashMap<String, ConfigValue>| {
            layer
                .iter()
                .filter_map(|(k, v)| strip(k).map(|k| (k, v.clone())))
                .collect::<HashMap<_, _>>()
        };
        Config {
            auto_env: self.auto_env,
            allow_empty_env: self.allow_empty_env,
            env_prefix: self.env_prefix.clone(),
            env_prefix_upper: self.env_prefix_upper.clone(),
            env_key_replacer: self.env_key_replacer.clone(),
            key_delim: self.key_delim,
            defaults: collect(&self.defaults),
            file_values: collect(&self.file_values),
            flag_values: collect(&self.flag_values),
            explicit_values: collect(&self.explicit_values),
            explicit_env_bindings: self
                .explicit_env_bindings
                .iter()
                .filter_map(|(k, v)| strip(k).map(|k| (k, v.clone())))
                .collect(),
            ..Config::default()
        }
    }

    // ── Reading & merging ─────────────────────────────────────────────────────

    /// Reads from a reader and replaces the file layer (Viper's `ReadConfig`).
    ///
    /// The format must be set via [`Config::set_config_type`].
    pub fn read_config<R: Read>(&mut self, mut reader: R) -> Result<()> {
        let ext = self
            .config_type
            .clone()
            .ok_or(WrCliError::ConfigTypeNotSet)?;
        let mut content = String::new();
        reader.read_to_string(&mut content)?;
        self.file_values = parse_config_content(&content, &ext, "<reader>")?;
        self.loaded_file = None;
        Ok(())
    }

    /// Merges a config file into the existing file layer (Viper's `MergeInConfig`).
    ///
    /// Keys that already exist are kept.
    pub fn merge_in_config(&mut self, path: impl AsRef<Path>) -> Result<()> {
        let expanded = expand_path(path.as_ref());
        let ext = expanded
            .extension()
            .and_then(|e| e.to_str())
            .map(str::to_lowercase)
            .ok_or_else(|| WrCliError::UnsupportedConfigFormat(String::new()))?;
        let content = std::fs::read_to_string(&expanded)?;
        let merged = parse_config_content(&content, &ext, &expanded.display().to_string())?;
        self.merge_values(merged);
        Ok(())
    }

    /// Merges a key-value map into the existing file layer (Viper's `MergeConfigMap`).
    ///
    /// Keys that already exist are kept.
    pub fn merge_config_map(&mut self, map: impl IntoIterator<Item = (String, ConfigValue)>) {
        let merged: HashMap<String, ConfigValue> = map
            .into_iter()
            .map(|(k, v)| (self.canonical_key(&k), v))
            .collect();
        self.merge_values(merged);
    }

    fn merge_values(&mut self, merged: HashMap<String, ConfigValue>) {
        for (k, v) in merged {
            self.file_values.entry(k).or_insert(v);
        }
    }

    // ── Writing ───────────────────────────────────────────────────────────────

    /// Writes the current config to a file (Viper's `WriteConfigAs`).
    ///
    /// Determines the format from the extension and serializes the nested settings.
    pub fn write_config_as(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = expand_path(path.as_ref());
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(str::to_lowercase)
            .ok_or_else(|| WrCliError::UnsupportedConfigFormat(String::new()))?;
        let content = serialize(&self.all_settings(), &ext, &path.display().to_string())?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Writes only if the target file does not exist (Viper's `SafeWriteConfigAs`).
    pub fn safe_write_config_as(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = expand_path(path.as_ref());
        if path.exists() {
            return Err(WrCliError::ConfigFileExists(path.display().to_string()));
        }
        self.write_config_as(path)
    }

    // ── Deserialization (serde) ───────────────────────────────────────────────

    /// Deserializes the settings merged from all layers into `T` (Viper's `Unmarshal`).
    #[cfg(feature = "serde")]
    pub fn unmarshal<T: serde::de::DeserializeOwned>(&self) -> Result<T> {
        let settings = self.all_settings();
        T::deserialize(super::de::ConfigDeserializer::map(&settings))
    }

    /// Deserializes the settings under `key` into `T` (Viper's `UnmarshalKey`).
    #[cfg(feature = "serde")]
    pub fn unmarshal_key<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<T> {
        if let Some(value) = self.get(key) {
            return T::deserialize(super::de::ConfigDeserializer::value(&value));
        }
        let settings = build_settings(self.subtree(key));
        if settings.is_empty() {
            return Err(WrCliError::ConfigDeserializeError(format!(
                "key '{}' not found",
                key
            )));
        }
        T::deserialize(super::de::ConfigDeserializer::map(&settings))
    }

    // ── Watching (WatchConfig) ────────────────────────────────────────────────

    /// Registers a config file change callback (Viper's `OnConfigChange`).
    pub fn on_config_change<F>(mut self, f: F) -> Self
    where
        F: Fn(&Config) + Send + Sync + 'static,
    {
        self.on_change = Some(ChangeCallback(Arc::new(f)));
        self
    }

    /// Sets the polling interval for detecting changes (default 1 second).
    pub fn set_watch_interval(mut self, interval: Duration) -> Self {
        self.watch_interval = interval;
        self
    }

    /// Watches the config file for changes (Viper's `WatchConfig`).
    ///
    /// [`Config::on_config_change`] and [`Config::read_in_config`] must be called first.
    /// Dropping the returned [`ConfigWatcher`] stops watching.
    pub fn watch_config(&mut self) -> Result<ConfigWatcher> {
        let callback = self
            .on_change
            .clone()
            .ok_or(WrCliError::ConfigWatchNotReady)?;
        let path = self
            .loaded_file
            .clone()
            .ok_or(WrCliError::ConfigWatchNotReady)?;
        let mut spec = self.clone();
        spec.on_change = None;
        let interval = self.watch_interval;
        let initial = read_file_snapshot(&path);

        let stop = Arc::new(AtomicBool::new(false));
        let stop_thread = Arc::clone(&stop);
        let handle = std::thread::spawn(move || {
            let mut last = initial;
            while !stop_thread.load(Ordering::Relaxed) {
                std::thread::sleep(interval);
                let current = read_file_snapshot(&path);
                if current.is_some() && current != last {
                    last = current;
                    let mut reloaded = spec.clone();
                    if reloaded.read_in_config().is_ok() {
                        (callback.0)(&reloaded);
                    }
                }
            }
        });
        Ok(ConfigWatcher {
            stop,
            handle: Some(handle),
        })
    }

    // ── Internal helpers ──────────────────────────────────────────────────────

    fn env_lookup(&self, key: &str) -> Option<String> {
        if let Some(env_var) = self.explicit_env_bindings.get(key)
            && let Some(v) = read_env(env_var, self.allow_empty_env)
        {
            return Some(v);
        }
        if self.auto_env
            && let Some(v) = read_env(&self.key_to_env_var(key), self.allow_empty_env)
        {
            return Some(v);
        }
        None
    }

    fn key_to_env_var(&self, key: &str) -> String {
        let upper = match &self.env_prefix_upper {
            Some(prefix) => format!("{}_{}", prefix, key).to_uppercase(),
            None => key.to_uppercase(),
        };
        match &self.env_key_replacer {
            Some(pairs) => pairs.iter().fold(upper, |acc, (from, to)| {
                acc.replace(from.as_str(), to.as_str())
            }),
            None => upper.replace(['.', '-'], "_"),
        }
    }
}

/// Watcher handle returned by [`Config::watch_config`]. Dropping it stops the watcher thread.
pub struct ConfigWatcher {
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl ConfigWatcher {
    /// Stops watching and waits for the thread to finish.
    pub fn stop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for ConfigWatcher {
    fn drop(&mut self) {
        self.stop();
    }
}

fn read_file_snapshot(path: &Path) -> Option<String> {
    std::fs::read_to_string(path).ok()
}

/// Env variable lookup. If `allow_empty` is false, an empty value is treated as unset.
fn read_env(var: &str, allow_empty: bool) -> Option<String> {
    match std::env::var(var) {
        Ok(v) if allow_empty || !v.is_empty() => Some(v),
        _ => None,
    }
}

/// List of supported config extensions based on enabled features.
fn supported_extensions() -> Vec<String> {
    #[allow(unused_mut)]
    let mut exts = Vec::new();
    #[cfg(feature = "toml-config")]
    exts.push("toml".to_owned());
    #[cfg(feature = "json-config")]
    exts.push("json".to_owned());
    #[cfg(feature = "yaml-config")]
    {
        exts.push("yaml".to_owned());
        exts.push("yml".to_owned());
    }
    #[cfg(feature = "dotenv-config")]
    {
        exts.push("env".to_owned());
        exts.push("dotenv".to_owned());
    }
    #[cfg(feature = "properties-config")]
    {
        exts.push("properties".to_owned());
        exts.push("props".to_owned());
        exts.push("prop".to_owned());
    }
    #[cfg(feature = "ini-config")]
    exts.push("ini".to_owned());
    exts
}

fn expand_path(path: &Path) -> PathBuf {
    let s = path.to_string_lossy();
    match shellexpand::full(&s) {
        Ok(expanded) => PathBuf::from(expanded.as_ref()),
        Err(e) => {
            log::warn!("failed to expand path '{}': {}", s, e);
            path.to_path_buf()
        }
    }
}

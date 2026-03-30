use std::collections::HashMap;
use std::path::PathBuf;
use crate::error::{Result, WrCliError};
use crate::flag::FlagValue;

// ── ConfigValue ───────────────────────────────────────────────────────────────

/// A typed value stored in the config system.
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Array(Vec<ConfigValue>),
}

impl ConfigValue {
    pub fn as_bool(&self) -> Option<bool> {
        if let ConfigValue::Bool(v) = self { Some(*v) } else { None }
    }
    pub fn as_int(&self) -> Option<i64> {
        if let ConfigValue::Int(v) = self { Some(*v) } else { None }
    }
    pub fn as_float(&self) -> Option<f64> {
        if let ConfigValue::Float(v) = self { Some(*v) } else { None }
    }
    pub fn as_str(&self) -> Option<&str> {
        if let ConfigValue::String(v) = self { Some(v) } else { None }
    }

    fn to_string_coerce(&self) -> Option<String> {
        match self {
            ConfigValue::String(s) => Some(s.clone()),
            ConfigValue::Int(i) => Some(i.to_string()),
            ConfigValue::Float(f) => Some(f.to_string()),
            ConfigValue::Bool(b) => Some(b.to_string()),
            ConfigValue::Array(_) => None,
        }
    }

    fn to_int_coerce(&self) -> Option<i64> {
        match self {
            ConfigValue::Int(i) => Some(*i),
            ConfigValue::Float(f) => Some(*f as i64),
            ConfigValue::String(s) => s.parse().ok(),
            _ => None,
        }
    }

    fn to_float_coerce(&self) -> Option<f64> {
        match self {
            ConfigValue::Float(f) => Some(*f),
            ConfigValue::Int(i) => Some(*i as f64),
            ConfigValue::String(s) => s.parse().ok(),
            _ => None,
        }
    }

    fn to_bool_coerce(&self) -> Option<bool> {
        match self {
            ConfigValue::Bool(b) => Some(*b),
            ConfigValue::String(s) => match s.as_str() {
                "true" | "1" | "yes" => Some(true),
                "false" | "0" | "no" => Some(false),
                _ => None,
            },
            ConfigValue::Int(i) => Some(*i != 0),
            _ => None,
        }
    }
}

// ── From impls ────────────────────────────────────────────────────────────────

impl From<bool> for ConfigValue { fn from(v: bool) -> Self { ConfigValue::Bool(v) } }
impl From<i64> for ConfigValue { fn from(v: i64) -> Self { ConfigValue::Int(v) } }
impl From<i32> for ConfigValue { fn from(v: i32) -> Self { ConfigValue::Int(v as i64) } }
impl From<u32> for ConfigValue { fn from(v: u32) -> Self { ConfigValue::Int(v as i64) } }
impl From<usize> for ConfigValue { fn from(v: usize) -> Self { ConfigValue::Int(v as i64) } }
impl From<f64> for ConfigValue { fn from(v: f64) -> Self { ConfigValue::Float(v) } }
impl From<f32> for ConfigValue { fn from(v: f32) -> Self { ConfigValue::Float(v as f64) } }
impl From<String> for ConfigValue { fn from(v: String) -> Self { ConfigValue::String(v) } }
impl From<&str> for ConfigValue { fn from(v: &str) -> Self { ConfigValue::String(v.to_owned()) } }

/// Convert a parsed `FlagValue` into a `ConfigValue` for layer-4 binding.
impl From<&FlagValue> for ConfigValue {
    fn from(fv: &FlagValue) -> Self {
        match fv {
            FlagValue::Bool(b) => ConfigValue::Bool(*b),
            FlagValue::String(s) => ConfigValue::String(s.clone()),
            FlagValue::Int(i) => ConfigValue::Int(*i),
            FlagValue::Float(f) => ConfigValue::Float(*f),
            FlagValue::StringVec(v) => {
                ConfigValue::Array(v.iter().map(|s| ConfigValue::String(s.clone())).collect())
            }
            FlagValue::IntVec(v) => {
                ConfigValue::Array(v.iter().map(|i| ConfigValue::Int(*i)).collect())
            }
        }
    }
}

// ── Config ────────────────────────────────────────────────────────────────────

/// Configuration store inspired by Go's Viper.
///
/// Reads from four sources in ascending priority order:
/// 1. Programmatic defaults ([`Config::set_default`])
/// 2. Config file ([`Config::read_in_config`])
/// 3. Environment variables ([`Config::automatic_env`], [`Config::bind_env`])
/// 4. CLI flag overrides (automatically injected after flag parsing)
///
/// Supports dot-notation keys for nested access: `"database.host"`.
///
/// # Supported config formats
///
/// | Format | Feature flag    |
/// |--------|-----------------|
/// | TOML   | `toml-config`   |
/// | JSON   | `json-config`   |
/// | YAML   | `yaml-config`   |
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
/// cfg.read_in_config().ok(); // non-fatal if file not found
/// let port = cfg.get_int("server.port").unwrap_or(8080);
/// ```
#[derive(Debug, Default)]
pub struct Config {
    // Priority layer 1 (lowest): programmatic defaults
    defaults: HashMap<String, ConfigValue>,
    // Priority layer 2: config file values
    file_values: HashMap<String, ConfigValue>,
    // Priority layer 4 (highest): CLI flag overrides, injected at execution time
    flag_values: HashMap<String, ConfigValue>,

    // Config file settings
    config_name: Option<String>,
    config_type: Option<String>,
    config_paths: Vec<PathBuf>,

    // Env settings
    env_prefix: Option<String>,
    auto_env: bool,
    /// config key → env var name
    explicit_env_bindings: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        Default::default()
    }

    // ── File config ──────────────────────────────────────────────────────────

    /// Base name of the config file without extension (e.g. `"config"`, `"myapp"`).
    pub fn set_config_name(mut self, name: &str) -> Self {
        self.config_name = Some(name.to_owned());
        self
    }

    /// Format of the config file: `"toml"`, `"json"`, or `"yaml"` / `"yml"`.
    pub fn set_config_type(mut self, t: &str) -> Self {
        self.config_type = Some(t.to_owned());
        self
    }

    /// Add a directory to search for the config file. Supports `~` and `$VAR` expansion.
    pub fn add_config_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.config_paths.push(path.into());
        self
    }

    /// Load the config file from the first matching path.
    ///
    /// Returns [`WrCliError::ConfigFileNotFound`] if no file is found.
    /// Call `.read_in_config().ok()` to silently ignore a missing file.
    pub fn read_in_config(&mut self) -> Result<()> {
        let name = self.config_name.as_deref().ok_or_else(|| {
            WrCliError::ConfigFileNotFound {
                name: "<not set>".to_owned(),
                paths: vec![],
            }
        })?;
        let ext = self.config_type.as_deref().unwrap_or("toml");
        let filename = format!("{}.{}", name, ext);

        // Find the first matching file without cloning the paths Vec.
        let found = self.config_paths.iter()
            .map(|p| expand_path(p).join(&filename))
            .find(|p| p.exists());

        if let Some(file_path) = found {
            let content = std::fs::read_to_string(&file_path)?;
            self.file_values =
                parse_config_content(&content, ext, &file_path.display().to_string())?;
            return Ok(());
        }

        Err(WrCliError::ConfigFileNotFound {
            name: filename,
            paths: self
                .config_paths
                .iter()
                .map(|p| p.display().to_string())
                .collect(),
        })
    }

    // ── Defaults ─────────────────────────────────────────────────────────────

    /// Set a programmatic default (lowest priority layer).
    pub fn set_default(mut self, key: &str, val: impl Into<ConfigValue>) -> Self {
        self.defaults.insert(key.to_owned(), val.into());
        self
    }

    // ── Environment ──────────────────────────────────────────────────────────

    /// Automatically look up environment variables for every key that is queried.
    ///
    /// The env var name is derived from the key: uppercase + `.` → `_`.
    /// With a prefix set via [`Config::set_env_prefix`]: `PREFIX_KEY_SUBKEY`.
    pub fn automatic_env(mut self) -> Self {
        self.auto_env = true;
        self
    }

    /// Prepend a prefix to all auto-env variable lookups (e.g. `"MYAPP"`).
    pub fn set_env_prefix(mut self, prefix: &str) -> Self {
        self.env_prefix = Some(prefix.to_owned());
        self
    }

    /// Bind a config key explicitly to a named environment variable.
    ///
    /// Takes priority over auto-env but is lower priority than flag overrides.
    pub fn bind_env(mut self, key: &str, env_var: &str) -> Self {
        self.explicit_env_bindings
            .insert(key.to_owned(), env_var.to_owned());
        self
    }

    // ── Internal: flag binding ────────────────────────────────────────────────

    /// Inject an explicitly-set CLI flag value as the highest-priority config layer.
    ///
    /// Called automatically by the command execution engine for every flag that
    /// the user provided on the command line (defaults are NOT injected).
    pub(crate) fn bind_flag_value(&mut self, key: &str, val: ConfigValue) {
        self.flag_values.insert(key.to_owned(), val);
    }

    // ── Getters ──────────────────────────────────────────────────────────────

    /// Get a raw [`ConfigValue`]. Priority: CLI flags > env vars > config file > defaults.
    pub fn get(&self, key: &str) -> Option<&ConfigValue> {
        if let Some(v) = self.flag_values.get(key) { return Some(v); }
        if let Some(v) = self.file_values.get(key) { return Some(v); }
        self.defaults.get(key)
    }

    /// Get a value as `String` (coerces numeric/bool values).
    pub fn get_string(&self, key: &str) -> Option<String> {
        if let Some(v) = self.flag_values.get(key) { return v.to_string_coerce(); }
        if let Some(v) = self.env_lookup(key) { return Some(v); }
        if let Some(v) = self.file_values.get(key) { return v.to_string_coerce(); }
        self.defaults.get(key)?.to_string_coerce()
    }

    /// Get a value as `i64` (parses string values if needed).
    pub fn get_int(&self, key: &str) -> Option<i64> {
        if let Some(v) = self.flag_values.get(key) { return v.to_int_coerce(); }
        if let Some(v) = self.env_lookup(key) { return v.parse().ok(); }
        if let Some(v) = self.file_values.get(key) { return v.to_int_coerce(); }
        self.defaults.get(key)?.to_int_coerce()
    }

    /// Get a value as `bool` (accepts `true/false/1/0/yes/no`).
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        if let Some(v) = self.flag_values.get(key) { return v.to_bool_coerce(); }
        if let Some(v) = self.env_lookup(key) {
            return match v.as_str() {
                "true" | "1" | "yes" => Some(true),
                "false" | "0" | "no" => Some(false),
                _ => None,
            };
        }
        if let Some(v) = self.file_values.get(key) { return v.to_bool_coerce(); }
        self.defaults.get(key)?.to_bool_coerce()
    }

    /// Get a value as `f64` (parses string values if needed).
    pub fn get_float(&self, key: &str) -> Option<f64> {
        if let Some(v) = self.flag_values.get(key) { return v.to_float_coerce(); }
        if let Some(v) = self.env_lookup(key) { return v.parse().ok(); }
        if let Some(v) = self.file_values.get(key) { return v.to_float_coerce(); }
        self.defaults.get(key)?.to_float_coerce()
    }

    /// Get a value as `Vec<String>`. Extracts string-coerced items from an `Array` config value.
    /// Env vars are not consulted (no standard format for array env vars).
    pub fn get_string_vec(&self, key: &str) -> Option<Vec<String>> {
        let cv = self.flag_values.get(key)
            .or_else(|| self.file_values.get(key))
            .or_else(|| self.defaults.get(key))?;
        if let ConfigValue::Array(arr) = cv {
            Some(arr.iter().filter_map(|v| v.to_string_coerce()).collect())
        } else {
            None
        }
    }

    // ── Internal helpers ──────────────────────────────────────────────────────

    /// Check explicit env binding, then auto-env (priority 3 layer).
    fn env_lookup(&self, key: &str) -> Option<String> {
        if let Some(env_var) = self.explicit_env_bindings.get(key) {
            if let Ok(v) = std::env::var(env_var) {
                return Some(v);
            }
        }
        if self.auto_env {
            if let Ok(v) = std::env::var(self.key_to_env_var(key)) {
                return Some(v);
            }
        }
        None
    }

    fn key_to_env_var(&self, key: &str) -> String {
        let upper = key.replace('.', "_").replace('-', "_").to_uppercase();
        match &self.env_prefix {
            Some(prefix) => format!("{}_{}", prefix.to_uppercase(), upper),
            None => upper,
        }
    }
}

// ── Config file parsing ───────────────────────────────────────────────────────

fn expand_path(path: &PathBuf) -> PathBuf {
    let s = path.to_string_lossy();
    match shellexpand::full(&s) {
        Ok(expanded) => PathBuf::from(expanded.as_ref()),
        Err(_) => path.clone(),
    }
}

fn parse_config_content(
    content: &str,
    ext: &str,
    path: &str,
) -> Result<HashMap<String, ConfigValue>> {
    match ext {
        #[cfg(feature = "toml-config")]
        "toml" => parse_toml(content, path),

        #[cfg(feature = "json-config")]
        "json" => parse_json(content, path),

        #[cfg(feature = "yaml-config")]
        "yaml" | "yml" => parse_yaml(content, path),

        other => Err(WrCliError::UnsupportedConfigFormat(other.to_owned())),
    }
}

// ── TOML ──────────────────────────────────────────────────────────────────────

#[cfg(feature = "toml-config")]
fn parse_toml(content: &str, path: &str) -> Result<HashMap<String, ConfigValue>> {
    let value: toml::Value = toml::from_str(content).map_err(|e| WrCliError::ConfigParseError {
        path: path.to_owned(),
        source: e.to_string(),
    })?;
    let mut map = HashMap::new();
    flatten_toml("", &value, &mut map);
    Ok(map)
}

#[cfg(feature = "toml-config")]
fn toml_scalar(v: &toml::Value) -> Option<ConfigValue> {
    match v {
        toml::Value::String(s)   => Some(ConfigValue::String(s.clone())),
        toml::Value::Integer(i)  => Some(ConfigValue::Int(*i)),
        toml::Value::Float(f)    => Some(ConfigValue::Float(*f)),
        toml::Value::Boolean(b)  => Some(ConfigValue::Bool(*b)),
        toml::Value::Datetime(dt) => Some(ConfigValue::String(dt.to_string())),
        _ => None,
    }
}

#[cfg(feature = "toml-config")]
fn flatten_toml(prefix: &str, value: &toml::Value, map: &mut HashMap<String, ConfigValue>) {
    match value {
        toml::Value::Table(t) => {
            for (k, v) in t {
                let key = child_key(prefix, k);
                flatten_toml(&key, v, map);
            }
        }
        toml::Value::Array(arr) => {
            let cv: Vec<ConfigValue> = arr.iter().filter_map(toml_scalar).collect();
            map.insert(prefix.to_owned(), ConfigValue::Array(cv));
        }
        other => {
            if let Some(cv) = toml_scalar(other) {
                map.insert(prefix.to_owned(), cv);
            }
        }
    }
}

// ── JSON ──────────────────────────────────────────────────────────────────────

#[cfg(feature = "json-config")]
fn parse_json(content: &str, path: &str) -> Result<HashMap<String, ConfigValue>> {
    let value: serde_json::Value =
        serde_json::from_str(content).map_err(|e| WrCliError::ConfigParseError {
            path: path.to_owned(),
            source: e.to_string(),
        })?;
    let mut map = HashMap::new();
    flatten_json("", &value, &mut map);
    Ok(map)
}

#[cfg(feature = "json-config")]
fn json_scalar(v: &serde_json::Value) -> Option<ConfigValue> {
    match v {
        serde_json::Value::String(s) => Some(ConfigValue::String(s.clone())),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64()      { Some(ConfigValue::Int(i)) }
            else if let Some(f) = n.as_f64() { Some(ConfigValue::Float(f)) }
            else { None }
        }
        serde_json::Value::Bool(b) => Some(ConfigValue::Bool(*b)),
        _ => None,
    }
}

#[cfg(feature = "json-config")]
fn flatten_json(prefix: &str, value: &serde_json::Value, map: &mut HashMap<String, ConfigValue>) {
    match value {
        serde_json::Value::Object(obj) => {
            for (k, v) in obj {
                let key = child_key(prefix, k);
                flatten_json(&key, v, map);
            }
        }
        serde_json::Value::Array(arr) => {
            let cv: Vec<ConfigValue> = arr.iter().filter_map(json_scalar).collect();
            map.insert(prefix.to_owned(), ConfigValue::Array(cv));
        }
        serde_json::Value::Null => {}
        other => {
            if let Some(cv) = json_scalar(other) {
                map.insert(prefix.to_owned(), cv);
            }
        }
    }
}

// ── YAML ──────────────────────────────────────────────────────────────────────

#[cfg(feature = "yaml-config")]
fn parse_yaml(content: &str, path: &str) -> Result<HashMap<String, ConfigValue>> {
    let value: serde_yml::Value =
        serde_yml::from_str(content).map_err(|e| WrCliError::ConfigParseError {
            path: path.to_owned(),
            source: e.to_string(),
        })?;
    let mut map = HashMap::new();
    flatten_yaml("", &value, &mut map);
    Ok(map)
}

#[cfg(feature = "yaml-config")]
fn yaml_scalar(v: &serde_yml::Value) -> Option<ConfigValue> {
    match v {
        serde_yml::Value::String(s) => Some(ConfigValue::String(s.clone())),
        serde_yml::Value::Number(n) => {
            if let Some(i) = n.as_i64()      { Some(ConfigValue::Int(i)) }
            else if let Some(f) = n.as_f64() { Some(ConfigValue::Float(f)) }
            else { None }
        }
        serde_yml::Value::Bool(b) => Some(ConfigValue::Bool(*b)),
        _ => None,
    }
}

#[cfg(feature = "yaml-config")]
fn flatten_yaml(prefix: &str, value: &serde_yml::Value, map: &mut HashMap<String, ConfigValue>) {
    match value {
        serde_yml::Value::Mapping(m) => {
            for (k, v) in m {
                if let Some(key_str) = k.as_str() {
                    let key = child_key(prefix, key_str);
                    flatten_yaml(&key, v, map);
                }
            }
        }
        serde_yml::Value::Sequence(arr) => {
            let cv: Vec<ConfigValue> = arr.iter().filter_map(yaml_scalar).collect();
            map.insert(prefix.to_owned(), ConfigValue::Array(cv));
        }
        serde_yml::Value::Null => {}
        serde_yml::Value::Tagged(tagged) => {
            flatten_yaml(prefix, &tagged.value, map);
        }
        other => {
            if let Some(cv) = yaml_scalar(other) {
                map.insert(prefix.to_owned(), cv);
            }
        }
    }
}

// ── Shared helpers ────────────────────────────────────────────────────────────

fn child_key(prefix: &str, key: &str) -> String {
    if prefix.is_empty() { key.to_owned() } else { format!("{}.{}", prefix, key) }
}

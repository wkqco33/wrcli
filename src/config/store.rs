use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use super::parser::parse_config_content;
use super::settings::{SettingsMap, build_settings};
use super::value::ConfigValue;
use crate::error::{Result, WrCliError};

/// Go의 Viper에서 영감을 받은 설정 저장소.
///
/// 네 가지 소스를 오름차순 우선순위로 읽음:
/// 1. 프로그래밍 기본값 ([`Config::set_default`])
/// 2. 설정 파일 ([`Config::read_in_config`])
/// 3. 환경 변수 ([`Config::automatic_env`], [`Config::bind_env`])
/// 4. CLI 플래그 오버라이드 (플래그 파싱 후 자동 주입)
///
/// 중첩 접근에 점 표기법 키 지원: `"database.host"`.
///
/// # 지원 포맷
///
/// | 포맷 | Feature flag    |
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
#[derive(Debug)]
pub struct Config {
    // 우선순위 레이어 1 (최저): 프로그래밍 기본값
    defaults: HashMap<String, ConfigValue>,
    // 우선순위 레이어 2: 설정 파일 값
    file_values: HashMap<String, ConfigValue>,
    // 우선순위 레이어 3: CLI 플래그 오버라이드
    flag_values: HashMap<String, ConfigValue>,
    // 우선순위 레이어 4 (최고): `set()`으로 지정한 명시 값
    explicit_values: HashMap<String, ConfigValue>,
    // 별칭 → 정규 키
    aliases: HashMap<String, String>,

    config_name: Option<String>,
    config_type: Option<String>,
    config_paths: Vec<PathBuf>,
    config_file: Option<PathBuf>,

    env_prefix: Option<String>,
    env_prefix_upper: Option<String>,
    auto_env: bool,
    allow_empty_env: bool,
    env_key_replacer: Option<Vec<(String, String)>>,
    explicit_env_bindings: HashMap<String, String>,

    key_delim: char,
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
            env_prefix: None,
            env_prefix_upper: None,
            auto_env: false,
            allow_empty_env: true,
            env_key_replacer: None,
            explicit_env_bindings: HashMap::new(),
            key_delim: '.',
        }
    }
}

/// [`Config::resolve`]가 값을 찾은 레이어.
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

    // ── 파일 설정 ─────────────────────────────────────────────────────────────

    /// 확장자를 제외한 설정 파일 기본 이름 (예: `"config"`, `"myapp"`).
    pub fn set_config_name(mut self, name: &str) -> Self {
        self.config_name = Some(name.to_owned());
        self
    }

    /// 설정 파일 포맷: `"toml"`, `"json"`, `"yaml"` / `"yml"`.
    pub fn set_config_type(mut self, t: &str) -> Self {
        self.config_type = Some(t.to_owned());
        self
    }

    /// 설정 파일 검색할 디렉토리 추가. `~` 및 `$VAR` 확장 지원.
    pub fn add_config_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.config_paths.push(path.into());
        self
    }

    /// 단일 설정 파일 경로를 명시적으로 지정. `~` 및 `$VAR` 확장 지원.
    ///
    /// 설정 이름/타입/검색 경로와 무관하게 이 경로에서 바로 로드한다.
    /// 확장자(`.toml`/`.json`/`.yaml`/`.yml`)에서 포맷을 자동 판별한다.
    pub fn set_config_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.config_file = Some(path.into());
        self
    }

    /// 첫 번째로 일치하는 경로에서 설정 파일 로드.
    ///
    /// 파일을 찾지 못하면 [`WrCliError::ConfigFileNotFound`] 반환.
    /// 파일 없을 때 무시하려면 `.read_in_config().ok()` 사용.
    pub fn read_in_config(&mut self) -> Result<()> {
        if let Some(file) = self.config_file.take() {
            return self.read_explicit_file(file);
        }
        let name = self
            .config_name
            .as_deref()
            .ok_or_else(|| WrCliError::ConfigFileNotFound {
                name: "<not set>".to_owned(),
                paths: vec![],
            })?;

        // 포맷이 지정되지 않았으면 모든 지원 확장자 후보를 순서대로 시도한다.
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
                log::debug!("설정 파일 검색 중: {}", full.display());
                if full.exists() {
                    log::debug!("설정 파일 발견: {}", full.display());
                    let content = std::fs::read_to_string(&full)?;
                    self.file_values =
                        parse_config_content(&content, ext, &full.display().to_string())?;
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
        Ok(())
    }

    /// 사용자가 추가한 검색 경로에 더해 표준 위치를 덧붙인 목록.
    ///
    /// Viper 스타일: 사용자 경로 → `$XDG_CONFIG_HOME/<name>` 또는 `~/.config/<name>`
    /// → `~/.<name>` → 현재 디렉토리.
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

    // ── 기본값 ───────────────────────────────────────────────────────────────

    /// 프로그래밍 기본값 설정 (최저 우선순위).
    pub fn set_default(mut self, key: &str, val: impl Into<ConfigValue>) -> Self {
        let key = self.canonical_key(key);
        self.defaults.insert(key, val.into());
        self
    }

    /// 명시 값을 설정 (최고 우선순위, Viper의 `Set`).
    pub fn set(mut self, key: &str, val: impl Into<ConfigValue>) -> Self {
        let key = self.canonical_key(key);
        self.explicit_values.insert(key, val.into());
        self
    }

    /// `alias`로 조회할 때 `key`의 값을 반환하도록 등록 (Viper의 `RegisterAlias`).
    pub fn register_alias(mut self, alias: &str, key: &str) -> Self {
        let alias = self.canonical_key(alias);
        let key = self.canonical_key(key);
        self.aliases.insert(alias, key);
        self
    }

    /// 중첩 키 구분자 변경 (기본 `'.'`, Viper의 `SetKeyDelimiter`).
    pub fn set_key_delimiter(mut self, delim: char) -> Self {
        self.key_delim = delim;
        self
    }

    /// 환경변수명 생성 시 적용할 치환 쌍 설정 (Viper의 `SetEnvKeyReplacer`).
    ///
    /// 대문자로 변환된 키에 순서대로 적용된다. 기본은 `.`/`-` → `_`.
    pub fn set_env_key_replacer(mut self, pairs: &[(&str, &str)]) -> Self {
        self.env_key_replacer = Some(
            pairs
                .iter()
                .map(|(from, to)| ((*from).to_owned(), (*to).to_owned()))
                .collect(),
        );
        self
    }

    /// 빈 환경변수를 값으로 취급할지 설정 (Viper의 `AllowEmptyEnv`).
    ///
    /// 기본값은 `true`(빈 값도 사용). Viper 기본 동작(빈 값=미설정)은 `false`.
    pub fn allow_empty_env(mut self, allow: bool) -> Self {
        self.allow_empty_env = allow;
        self
    }

    // ── 환경 변수 ─────────────────────────────────────────────────────────────

    /// 조회되는 모든 키에 대해 환경 변수를 자동으로 검색.
    ///
    /// 환경 변수명은 키에서 파생: 대문자 + `.` → `_`.
    /// 접두사 설정 시: `PREFIX_KEY_SUBKEY`.
    pub fn automatic_env(mut self) -> Self {
        self.auto_env = true;
        self
    }

    /// 자동 환경 변수 조회에 접두사 추가 (예: `"MYAPP"`).
    pub fn set_env_prefix(mut self, prefix: &str) -> Self {
        self.env_prefix_upper = Some(prefix.to_uppercase());
        self.env_prefix = Some(prefix.to_owned());
        self
    }

    /// 설정 키를 특정 환경 변수에 명시적으로 바인딩.
    pub fn bind_env(mut self, key: &str, env_var: &str) -> Self {
        let key = self.canonical_key(key);
        self.explicit_env_bindings.insert(key, env_var.to_owned());
        self
    }

    // ── 내부: 플래그 바인딩 ───────────────────────────────────────────────────

    /// CLI 플래그 값을 최고 우선순위 레이어로 주입.
    ///
    /// 명령 실행 엔진이 사용자가 실제 입력한 플래그에 대해 자동 호출.
    pub(crate) fn bind_flag_value(&mut self, key: &str, val: ConfigValue) {
        self.flag_values.insert(key.to_owned(), val);
    }

    // ── Getter ───────────────────────────────────────────────────────────────

    /// 별칭과 커스텀 구분자를 반영한 내부(점 표기) 키.
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

    /// 우선순위 레이어에서 `key`를 찾아 반환. 환경변수만 동적 조회라 소유 문자열을 담는다.
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

    /// 어떤 레이어에서든 값이 있으면 `true` (기본값 포함, Viper와 동일).
    pub fn is_set(&self, key: &str) -> bool {
        self.resolve(key).is_some()
    }

    /// 원시 [`ConfigValue`] 조회. 우선순위: Set > CLI 플래그 > 환경변수 > 설정파일 > 기본값.
    ///
    /// 환경변수는 항상 문자열이므로 [`ConfigValue::String`]으로 감싸 반환됨.
    pub fn get(&self, key: &str) -> Option<ConfigValue> {
        match self.resolve(key)? {
            Layer::Env(v) => Some(ConfigValue::String(v)),
            Layer::Explicit(v) | Layer::Flag(v) | Layer::File(v) | Layer::Default(v) => {
                Some(v.clone())
            }
        }
    }

    /// 저장된 레이어(Set/플래그/파일/기본값)의 [`ConfigValue`] 참조 조회.
    /// 환경변수 레이어는 동적 조회이므로 포함되지 않음.
    pub fn get_ref(&self, key: &str) -> Option<&ConfigValue> {
        let key = self.canonical_key(key);
        self.explicit_values
            .get(&key)
            .or_else(|| self.flag_values.get(&key))
            .or_else(|| self.file_values.get(&key))
            .or_else(|| self.defaults.get(&key))
    }

    /// `String` 으로 값 조회 (숫자/bool 값도 문자열로 강제 변환).
    pub fn get_string(&self, key: &str) -> Option<String> {
        match self.resolve(key)? {
            Layer::Env(v) => Some(v),
            Layer::Explicit(v) | Layer::Flag(v) | Layer::File(v) | Layer::Default(v) => {
                v.to_string_coerce()
            }
        }
    }

    /// `i64` 로 값 조회 (필요 시 문자열 파싱).
    pub fn get_int(&self, key: &str) -> Option<i64> {
        match self.resolve(key)? {
            Layer::Env(v) => v.parse().ok(),
            Layer::Explicit(v) | Layer::Flag(v) | Layer::File(v) | Layer::Default(v) => {
                v.to_int_coerce()
            }
        }
    }

    /// `i64` 로 값 조회. [`Config::get_int`]와 동일 (Viper 명칭 호환).
    pub fn get_int64(&self, key: &str) -> Option<i64> {
        self.get_int(key)
    }

    /// `u64` 로 값 조회 (음수는 `None`).
    pub fn get_uint(&self, key: &str) -> Option<u64> {
        match self.resolve(key)? {
            Layer::Env(v) => v.parse().ok(),
            Layer::Explicit(v) | Layer::Flag(v) | Layer::File(v) | Layer::Default(v) => {
                v.to_uint_coerce()
            }
        }
    }

    /// `bool` 로 값 조회 (`true/false/1/0/yes/no` 허용).
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

    /// `f64` 로 값 조회 (필요 시 문자열 파싱).
    pub fn get_float(&self, key: &str) -> Option<f64> {
        match self.resolve(key)? {
            Layer::Env(v) => v.parse().ok(),
            Layer::Explicit(v) | Layer::Flag(v) | Layer::File(v) | Layer::Default(v) => {
                v.to_float_coerce()
            }
        }
    }

    /// `Vec<String>` 으로 값 조회. 환경변수는 쉼표(`,`)로 구분하여 배열로 파싱.
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

    /// `Vec<String>` 으로 값 조회. [`Config::get_string_vec`]와 동일 (Viper 명칭 호환).
    pub fn get_string_slice(&self, key: &str) -> Option<Vec<String>> {
        self.get_string_vec(key)
    }

    /// [`std::time::Duration`] 으로 값 조회.
    ///
    /// 문자열은 Go 스타일(`"1h30m"`, `"250ms"`)로, 숫자는 초로 해석한다.
    pub fn get_duration(&self, key: &str) -> Option<Duration> {
        match self.resolve(key)? {
            Layer::Env(v) => ConfigValue::String(v).to_duration_coerce(),
            Layer::Explicit(v) | Layer::Flag(v) | Layer::File(v) | Layer::Default(v) => {
                v.to_duration_coerce()
            }
        }
    }

    /// [`std::time::SystemTime`] 으로 값 조회.
    ///
    /// 숫자는 Unix epoch 초, 문자열은 RFC3339 또는 Unix 초로 해석한다.
    pub fn get_time(&self, key: &str) -> Option<SystemTime> {
        match self.resolve(key)? {
            Layer::Env(v) => ConfigValue::String(v).to_time_coerce(),
            Layer::Explicit(v) | Layer::Flag(v) | Layer::File(v) | Layer::Default(v) => {
                v.to_time_coerce()
            }
        }
    }

    /// 바이트 수(`u64`)로 값 조회. `"1KB"`, `"1.5MB"`, `"2GiB"` 등 1024 기반 단위 지원.
    pub fn get_size_in_bytes(&self, key: &str) -> Option<u64> {
        match self.resolve(key)? {
            Layer::Env(v) => ConfigValue::String(v).to_size_in_bytes_coerce(),
            Layer::Explicit(v) | Layer::Flag(v) | Layer::File(v) | Layer::Default(v) => {
                v.to_size_in_bytes_coerce()
            }
        }
    }

    // ── 열거 & 하위 트리 ───────────────────────────────────────────────────────

    /// 모든 레이어의 키를 정렬해서 반환 (Viper의 `AllKeys`).
    pub fn all_keys(&self) -> Vec<String> {
        let mut keys: BTreeSet<String> = BTreeSet::new();
        keys.extend(self.defaults.keys().cloned());
        keys.extend(self.file_values.keys().cloned());
        keys.extend(self.flag_values.keys().cloned());
        keys.extend(self.explicit_values.keys().cloned());
        keys.extend(self.explicit_env_bindings.keys().cloned());
        keys.into_iter().collect()
    }

    /// 모든 레이어를 병합한 중첩 설정 트리 (Viper의 `AllSettings`).
    ///
    /// 자동 env 레이어는 키를 열거할 수 없으므로 포함되지 않는다.
    pub fn all_settings(&self) -> SettingsMap {
        build_settings(
            self.all_keys()
                .into_iter()
                .filter_map(|k| self.get(&k).map(|v| (k, v))),
        )
    }

    /// `key` 바로 아래(하위 트리)의 상대 키 → 값 목록.
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

    /// `key` 하위 값을 중첩 트리로 반환 (Viper의 `GetStringMap`).
    pub fn get_string_map(&self, key: &str) -> Option<SettingsMap> {
        let entries = self.subtree(key);
        (!entries.is_empty()).then(|| build_settings(entries))
    }

    /// `key` 하위 값을 `String` 맵으로 반환 (Viper의 `GetStringMapString`).
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

    /// `key` 하위 배열 값을 `Vec<String>` 맵으로 반환 (Viper의 `GetStringMapStringSlice`).
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

    /// `key` 하위 트리만 담은 새 `Config` 반환 (Viper의 `Sub`).
    ///
    /// 저장된 레이어와 env 설정을 복사한다. 자동 env 조회는 상대 키 기준이므로
    /// 원본과 env 변수명이 달라질 수 있다.
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

    // ── 읽기 & 병합 ──────────────────────────────────────────────────────────

    /// 리더에서 읽어 파일 레이어를 교체 (Viper의 `ReadConfig`).
    ///
    /// 포맷은 [`Config::set_config_type`]으로 지정해야 한다.
    pub fn read_config<R: Read>(&mut self, mut reader: R) -> Result<()> {
        let ext = self
            .config_type
            .clone()
            .ok_or(WrCliError::ConfigTypeNotSet)?;
        let mut content = String::new();
        reader.read_to_string(&mut content)?;
        self.file_values = parse_config_content(&content, &ext, "<reader>")?;
        Ok(())
    }

    /// 설정 파일을 기존 파일 레이어에 병합 (Viper의 `MergeInConfig`).
    ///
    /// 이미 존재하는 키는 유지된다.
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

    /// 키-값 맵을 기존 파일 레이어에 병합 (Viper의 `MergeConfigMap`).
    ///
    /// 이미 존재하는 키는 유지된다.
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

    // ── 내부 헬퍼 ─────────────────────────────────────────────────────────────

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

/// 환경변수 조회. `allow_empty`가 false면 빈 값을 미설정으로 취급.
fn read_env(var: &str, allow_empty: bool) -> Option<String> {
    match std::env::var(var) {
        Ok(v) if allow_empty || !v.is_empty() => Some(v),
        _ => None,
    }
}

/// 활성화된 feature에 따라 지원되는 설정 확장자 목록.
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
    exts
}

fn expand_path(path: &Path) -> PathBuf {
    let s = path.to_string_lossy();
    match shellexpand::full(&s) {
        Ok(expanded) => PathBuf::from(expanded.as_ref()),
        Err(e) => {
            log::warn!("경로 확장 실패 '{}': {}", s, e);
            path.to_path_buf()
        }
    }
}

use std::collections::HashMap;
use std::path::PathBuf;

use crate::error::{Result, WrCliError};
use super::value::ConfigValue;
use super::parser::parse_config_content;

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
#[derive(Debug, Default)]
pub struct Config {
    // 우선순위 레이어 1 (최저): 프로그래밍 기본값
    defaults: HashMap<String, ConfigValue>,
    // 우선순위 레이어 2: 설정 파일 값
    file_values: HashMap<String, ConfigValue>,
    // 우선순위 레이어 4 (최고): CLI 플래그 오버라이드
    flag_values: HashMap<String, ConfigValue>,

    config_name: Option<String>,
    config_type: Option<String>,
    config_paths: Vec<PathBuf>,

    env_prefix: Option<String>,
    auto_env: bool,
    explicit_env_bindings: HashMap<String, String>,
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

    /// 설정 파일을 검색할 디렉토리 추가. `~` 및 `$VAR` 확장 지원.
    pub fn add_config_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.config_paths.push(path.into());
        self
    }

    /// 첫 번째로 일치하는 경로에서 설정 파일 로드.
    ///
    /// 파일을 찾지 못하면 [`WrCliError::ConfigFileNotFound`] 반환.
    /// 파일 없을 때 무시하려면 `.read_in_config().ok()` 사용.
    pub fn read_in_config(&mut self) -> Result<()> {
        let name = self
            .config_name
            .as_deref()
            .ok_or_else(|| WrCliError::ConfigFileNotFound {
                name: "<not set>".to_owned(),
                paths: vec![],
            })?;
        let ext = self.config_type.as_deref().unwrap_or("toml");
        let filename = format!("{}.{}", name, ext);

        for path in &self.config_paths {
            let expanded = expand_path(path);
            let full = expanded.join(&filename);
            log::debug!("설정 파일 검색 중: {}", full.display());
            if full.exists() {
                log::debug!("설정 파일 발견: {}", full.display());
                let content = std::fs::read_to_string(&full)?;
                self.file_values = parse_config_content(&content, ext, &full.display().to_string())?;
                return Ok(());
            }
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

    // ── 기본값 ───────────────────────────────────────────────────────────────

    /// 프로그래밍 기본값 설정 (최저 우선순위).
    pub fn set_default(mut self, key: &str, val: impl Into<ConfigValue>) -> Self {
        self.defaults.insert(key.to_owned(), val.into());
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
        self.env_prefix = Some(prefix.to_owned());
        self
    }

    /// 설정 키를 특정 환경 변수에 명시적으로 바인딩.
    pub fn bind_env(mut self, key: &str, env_var: &str) -> Self {
        self.explicit_env_bindings
            .insert(key.to_owned(), env_var.to_owned());
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

    /// 원시 [`ConfigValue`] 조회. 우선순위: CLI 플래그 > 환경변수 > 설정파일 > 기본값.
    pub fn get(&self, key: &str) -> Option<&ConfigValue> {
        if let Some(v) = self.flag_values.get(key) { return Some(v); }
        if let Some(v) = self.file_values.get(key) { return Some(v); }
        self.defaults.get(key)
    }

    /// `String` 으로 값 조회 (숫자/bool 값도 문자열로 강제 변환).
    pub fn get_string(&self, key: &str) -> Option<String> {
        if let Some(v) = self.flag_values.get(key) { return v.to_string_coerce(); }
        if let Some(v) = self.env_lookup(key) { return Some(v); }
        if let Some(v) = self.file_values.get(key) { return v.to_string_coerce(); }
        self.defaults.get(key)?.to_string_coerce()
    }

    /// `i64` 로 값 조회 (필요 시 문자열 파싱).
    pub fn get_int(&self, key: &str) -> Option<i64> {
        if let Some(v) = self.flag_values.get(key) { return v.to_int_coerce(); }
        if let Some(v) = self.env_lookup(key) { return v.parse().ok(); }
        if let Some(v) = self.file_values.get(key) { return v.to_int_coerce(); }
        self.defaults.get(key)?.to_int_coerce()
    }

    /// `bool` 로 값 조회 (`true/false/1/0/yes/no` 허용).
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

    /// `f64` 로 값 조회 (필요 시 문자열 파싱).
    pub fn get_float(&self, key: &str) -> Option<f64> {
        if let Some(v) = self.flag_values.get(key) { return v.to_float_coerce(); }
        if let Some(v) = self.env_lookup(key) { return v.parse().ok(); }
        if let Some(v) = self.file_values.get(key) { return v.to_float_coerce(); }
        self.defaults.get(key)?.to_float_coerce()
    }

    /// `Vec<String>` 으로 값 조회. 환경변수는 배열 형식 미지원.
    pub fn get_string_vec(&self, key: &str) -> Option<Vec<String>> {
        let cv = self
            .flag_values
            .get(key)
            .or_else(|| self.file_values.get(key))
            .or_else(|| self.defaults.get(key))?;
        if let ConfigValue::Array(arr) = cv {
            Some(arr.iter().filter_map(|v| v.to_string_coerce()).collect())
        } else {
            None
        }
    }

    // ── 내부 헬퍼 ─────────────────────────────────────────────────────────────

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

fn expand_path(path: &PathBuf) -> PathBuf {
    let s = path.to_string_lossy();
    match shellexpand::full(&s) {
        Ok(expanded) => PathBuf::from(expanded.as_ref()),
        Err(e) => {
            log::warn!("경로 확장 실패 '{}': {}", s, e);
            path.clone()
        }
    }
}

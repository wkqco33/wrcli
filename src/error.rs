use std::fmt;

/// `non_exhaustive`: 향후 variant 추가가 semver-breaking이 되지 않도록 함.
/// 소비자는 `match`에 반드시 wildcard(`_`) 분기를 포함해야 함.
#[derive(Debug)]
#[non_exhaustive]
pub enum WrCliError {
    // Command routing errors
    UnknownFlag {
        flag: String,
        command: String,
        /// 편집 거리 기반 근접 후보 (비어 있을 수 있음).
        suggestions: Vec<String>,
    },
    UnknownSubcommand {
        name: String,
        parent: String,
        /// 편집 거리 기반 근접 후보 (비어 있을 수 있음).
        suggestions: Vec<String>,
    },
    MissingRequiredFlag(String),
    /// 플래그는 제공되었지만 값이 누락됨 (예: `--name` without value).
    MissingFlagValue(String),
    InvalidFlagValue {
        flag: String,
        expected: &'static str,
        got: String,
    },
    ArgValidationFailed(String),
    CommandHasNoRunner(String),

    // Config errors
    ConfigFileNotFound {
        name: String,
        paths: Vec<String>,
    },
    ConfigParseError {
        path: String,
        source: String,
    },
    UnsupportedConfigFormat(String),

    // User-surfaced errors from RunE callbacks
    UserError(Box<dyn std::error::Error + Send + Sync>),

    // IO errors
    Io(std::io::Error),

    // Completion generation errors
    UnsupportedCompletionShell(String),

    /// `read_config` 호출 시 설정 포맷이 지정되지 않음.
    ConfigTypeNotSet,

    /// `safe_write_config_as` 대상 파일이 이미 존재함.
    ConfigFileExists(String),

    /// `unmarshal` 시 serde 역직렬화 실패.
    ConfigDeserializeError(String),

    /// `watch_config` 선행 조건(콜백/로드된 파일)이 충족되지 않음.
    ConfigWatchNotReady,

    /// `mutually_exclusive` 그룹에서 둘 이상의 플래그가 동시에 지정됨.
    MutuallyExclusiveFlags {
        group: Vec<String>,
        provided: Vec<String>,
    },

    /// `required_together` 그룹의 일부만 지정됨.
    RequiredFlagsTogether {
        group: Vec<String>,
        missing: Vec<String>,
    },

    /// `one_required` 그룹에서 아무 플래그도 지정되지 않음.
    OneFlagRequired {
        group: Vec<String>,
    },
}

impl WrCliError {
    /// Convenience constructor to wrap any error as a `UserError`.
    pub fn user<E: std::error::Error + Send + Sync + 'static>(e: E) -> Self {
        WrCliError::UserError(Box::new(e))
    }

    /// 사용법(usage) 오류 여부. true면 프로세스 종료 코드는 [`WrCliError::exit_code`]에서 2.
    pub fn is_usage_error(&self) -> bool {
        matches!(
            self,
            WrCliError::UnknownFlag { .. }
                | WrCliError::UnknownSubcommand { .. }
                | WrCliError::MissingRequiredFlag(_)
                | WrCliError::MissingFlagValue(_)
                | WrCliError::InvalidFlagValue { .. }
                | WrCliError::ArgValidationFailed(_)
                | WrCliError::CommandHasNoRunner(_)
                | WrCliError::MutuallyExclusiveFlags { .. }
                | WrCliError::RequiredFlagsTogether { .. }
                | WrCliError::OneFlagRequired { .. }
        )
    }

    /// 프로세스 종료에 사용할 코드. 사용법 오류는 2, 그 외는 1.
    pub fn exit_code(&self) -> i32 {
        if self.is_usage_error() { 2 } else { 1 }
    }
}

impl fmt::Display for WrCliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WrCliError::UnknownFlag {
                flag,
                command,
                suggestions,
            } => {
                write!(
                    f,
                    "unknown flag '{}' for '{}'  Run with --help for usage.",
                    flag, command
                )?;
                write_suggestions(f, suggestions, "--")
            }
            WrCliError::UnknownSubcommand {
                name,
                parent,
                suggestions,
            } => {
                write!(
                    f,
                    "unknown command '{}' for '{}'  Run with --help for available commands.",
                    name, parent
                )?;
                write_suggestions(f, suggestions, "")
            }
            WrCliError::MissingRequiredFlag(name) => {
                write!(
                    f,
                    "required flag '--{}' not provided  Run with --help for usage.",
                    name
                )
            }
            WrCliError::MissingFlagValue(name) => {
                write!(
                    f,
                    "flag '--{}' requires a value  Run with --help for usage.",
                    name
                )
            }
            WrCliError::InvalidFlagValue {
                flag,
                expected,
                got,
            } => {
                write!(
                    f,
                    "invalid value '{}' for flag '--{}': expected {}",
                    got, flag, expected
                )
            }
            WrCliError::ArgValidationFailed(msg) => {
                write!(f, "{}", msg)
            }
            WrCliError::CommandHasNoRunner(name) => {
                write!(f, "command '{}' has no run handler", name)
            }
            WrCliError::ConfigFileNotFound { name, paths } => {
                write!(
                    f,
                    "config file '{}' not found in: {}",
                    name,
                    paths.join(", ")
                )
            }
            WrCliError::ConfigParseError { path, source } => {
                write!(f, "failed to parse config '{}': {}", path, source)
            }
            WrCliError::UnsupportedConfigFormat(ext) => {
                #[allow(unused_mut)]
                let mut supported: Vec<&str> = Vec::new();
                #[cfg(feature = "toml-config")]
                supported.push("toml");
                #[cfg(feature = "json-config")]
                supported.push("json");
                #[cfg(feature = "yaml-config")]
                supported.push("yaml");
                write!(
                    f,
                    "unsupported config format '{}' (supported: {})",
                    ext,
                    if supported.is_empty() {
                        "none enabled".to_owned()
                    } else {
                        supported.join(", ")
                    }
                )
            }
            WrCliError::UserError(e) => write!(f, "{}", e),
            WrCliError::Io(e) => write!(f, "io error: {}", e),
            WrCliError::UnsupportedCompletionShell(shell) => {
                write!(
                    f,
                    "unsupported completion shell '{}' (supported: bash, zsh, fish)",
                    shell
                )
            }
            WrCliError::ConfigTypeNotSet => {
                write!(
                    f,
                    "config type not set; call set_config_type() before read_config"
                )
            }
            WrCliError::ConfigFileExists(path) => {
                write!(f, "config file '{}' already exists", path)
            }
            WrCliError::ConfigDeserializeError(msg) => {
                write!(f, "failed to deserialize config: {}", msg)
            }
            WrCliError::ConfigWatchNotReady => {
                write!(
                    f,
                    "watch_config requires on_config_change() and a loaded config file"
                )
            }
            WrCliError::MutuallyExclusiveFlags { group, provided } => {
                write!(
                    f,
                    "flags {} are mutually exclusive (got {})",
                    format_flag_list(group),
                    format_flag_list(provided)
                )
            }
            WrCliError::RequiredFlagsTogether { group, missing } => {
                write!(
                    f,
                    "flags {} must be used together (missing {})",
                    format_flag_list(group),
                    format_flag_list(missing)
                )
            }
            WrCliError::OneFlagRequired { group } => {
                write!(f, "one of {} is required", format_flag_list(group))
            }
        }
    }
}

/// 플래그 이름 목록을 `--a, --b` 형태로 포맷.
fn format_flag_list(flags: &[String]) -> String {
    flags
        .iter()
        .map(|f| format!("--{}", f))
        .collect::<Vec<_>>()
        .join(", ")
}

/// 오타 제안 블록을 출력.
fn write_suggestions(
    f: &mut fmt::Formatter<'_>,
    suggestions: &[String],
    prefix: &str,
) -> fmt::Result {
    match suggestions {
        [] => Ok(()),
        [only] => write!(f, "\n\nDid you mean this?\n\t{}{}", prefix, only),
        many => {
            write!(f, "\n\nDid you mean one of these?")?;
            for s in many {
                write!(f, "\n\t{}{}", prefix, s)?;
            }
            Ok(())
        }
    }
}

impl std::error::Error for WrCliError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            WrCliError::UserError(e) => Some(e.as_ref()),
            WrCliError::Io(e) => Some(e),
            _ => None,
        }
    }
}

// ── From impls ────────────────────────────────────────────────────────────────

impl From<std::io::Error> for WrCliError {
    fn from(e: std::io::Error) -> Self {
        WrCliError::Io(e)
    }
}

#[cfg(feature = "serde")]
impl serde::de::Error for WrCliError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        WrCliError::ConfigDeserializeError(msg.to_string())
    }
}

#[cfg(feature = "yaml-config")]
impl From<noyalib::Error> for WrCliError {
    fn from(e: noyalib::Error) -> Self {
        WrCliError::ConfigParseError {
            path: String::new(),
            source: e.to_string(),
        }
    }
}

pub type Result<T> = std::result::Result<T, WrCliError>;

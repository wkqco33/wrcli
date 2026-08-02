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
    },
    UnknownSubcommand {
        name: String,
        parent: String,
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
}

impl WrCliError {
    /// Convenience constructor to wrap any error as a `UserError`.
    pub fn user<E: std::error::Error + Send + Sync + 'static>(e: E) -> Self {
        WrCliError::UserError(Box::new(e))
    }
}

impl fmt::Display for WrCliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WrCliError::UnknownFlag { flag, command } => {
                write!(
                    f,
                    "unknown flag '{}' for '{}'  Run with --help for usage.",
                    flag, command
                )
            }
            WrCliError::UnknownSubcommand { name, parent } => {
                write!(
                    f,
                    "unknown command '{}' for '{}'  Run with --help for available commands.",
                    name, parent
                )
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

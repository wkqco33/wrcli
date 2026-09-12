use std::fmt;

/// `non_exhaustive`: prevents future variant additions from being semver-breaking.
/// Consumers must include a wildcard (`_`) arm in `match`.
#[derive(Debug)]
#[non_exhaustive]
pub enum WrCliError {
    // Command routing errors
    UnknownFlag {
        flag: String,
        command: String,
        /// Edit-distance-based close candidates (may be empty).
        suggestions: Vec<String>,
    },
    UnknownSubcommand {
        name: String,
        parent: String,
        /// Edit-distance-based close candidates (may be empty).
        suggestions: Vec<String>,
    },
    MissingRequiredFlag(String),
    /// A flag was provided but its value is missing (e.g. `--name` without value).
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

    /// No config format was specified when `read_config` was called.
    ConfigTypeNotSet,

    /// The target file for `safe_write_config_as` already exists.
    ConfigFileExists(String),

    /// serde deserialization failed during `unmarshal`.
    ConfigDeserializeError(String),

    /// `watch_config` preconditions (callback/loaded file) are not satisfied.
    ConfigWatchNotReady,

    /// Two or more flags in a `mutually_exclusive` group were given at once.
    MutuallyExclusiveFlags {
        group: Vec<String>,
        provided: Vec<String>,
    },

    /// Only some flags in a `required_together` group were given.
    RequiredFlagsTogether {
        group: Vec<String>,
        missing: Vec<String>,
    },

    /// No flag in the `one_required` group was given.
    OneFlagRequired {
        group: Vec<String>,
    },

    /// Interactive input is required, but stdin is not a TTY or `--no-input` was given.
    InteractiveInputRequired {
        /// Hint for the flag/approach to use instead.
        hint: String,
    },

    /// Confirmation text did not match in `confirm_severe`.
    ConfirmationFailed {
        /// The required confirmation text.
        expected: String,
    },
}

impl WrCliError {
    /// Convenience constructor to wrap any error as a `UserError`.
    pub fn user<E: std::error::Error + Send + Sync + 'static>(e: E) -> Self {
        WrCliError::UserError(Box::new(e))
    }

    /// Whether this is a usage error. If true, the process exit code is 2 from [`WrCliError::exit_code`].
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
                | WrCliError::InteractiveInputRequired { .. }
                | WrCliError::ConfirmationFailed { .. }
        )
    }

    /// Exit code to use for the process. Usage errors are 2, others are 1.
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
                    "invalid value '{}' for flag '--{}': expected {}  Run with --help for usage.",
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
            WrCliError::InteractiveInputRequired { hint } => {
                write!(
                    f,
                    "input required, but stdin is not an interactive terminal (or --no-input was passed); instead pass {}",
                    hint
                )
            }
            WrCliError::ConfirmationFailed { expected } => {
                write!(f, "confirmation failed: expected \"{}\"", expected)
            }
        }
    }
}

/// Formats a list of flag names as `--a, --b`.
fn format_flag_list(flags: &[String]) -> String {
    flags
        .iter()
        .map(|f| format!("--{}", f))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Writes the typo suggestion block.
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

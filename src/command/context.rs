use crate::config::{Config, ConfigValue, SettingsMap};
use crate::error::{Result, WrCliError};
use crate::flag::FlagSet;
use std::io::{BufRead, Write};
use std::time::{Duration, SystemTime};

/// Output format. Corresponds to the `--plain` / `--json` standard flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    /// Default human-readable output.
    #[default]
    Human,
    /// Machine-friendly output with one record per line (for `grep`/`awk`).
    Plain,
    /// JSON output.
    Json,
}

/// Passed by reference to every `on_run` / `on_run_e` callback.
///
/// Provides access to:
/// - Parsed flag values (local + inherited persistent flags from ancestor commands)
/// - Positional arguments remaining after flag parsing
/// - The configuration store ([`Config`])
/// - The full command path that was invoked
pub struct CommandContext<'a> {
    /// Path of command names from root to the matched leaf, e.g. `["myapp", "config", "get"]`.
    pub command_path: Vec<String>,
    /// Positional arguments remaining after flags were consumed.
    pub args: Vec<String>,
    /// Merged flags: local flags + all inherited persistent flags.
    pub flags: &'a FlagSet,
    /// Configuration store (viper equivalent).
    pub config: &'a Config,
}

impl<'a> CommandContext<'a> {
    /// Get a string value: checks flags first, then config.
    pub fn get_string(&self, key: &str) -> Option<String> {
        self.flags
            .get_string(key)
            .map(str::to_owned)
            .or_else(|| self.config.get_string(key))
    }

    /// Get an integer value: checks flags first, then config.
    pub fn get_int(&self, key: &str) -> Option<i64> {
        self.flags.get_int(key).or_else(|| self.config.get_int(key))
    }

    /// Get an unsigned integer value: checks flags first, then config.
    pub fn get_uint(&self, key: &str) -> Option<u64> {
        self.flags
            .get_uint(key)
            .or_else(|| self.config.get_uint(key))
    }

    /// Get a boolean value: checks flags first, then config.
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.flags
            .get_bool(key)
            .or_else(|| self.config.get_bool(key))
    }

    /// Get a float value: checks flags first, then config.
    pub fn get_float(&self, key: &str) -> Option<f64> {
        self.flags
            .get_float(key)
            .or_else(|| self.config.get_float(key))
    }

    /// Get a string-vec value: checks flags first, then config.
    pub fn get_string_vec(&self, key: &str) -> Option<Vec<String>> {
        self.flags
            .get_string_vec(key)
            .map(|s| s.to_vec())
            .or_else(|| self.config.get_string_vec(key))
    }

    /// Get an int-vec value: checks flags first, then config.
    pub fn get_int_vec(&self, key: &str) -> Option<Vec<i64>> {
        self.flags.get_int_vec(key).map(|s| s.to_vec()).or_else(|| {
            self.config.get(key).and_then(|v| {
                v.as_array()
                    .map(|arr| arr.iter().filter_map(|x| x.as_int()).collect())
            })
        })
    }

    /// Get a [`Duration`] value: checks flags first, then config.
    pub fn get_duration(&self, key: &str) -> Option<Duration> {
        self.flag_value(key)
            .and_then(|v| v.to_duration_coerce())
            .or_else(|| self.config.get_duration(key))
    }

    /// Get a [`SystemTime`] value: checks flags first, then config.
    pub fn get_time(&self, key: &str) -> Option<SystemTime> {
        self.flag_value(key)
            .and_then(|v| v.to_time_coerce())
            .or_else(|| self.config.get_time(key))
    }

    /// Get a byte-size (`u64`) value: checks flags first, then config.
    pub fn get_size_in_bytes(&self, key: &str) -> Option<u64> {
        self.flag_value(key)
            .and_then(|v| v.to_size_in_bytes_coerce())
            .or_else(|| self.config.get_size_in_bytes(key))
    }

    /// Get a nested settings map from config (flags have no map type).
    pub fn get_string_map(&self, key: &str) -> Option<SettingsMap> {
        self.config.get_string_map(key)
    }

    /// Whether the key was explicitly provided as a flag or is present in config.
    pub fn is_set(&self, key: &str) -> bool {
        self.flags.is_set(key) || self.config.is_set(key)
    }

    /// The leaf command name (last element of `command_path`).
    pub fn command_name(&self) -> &str {
        self.command_path.last().map(String::as_str).unwrap_or("")
    }

    /// Whether `-q` / `--quiet` was specified.
    pub fn is_quiet(&self) -> bool {
        self.flags.get_bool("quiet").unwrap_or(false)
    }

    /// Whether `-f` / `--force` was specified (skips confirmation prompts).
    pub fn is_force(&self) -> bool {
        self.flags.get_bool("force").unwrap_or(false)
    }

    /// Whether `--no-input` was specified (forbids interactive input).
    pub fn no_input(&self) -> bool {
        self.flags.get_bool("no-input").unwrap_or(false)
    }

    /// Whether `--plain` was specified.
    pub fn is_plain(&self) -> bool {
        self.flags.get_bool("plain").unwrap_or(false)
    }

    /// Whether `--json` was specified.
    pub fn is_json(&self) -> bool {
        self.flags.get_bool("json").unwrap_or(false)
    }

    /// Output format derived from `--plain` / `--json`.
    pub fn output_format(&self) -> OutputFormat {
        if self.is_json() {
            OutputFormat::Json
        } else if self.is_plain() {
            OutputFormat::Plain
        } else {
            OutputFormat::Human
        }
    }

    /// Whether interactive prompts can be used (a TTY and not `--no-input`).
    pub fn is_interactive(&self) -> bool {
        !self.no_input() && crate::style::stdin_is_terminal()
    }

    /// Ask the user for confirmation (clig.dev: Confirm before doing anything dangerous).
    ///
    /// - If `--force` is specified, `true` without prompting.
    /// - If `--no-input` is set or stdin is not a TTY, return an error suggesting an alternative flag.
    pub fn confirm(&self, message: &str) -> Result<bool> {
        if self.is_force() {
            return Ok(true);
        }
        self.ensure_interactive("--force")?;
        ask_yes_no(message)
    }

    /// Confirm a severe (dangerous) action. `--confirm="<expected>"` or direct input must match.
    pub fn confirm_severe(&self, expected: &str) -> Result<bool> {
        let provided = self.flags.get_string("confirm").unwrap_or("");
        if !provided.is_empty() {
            return if provided == expected {
                Ok(true)
            } else {
                Err(WrCliError::ConfirmationFailed {
                    expected: expected.to_owned(),
                })
            };
        }
        self.ensure_interactive("--confirm=\"<name>\"")?;
        eprint!("Type '{}' to confirm: ", expected);
        let _ = std::io::stderr().flush();
        let line = read_line()?;
        if line.trim() == expected {
            Ok(true)
        } else {
            Err(WrCliError::ConfirmationFailed {
                expected: expected.to_owned(),
            })
        }
    }

    /// Read a password without echo (clig.dev: don't print it as the user types).
    ///
    /// On unix it is hidden via `stty -echo`. On other platforms echo cannot be turned off.
    pub fn prompt_password(&self, message: &str) -> Result<String> {
        self.ensure_interactive("a credentials file or stdin")?;
        eprint!("{}", message);
        let _ = std::io::stderr().flush();
        #[cfg(unix)]
        set_terminal_echo(false);
        let result = read_line();
        #[cfg(unix)]
        set_terminal_echo(true);
        eprintln!();
        result
    }

    /// If stdin is not a TTY or `--no-input` is set, return an error with guidance.
    fn ensure_interactive(&self, hint: &str) -> Result<()> {
        if self.no_input() || !crate::style::stdin_is_terminal() {
            return Err(WrCliError::InteractiveInputRequired {
                hint: hint.to_owned(),
            });
        }
        Ok(())
    }

    /// Convert the flag value to [`ConfigValue`] (`None` if absent).
    fn flag_value(&self, key: &str) -> Option<ConfigValue> {
        self.flags.get(key).map(ConfigValue::from)
    }
}

/// Print a y/N prompt to stderr, read one line from stdin, and return whether it was yes.
fn ask_yes_no(message: &str) -> Result<bool> {
    eprint!("{} [y/N] ", message);
    let _ = std::io::stderr().flush();
    let line = read_line()?;
    Ok(matches!(
        line.trim().to_ascii_lowercase().as_str(),
        "y" | "yes"
    ))
}

/// Read a single line from stdin.
fn read_line() -> Result<String> {
    let mut line = String::new();
    std::io::stdin().lock().read_line(&mut line)?;
    Ok(line)
}

/// Enable or disable terminal echo (unix).
#[cfg(unix)]
fn set_terminal_echo(enable: bool) {
    let arg = if enable { "echo" } else { "-echo" };
    let _ = std::process::Command::new("stty").arg(arg).status();
}

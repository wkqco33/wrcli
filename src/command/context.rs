use crate::config::{Config, ConfigValue, SettingsMap};
use crate::error::{Result, WrCliError};
use crate::flag::FlagSet;
use std::io::{BufRead, Write};
use std::time::{Duration, SystemTime};

/// 출력 포맷. `--plain` / `--json` 표준 플래그에 대응한다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    /// 사람이 읽기 좋은 기본 출력.
    #[default]
    Human,
    /// 한 줄에 레코드 하나인 기계 친화적 출력 (`grep`/`awk`용).
    Plain,
    /// JSON 출력.
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

    /// `-q` / `--quiet` 지정 여부.
    pub fn is_quiet(&self) -> bool {
        self.flags.get_bool("quiet").unwrap_or(false)
    }

    /// `-f` / `--force` 지정 여부 (확인 프롬프트 생략).
    pub fn is_force(&self) -> bool {
        self.flags.get_bool("force").unwrap_or(false)
    }

    /// `--no-input` 지정 여부 (대화형 입력 금지).
    pub fn no_input(&self) -> bool {
        self.flags.get_bool("no-input").unwrap_or(false)
    }

    /// `--plain` 지정 여부.
    pub fn is_plain(&self) -> bool {
        self.flags.get_bool("plain").unwrap_or(false)
    }

    /// `--json` 지정 여부.
    pub fn is_json(&self) -> bool {
        self.flags.get_bool("json").unwrap_or(false)
    }

    /// `--plain` / `--json`에 따른 출력 포맷.
    pub fn output_format(&self) -> OutputFormat {
        if self.is_json() {
            OutputFormat::Json
        } else if self.is_plain() {
            OutputFormat::Plain
        } else {
            OutputFormat::Human
        }
    }

    /// 대화형 프롬프트를 사용할 수 있는지 여부 (TTY이고 `--no-input`이 아님).
    pub fn is_interactive(&self) -> bool {
        !self.no_input() && crate::style::stdin_is_terminal()
    }

    /// 사용자에게 확인을 요청한다 (clig.dev: Confirm before doing anything dangerous).
    ///
    /// - `--force`가 지정되면 프롬프트 없이 `true`.
    /// - `--no-input`이거나 stdin이 TTY가 아니면 대신 쓸 플래그를 알려주며 오류를 낸다.
    pub fn confirm(&self, message: &str) -> Result<bool> {
        if self.is_force() {
            return Ok(true);
        }
        self.ensure_interactive("--force")?;
        ask_yes_no(message)
    }

    /// 위험한(severe) 작업 확인. `--confirm="<expected>"` 또는 직접 입력이 일치해야 한다.
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

    /// 비밀번호를 echo 없이 입력받는다 (clig.dev: don't print it as the user types).
    ///
    /// unix에서는 `stty -echo`로 가린다. 그 외 플랫폼에서는 echo를 끌 수 없다.
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

    /// stdin이 TTY가 아니거나 `--no-input`이면 안내와 함께 오류를 반환한다.
    fn ensure_interactive(&self, hint: &str) -> Result<()> {
        if self.no_input() || !crate::style::stdin_is_terminal() {
            return Err(WrCliError::InteractiveInputRequired {
                hint: hint.to_owned(),
            });
        }
        Ok(())
    }

    /// 플래그 값을 [`ConfigValue`]로 변환 (없으면 `None`).
    fn flag_value(&self, key: &str) -> Option<ConfigValue> {
        self.flags.get(key).map(ConfigValue::from)
    }
}

/// stderr에 y/N 프롬프트를 출력하고 stdin 한 줄을 읽어 yes 여부를 반환.
fn ask_yes_no(message: &str) -> Result<bool> {
    eprint!("{} [y/N] ", message);
    let _ = std::io::stderr().flush();
    let line = read_line()?;
    Ok(matches!(
        line.trim().to_ascii_lowercase().as_str(),
        "y" | "yes"
    ))
}

/// stdin에서 한 줄을 읽는다.
fn read_line() -> Result<String> {
    let mut line = String::new();
    std::io::stdin().lock().read_line(&mut line)?;
    Ok(line)
}

/// 터미널 echo를 켜고 끈다 (unix).
#[cfg(unix)]
fn set_terminal_echo(enable: bool) {
    let arg = if enable { "echo" } else { "-echo" };
    let _ = std::process::Command::new("stty").arg(arg).status();
}

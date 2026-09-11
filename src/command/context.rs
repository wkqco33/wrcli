use crate::config::{Config, ConfigValue, SettingsMap};
use crate::flag::FlagSet;
use std::time::{Duration, SystemTime};

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

    /// 플래그 값을 [`ConfigValue`]로 변환 (없으면 `None`).
    fn flag_value(&self, key: &str) -> Option<ConfigValue> {
        self.flags.get(key).map(ConfigValue::from)
    }
}

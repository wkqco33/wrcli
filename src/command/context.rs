use crate::config::Config;
use crate::flag::FlagSet;

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

    /// The leaf command name (last element of `command_path`).
    pub fn command_name(&self) -> &str {
        self.command_path.last().map(String::as_str).unwrap_or("")
    }
}

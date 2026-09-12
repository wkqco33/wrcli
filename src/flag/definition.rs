use super::value::FlagValue;

/// A single flag definition (both local and persistent flags use this type).
#[derive(Debug, Clone)]
pub struct Flag {
    pub name: String,
    pub short: Option<char>,
    pub usage: String,
    pub default: FlagValue,
    pub required: bool,
    pub persistent: bool,
    /// Hidden from help/completion (parsing still works).
    pub hidden: bool,
    /// Warning message shown when set. Printed to stderr when parsed.
    pub deprecated: Option<String>,
    /// Whether this is a persistent flag inherited from a parent command (for separating help sections).
    pub inherited: bool,
    /// Whether to split `StringVec`/`IntVec` values on commas (opt-in).
    pub comma_separated: bool,
    /// Whether this is an optional-value flag (`none` = no value).
    pub optional_value: bool,
    /// Whether the value is sensitive — redacted in help defaults and error messages.
    pub sensitive: bool,
}

impl Flag {
    pub fn new(name: &str, default: FlagValue, usage: &str) -> Self {
        Flag {
            name: name.to_owned(),
            short: None,
            usage: usage.to_owned(),
            default,
            required: false,
            persistent: false,
            hidden: false,
            deprecated: None,
            inherited: false,
            comma_separated: false,
            optional_value: false,
            sensitive: false,
        }
    }

    pub fn short(mut self, c: char) -> Self {
        self.short = Some(c);
        self
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    pub fn persistent(mut self) -> Self {
        self.persistent = true;
        self
    }

    /// Hide this flag from help and completion.
    pub fn hidden(mut self) -> Self {
        self.hidden = true;
        self
    }

    /// Mark the flag as deprecated. Prints a warning to stderr when used.
    pub fn deprecated(mut self, msg: &str) -> Self {
        self.deprecated = Some(msg.to_owned());
        self
    }

    /// Split `--tag a,b,c` into multiple values for `StringVec`/`IntVec` (not split by default).
    pub fn comma_separated(mut self) -> Self {
        self.comma_separated = true;
        self
    }

    /// Mark the flag as taking an optional value.
    ///
    /// In this case the special word `none` (case-insensitive) means "no value" (empty string).
    /// clig.dev: "If a flag can accept an optional value, allow a special word like 'none'."
    pub fn optional_value(mut self) -> Self {
        self.optional_value = true;
        self
    }

    /// Mark the flag as taking a sensitive value (password, token, etc.).
    ///
    /// The value is redacted to `***` in the help default display and error messages.
    /// clig.dev: prefer taking secrets from a file/stdin instead of a flag whenever possible.
    pub fn sensitive(mut self) -> Self {
        self.sensitive = true;
        self
    }
}

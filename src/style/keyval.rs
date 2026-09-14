use super::{Style, display_width, stdout_is_styled};

/// Renders a list of aligned key-value pairs.
///
/// Useful for displaying command statuses, configuration properties, or metadata cleanly.
///
/// # Example
///
/// ```
/// use wrcli::style::KeyVal;
///
/// let out = KeyVal::new()
///     .entry("Host", "127.0.0.1")
///     .entry("Port", "8080")
///     .render(false);
///
/// assert!(out.contains("Host : 127.0.0.1"));
/// assert!(out.contains("Port : 8080"));
/// ```
pub struct KeyVal {
    entries: Vec<(String, String)>,
    separator: String,
    key_style: Style,
    val_style: Style,
    sep_style: Style,
}

impl KeyVal {
    pub fn new() -> Self {
        KeyVal {
            entries: Vec::new(),
            separator: " : ".to_owned(),
            key_style: Style::new().bold(),
            val_style: Style::new(),
            sep_style: Style::new(),
        }
    }

    /// Appends a key-value entry.
    pub fn entry<K: Into<String>, V: Into<String>>(mut self, key: K, val: V) -> Self {
        self.entries.push((key.into(), val.into()));
        self
    }

    /// Sets the separator string between key and value (default `" : "`).
    pub fn separator(mut self, s: &str) -> Self {
        self.separator = s.to_owned();
        self
    }

    /// Sets the style for keys.
    pub fn key_style(mut self, s: Style) -> Self {
        self.key_style = s;
        self
    }

    /// Sets the style for values.
    pub fn val_style(mut self, s: Style) -> Self {
        self.val_style = s;
        self
    }

    /// Sets the style for the separator.
    pub fn sep_style(mut self, s: Style) -> Self {
        self.sep_style = s;
        self
    }

    /// Prints to stdout, auto-detecting whether it is a TTY.
    pub fn print(&self) {
        print!("{}", self.render(stdout_is_styled()));
    }

    /// Renders the key-value list as a `String`.
    pub fn render(&self, styled: bool) -> String {
        let max_w = self
            .entries
            .iter()
            .map(|(k, _)| display_width(k))
            .max()
            .unwrap_or(0);

        let mut buf = String::new();
        for (k, v) in &self.entries {
            let k_w = display_width(k);
            let pad = max_w.saturating_sub(k_w);
            let padded_k = format!("{}{}", k, " ".repeat(pad));
            buf.push_str(&self.key_style.apply(&padded_k, styled));
            buf.push_str(&self.sep_style.apply(&self.separator, styled));
            buf.push_str(&self.val_style.apply(v, styled));
            buf.push('\n');
        }
        buf
    }
}

impl Default for KeyVal {
    fn default() -> Self {
        Self::new()
    }
}

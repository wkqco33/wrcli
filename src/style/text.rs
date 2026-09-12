use super::{Style, stdout_is_styled};

/// Renders several spans with different styles concatenated into a single text.
///
/// Inspired by the rich library's `Text`. Applies an individual style to each span
/// and wraps it in ANSI escape codes to merge them.
///
/// # Example
///
/// ```
/// use wrcli::style::{Text, Style, Color};
///
/// let text = Text::new()
///     .plain("Error: ")
///     .span("boom", Style::new().fg(Color::Red).bold());
///
/// let out = text.render(false);
/// assert_eq!(out, "Error: boom");
/// ```
#[derive(Debug, Default)]
pub struct Text {
    spans: Vec<(String, Style)>,
}

impl Text {
    pub fn new() -> Self {
        Default::default()
    }

    /// Appends text with the default (no) style.
    pub fn plain(mut self, s: &str) -> Self {
        self.spans.push((s.to_owned(), Style::new()));
        self
    }

    /// Appends text with the given style.
    pub fn span(mut self, s: &str, style: Style) -> Self {
        self.spans.push((s.to_owned(), style));
        self
    }

    /// Appends text with an explicit style. (The styled version of `plain`.)
    pub fn plain_styled(mut self, s: &str, style: Style) -> Self {
        self.spans.push((s.to_owned(), style));
        self
    }

    /// Prints, auto-detecting whether stdout is a TTY.
    pub fn print(&self) {
        print!("{}", self.render(stdout_is_styled()));
    }

    /// Renders the text as a `String`.
    ///
    /// When `styled = true`, includes ANSI escape sequences.
    pub fn render(&self, styled: bool) -> String {
        let mut buf = String::new();
        for (text, style) in &self.spans {
            buf.push_str(&style.apply(text, styled));
        }
        buf
    }
}

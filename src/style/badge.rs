use super::{Color, Style, stdout_is_styled};

/// A compact status badge or tag.
///
/// # Example
///
/// ```
/// use wrcli::style::Badge;
///
/// let b = Badge::success("DONE");
/// assert_eq!(b.render(false), "[DONE]");
/// ```
#[derive(Debug, Clone)]
pub struct Badge {
    text: String,
    style: Style,
    bracket_style: Style,
    brackets: Option<(char, char)>,
}

impl Badge {
    pub fn new(text: &str) -> Self {
        Badge {
            text: text.to_owned(),
            style: Style::new().bold(),
            bracket_style: Style::new().fg(Color::BrightBlack),
            brackets: Some(('[', ']')),
        }
    }

    /// Green success badge: `[SUCCESS]`
    pub fn success(text: &str) -> Self {
        Self::new(text).style(Style::new().fg(Color::Green).bold())
    }

    /// Red error badge: `[ERROR]`
    pub fn error(text: &str) -> Self {
        Self::new(text).style(Style::new().fg(Color::Red).bold())
    }

    /// Yellow warning badge: `[WARN]`
    pub fn warn(text: &str) -> Self {
        Self::new(text).style(Style::new().fg(Color::Yellow).bold())
    }

    /// Cyan info badge: `[INFO]`
    pub fn info(text: &str) -> Self {
        Self::new(text).style(Style::new().fg(Color::Cyan).bold())
    }

    /// Sets the text style.
    pub fn style(mut self, s: Style) -> Self {
        self.style = s;
        self
    }

    /// Sets the bracket style.
    pub fn bracket_style(mut self, s: Style) -> Self {
        self.bracket_style = s;
        self
    }

    /// Sets custom opening and closing bracket characters.
    pub fn brackets(mut self, open: char, close: char) -> Self {
        self.brackets = Some((open, close));
        self
    }

    /// Removes brackets and adds single space padding on both sides (pill style).
    pub fn no_brackets(mut self) -> Self {
        self.brackets = None;
        self
    }

    /// Prints the badge to stdout, auto-detecting TTY.
    pub fn print(&self) {
        print!("{}", self.render(stdout_is_styled()));
    }

    /// Renders the badge as a `String`.
    pub fn render(&self, styled: bool) -> String {
        if let Some((open, close)) = self.brackets {
            let open_str = self.bracket_style.apply(&open.to_string(), styled);
            let text_str = self.style.apply(&self.text, styled);
            let close_str = self.bracket_style.apply(&close.to_string(), styled);
            format!("{}{}{}", open_str, text_str, close_str)
        } else {
            let padded = format!(" {} ", self.text);
            self.style.apply(&padded, styled)
        }
    }
}

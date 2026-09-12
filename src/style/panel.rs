use super::{Color, Style, display_width, stdout_is_styled};

/// A panel that draws borders with Unicode box-drawing characters (with an optional title).
///
/// # Example
///
/// ```
/// use wrcli::style::{Panel, Style, Color};
///
/// let rendered = Panel::new("Deploy complete.\nAll services healthy.")
///     .title("Status")
///     .border_style(Style::new().fg(Color::Green))
///     .render(false);
///
/// assert!(rendered.contains("Status"));
/// assert!(rendered.contains("Deploy complete."));
/// ```
pub struct Panel {
    content: String,
    title: Option<String>,
    border_style: Style,
    title_style: Style,
    padding: usize,
    width: Option<usize>,
}

impl Panel {
    pub fn new(content: &str) -> Self {
        Panel {
            content: content.to_owned(),
            title: None,
            border_style: Style::new().fg(Color::Cyan),
            title_style: Style::new().bold(),
            padding: 1,
            width: None,
        }
    }

    pub fn title(mut self, t: &str) -> Self {
        self.title = Some(t.to_owned());
        self
    }

    pub fn border_style(mut self, s: Style) -> Self {
        self.border_style = s;
        self
    }

    pub fn title_style(mut self, s: Style) -> Self {
        self.title_style = s;
        self
    }

    pub fn padding(mut self, p: usize) -> Self {
        self.padding = p;
        self
    }

    pub fn width(mut self, w: usize) -> Self {
        self.width = Some(w);
        self
    }

    /// Prints, auto-detecting whether stdout is a TTY.
    pub fn print(&self) {
        print!("{}", self.render(stdout_is_styled()));
    }

    /// Renders the panel as a `String`.
    ///
    /// When `styled = true`, includes ANSI escape sequences.
    pub fn render(&self, styled: bool) -> String {
        // Compute widths in character count (str::len() is byte length and breaks alignment for non-ASCII).
        let lines: Vec<&str> = self.content.lines().collect();
        let content_width = lines.iter().map(|l| display_width(l)).max().unwrap_or(0);
        let title_min = self
            .title
            .as_deref()
            .map(|t| display_width(t) + 2 + 4)
            .unwrap_or(0);
        let inner_width = self
            .width
            .unwrap_or_else(|| content_width.max(title_min).max(20))
            .max(content_width)
            .max(title_min);

        let mut buf = String::new();
        let pad = self.padding;

        let b = |s: &str| self.border_style.apply(s, styled);
        let t = |s: &str| self.title_style.apply(s, styled);

        if let Some(ref title) = self.title {
            let title_part = format!(" {} ", title);
            let dashes_needed = inner_width + pad * 2;
            let left_dashes = pad + 1;
            let right_dashes =
                dashes_needed.saturating_sub(left_dashes + display_width(&title_part));
            buf.push_str(&format!(
                "{}{}{}{}{}",
                b("╭"),
                b(&"─".repeat(left_dashes)),
                t(&title_part),
                b(&"─".repeat(right_dashes)),
                b("╮"),
            ));
        } else {
            buf.push_str(&format!(
                "{}{}{}",
                b("╭"),
                b(&"─".repeat(inner_width + pad * 2)),
                b("╮"),
            ));
        }
        buf.push('\n');

        let padding_str = " ".repeat(pad);
        for line in &lines {
            let right_fill = inner_width.saturating_sub(display_width(line));
            buf.push_str(&format!(
                "{}{}{}{}{}",
                b("│"),
                padding_str,
                line,
                " ".repeat(right_fill + pad),
                b("│"),
            ));
            buf.push('\n');
        }

        buf.push_str(&format!(
            "{}{}{}",
            b("╰"),
            b(&"─".repeat(inner_width + pad * 2)),
            b("╯"),
        ));
        buf.push('\n');

        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_contains_content() {
        let out = Panel::new("Hello, World!").render(false);
        assert!(out.contains("Hello, World!"));
    }

    #[test]
    fn render_title() {
        let out = Panel::new("body").title("My Panel").render(false);
        assert!(out.contains("My Panel"));
        assert!(out.contains("body"));
    }

    #[test]
    fn render_borders() {
        let out = Panel::new("content").render(false);
        assert!(out.contains('╭'));
        assert!(out.contains('╯'));
        assert!(out.contains('│'));
    }

    #[test]
    fn multiline_content() {
        let out = Panel::new("line one\nline two\nline three").render(false);
        assert!(out.contains("line one"));
        assert!(out.contains("line two"));
        assert!(out.contains("line three"));
    }

    #[test]
    fn fixed_width() {
        let out = Panel::new("hi").width(40).render(false);
        assert!(out.contains("hi"));
    }

    #[test]
    fn title_border_matches_content_width_with_padding() {
        let out = Panel::new("body").title("T").padding(2).render(false);
        let widths: Vec<usize> = out.lines().map(display_width).collect();
        assert!(
            widths.windows(2).all(|w| w[0] == w[1]),
            "misaligned lines: {widths:?}"
        );
    }

    #[test]
    fn non_ascii_lines_stay_aligned() {
        // With display_width, the right border (│) of every line must align.
        let out = Panel::new("한글 콘텐츠\nshort").render(false);
        let right_border_col: Vec<usize> = out
            .lines()
            .filter(|l| l.starts_with('│'))
            .map(|l| crate::style::display_width(l) - 1)
            .collect();
        assert!(right_border_col.windows(2).all(|w| w[0] == w[1]));
    }
}

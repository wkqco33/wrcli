use super::{Align, BoxStyle, Color, Style, display_width, stdout_is_styled};

/// A panel that draws borders with Unicode box-drawing characters (with an optional title and subtitle).
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
    subtitle: Option<String>,
    border_style: Style,
    title_style: Style,
    subtitle_style: Style,
    padding: usize,
    width: Option<usize>,
    box_style: BoxStyle,
    content_align: Align,
    subtitle_align: Align,
}

impl Panel {
    pub fn new(content: &str) -> Self {
        Panel {
            content: content.to_owned(),
            title: None,
            subtitle: None,
            border_style: Style::new().fg(Color::Cyan),
            title_style: Style::new().bold(),
            subtitle_style: Style::new().bold(),
            padding: 1,
            width: None,
            box_style: BoxStyle::Rounded,
            content_align: Align::Left,
            subtitle_align: Align::Right,
        }
    }

    pub fn title(mut self, t: &str) -> Self {
        self.title = Some(t.to_owned());
        self
    }

    pub fn subtitle(mut self, s: &str) -> Self {
        self.subtitle = Some(s.to_owned());
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

    pub fn subtitle_style(mut self, s: Style) -> Self {
        self.subtitle_style = s;
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

    pub fn box_style(mut self, bs: BoxStyle) -> Self {
        self.box_style = bs;
        self
    }

    pub fn content_align(mut self, a: Align) -> Self {
        self.content_align = a;
        self
    }

    pub fn subtitle_align(mut self, a: Align) -> Self {
        self.subtitle_align = a;
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
        let subtitle_min = self
            .subtitle
            .as_deref()
            .map(|s| display_width(s) + 2 + 4)
            .unwrap_or(0);
        let inner_width = self
            .width
            .unwrap_or_else(|| content_width.max(title_min).max(subtitle_min).max(20))
            .max(content_width)
            .max(title_min)
            .max(subtitle_min);

        let mut buf = String::new();
        let pad = self.padding;
        let bs = self.box_style;

        let b = |s: &str| self.border_style.apply(s, styled);
        let t = |s: &str| self.title_style.apply(s, styled);
        let sub = |s: &str| self.subtitle_style.apply(s, styled);

        let h = bs.horizontal().to_string();
        let v = bs.vertical().to_string();

        if let Some(ref title) = self.title {
            let title_part = format!(" {} ", title);
            let dashes_needed = inner_width + pad * 2;
            let left_dashes = pad + 1;
            let right_dashes =
                dashes_needed.saturating_sub(left_dashes + display_width(&title_part));
            buf.push_str(&format!(
                "{}{}{}{}{}",
                b(&bs.top_left().to_string()),
                b(&h.repeat(left_dashes)),
                t(&title_part),
                b(&h.repeat(right_dashes)),
                b(&bs.top_right().to_string()),
            ));
        } else {
            buf.push_str(&format!(
                "{}{}{}",
                b(&bs.top_left().to_string()),
                b(&h.repeat(inner_width + pad * 2)),
                b(&bs.top_right().to_string()),
            ));
        }
        buf.push('\n');

        let padding_str = " ".repeat(pad);
        for line in &lines {
            let right_fill = inner_width.saturating_sub(display_width(line));
            let (pad_l, pad_r) = match self.content_align {
                Align::Left => (padding_str.clone(), " ".repeat(right_fill + pad)),
                Align::Right => (" ".repeat(right_fill + pad), padding_str.clone()),
                Align::Center => {
                    let l = right_fill / 2;
                    let r = right_fill - l;
                    (" ".repeat(l + pad), " ".repeat(r + pad))
                }
            };
            buf.push_str(&format!("{}{}{}{}{}", b(&v), pad_l, line, pad_r, b(&v),));
            buf.push('\n');
        }

        if let Some(ref subtitle) = self.subtitle {
            let sub_part = format!(" {} ", subtitle);
            let dashes_needed = inner_width + pad * 2;
            let sub_w = display_width(&sub_part);
            let remaining = dashes_needed.saturating_sub(sub_w);
            let (left_dashes, right_dashes) = match self.subtitle_align {
                Align::Right => {
                    let r = pad + 1;
                    let l = remaining.saturating_sub(r);
                    (l, r)
                }
                Align::Left => {
                    let l = pad + 1;
                    let r = remaining.saturating_sub(l);
                    (l, r)
                }
                Align::Center => {
                    let l = remaining / 2;
                    let r = remaining - l;
                    (l, r)
                }
            };
            buf.push_str(&format!(
                "{}{}{}{}{}",
                b(&bs.bottom_left().to_string()),
                b(&h.repeat(left_dashes)),
                sub(&sub_part),
                b(&h.repeat(right_dashes)),
                b(&bs.bottom_right().to_string()),
            ));
        } else {
            buf.push_str(&format!(
                "{}{}{}",
                b(&bs.bottom_left().to_string()),
                b(&h.repeat(inner_width + pad * 2)),
                b(&bs.bottom_right().to_string()),
            ));
        }
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

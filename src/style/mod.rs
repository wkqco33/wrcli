//! Rich terminal styling — colors, text attributes, tables, panels, and rules.
//!
//! Inspired by Python's [rich](https://github.com/Textualize/rich) library,
//! this module provides utilities for producing beautifully formatted terminal
//! output with ANSI color codes.
//!
//! Styling is **automatically disabled** when:
//! - The `NO_COLOR` environment variable is set (see <https://no-color.org/>)
//! - The output stream is not connected to a terminal (e.g., piped to a file)
//!
//! # Quick start
//!
//! ```
//! use wrcli::style::{Color, Style, Panel, Table, Rule, Align};
//!
//! // Styled text
//! let s = Style::new().fg(Color::Green).bold();
//! let text = s.apply("Hello, World!", false); // false = no ANSI (non-TTY)
//! assert_eq!(text, "Hello, World!");
//!
//! // Table
//! let table = Table::new()
//!     .headers(["Name", "Version"])
//!     .row(["wrcli", "0.1.0"]);
//! let rendered = table.render(false);
//! assert!(rendered.contains("wrcli"));
//!
//! // Panel
//! let panel = Panel::new("Content here").title("Info");
//! let rendered = panel.render(false);
//! assert!(rendered.contains("Content here"));
//!
//! // Rule
//! let rule = Rule::new().title("Section");
//! let rendered = rule.render(false);
//! assert!(rendered.contains("Section"));
//! ```

use std::io::IsTerminal;

// ── Color ─────────────────────────────────────────────────────────────────────

/// A terminal foreground or background color.
///
/// Supports the 16 standard ANSI colors, 8-bit (256-color) fixed palette, and
/// 24-bit RGB true color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
    /// An 8-bit (256-color) terminal color index.
    Fixed(u8),
    /// A 24-bit RGB true color.
    Rgb(u8, u8, u8),
}

impl Color {
    fn fg_code(self) -> String {
        match self {
            Color::Black         => "30".to_owned(),
            Color::Red           => "31".to_owned(),
            Color::Green         => "32".to_owned(),
            Color::Yellow        => "33".to_owned(),
            Color::Blue          => "34".to_owned(),
            Color::Magenta       => "35".to_owned(),
            Color::Cyan          => "36".to_owned(),
            Color::White         => "37".to_owned(),
            Color::BrightBlack   => "90".to_owned(),
            Color::BrightRed     => "91".to_owned(),
            Color::BrightGreen   => "92".to_owned(),
            Color::BrightYellow  => "93".to_owned(),
            Color::BrightBlue    => "94".to_owned(),
            Color::BrightMagenta => "95".to_owned(),
            Color::BrightCyan    => "96".to_owned(),
            Color::BrightWhite   => "97".to_owned(),
            Color::Fixed(n)      => format!("38;5;{}", n),
            Color::Rgb(r, g, b)  => format!("38;2;{};{};{}", r, g, b),
        }
    }

    fn bg_code(self) -> String {
        match self {
            Color::Black         => "40".to_owned(),
            Color::Red           => "41".to_owned(),
            Color::Green         => "42".to_owned(),
            Color::Yellow        => "43".to_owned(),
            Color::Blue          => "44".to_owned(),
            Color::Magenta       => "45".to_owned(),
            Color::Cyan          => "46".to_owned(),
            Color::White         => "47".to_owned(),
            Color::BrightBlack   => "100".to_owned(),
            Color::BrightRed     => "101".to_owned(),
            Color::BrightGreen   => "102".to_owned(),
            Color::BrightYellow  => "103".to_owned(),
            Color::BrightBlue    => "104".to_owned(),
            Color::BrightMagenta => "105".to_owned(),
            Color::BrightCyan    => "106".to_owned(),
            Color::BrightWhite   => "107".to_owned(),
            Color::Fixed(n)      => format!("48;5;{}", n),
            Color::Rgb(r, g, b)  => format!("48;2;{};{};{}", r, g, b),
        }
    }
}

// ── Style ─────────────────────────────────────────────────────────────────────

/// A set of text styling attributes (colors and text decorations).
///
/// Build via the fluent builder API, then call [`Style::apply`] to produce an
/// ANSI-escaped string for a given piece of text.
///
/// # Example
///
/// ```
/// use wrcli::style::{Style, Color};
///
/// let s = Style::new().fg(Color::Green).bold().underline();
/// let text = s.apply("Success", false);
/// assert_eq!(text, "Success"); // no ANSI when not a TTY
/// ```
#[derive(Debug, Clone, Default)]
pub struct Style {
    /// Foreground (text) color.
    pub fg: Option<Color>,
    /// Background color.
    pub bg: Option<Color>,
    /// Bold / increased intensity.
    pub bold: bool,
    /// Dim / decreased intensity.
    pub dim: bool,
    /// Italic text.
    pub italic: bool,
    /// Underlined text.
    pub underline: bool,
    /// Blinking text.
    pub blink: bool,
    /// Reversed foreground and background.
    pub reverse: bool,
    /// Struck-through text.
    pub strikethrough: bool,
}

impl Style {
    /// Create a new, empty `Style`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the foreground (text) color.
    pub fn fg(mut self, c: Color) -> Self {
        self.fg = Some(c);
        self
    }

    /// Set the background color.
    pub fn bg(mut self, c: Color) -> Self {
        self.bg = Some(c);
        self
    }

    /// Enable bold text.
    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    /// Enable dim/faint text.
    pub fn dim(mut self) -> Self {
        self.dim = true;
        self
    }

    /// Enable italic text.
    pub fn italic(mut self) -> Self {
        self.italic = true;
        self
    }

    /// Enable underlined text.
    pub fn underline(mut self) -> Self {
        self.underline = true;
        self
    }

    /// Enable blinking text.
    pub fn blink(mut self) -> Self {
        self.blink = true;
        self
    }

    /// Swap foreground and background colors.
    pub fn reverse(mut self) -> Self {
        self.reverse = true;
        self
    }

    /// Enable strikethrough text.
    pub fn strikethrough(mut self) -> Self {
        self.strikethrough = true;
        self
    }

    /// Apply this style to `text`.
    ///
    /// When `styled` is `true` the returned string is wrapped in ANSI escape
    /// sequences. When `styled` is `false` (e.g. when output is not a TTY) the
    /// original text is returned unchanged.
    pub fn apply(&self, text: &str, styled: bool) -> String {
        if !styled || self.is_plain() {
            return text.to_owned();
        }
        let mut codes: Vec<String> = Vec::new();
        if self.bold          { codes.push("1".to_owned()); }
        if self.dim           { codes.push("2".to_owned()); }
        if self.italic        { codes.push("3".to_owned()); }
        if self.underline     { codes.push("4".to_owned()); }
        if self.blink         { codes.push("5".to_owned()); }
        if self.reverse       { codes.push("7".to_owned()); }
        if self.strikethrough { codes.push("9".to_owned()); }
        if let Some(fg) = self.fg { codes.push(fg.fg_code()); }
        if let Some(bg) = self.bg { codes.push(bg.bg_code()); }

        if codes.is_empty() {
            text.to_owned()
        } else {
            format!("\x1b[{}m{}\x1b[0m", codes.join(";"), text)
        }
    }

    /// Returns `true` if no styling attributes are set.
    fn is_plain(&self) -> bool {
        self.fg.is_none()
            && self.bg.is_none()
            && !self.bold
            && !self.dim
            && !self.italic
            && !self.underline
            && !self.blink
            && !self.reverse
            && !self.strikethrough
    }
}

// ── TTY detection ─────────────────────────────────────────────────────────────

/// Returns `true` when stdout supports ANSI styling.
///
/// Styling is disabled when the `NO_COLOR` environment variable is set or
/// stdout is not connected to a terminal.
pub fn stdout_is_styled() -> bool {
    if std::env::var_os("NO_COLOR").is_some() {
        return false;
    }
    std::io::stdout().is_terminal()
}

/// Returns `true` when stderr supports ANSI styling.
///
/// Styling is disabled when the `NO_COLOR` environment variable is set or
/// stderr is not connected to a terminal.
pub fn stderr_is_styled() -> bool {
    if std::env::var_os("NO_COLOR").is_some() {
        return false;
    }
    std::io::stderr().is_terminal()
}

// ── Convenience print helpers ─────────────────────────────────────────────────

/// Print a success message with a green **✓** prefix to stdout.
///
/// ```
/// wrcli::style::print_success("Build passed");
/// ```
pub fn print_success(msg: &str) {
    let styled = stdout_is_styled();
    let icon = Style::new().fg(Color::Green).bold().apply("✓", styled);
    println!("{} {}", icon, msg);
}

/// Print an error message with a red **✗** prefix to stderr.
///
/// ```
/// wrcli::style::print_error("Build failed");
/// ```
pub fn print_error(msg: &str) {
    let styled = stderr_is_styled();
    let icon = Style::new().fg(Color::Red).bold().apply("✗", styled);
    eprintln!("{} {}", icon, msg);
}

/// Print a warning message with a yellow **⚠** prefix to stdout.
///
/// ```
/// wrcli::style::print_warning("Deprecated flag used");
/// ```
pub fn print_warning(msg: &str) {
    let styled = stdout_is_styled();
    let icon = Style::new().fg(Color::Yellow).bold().apply("⚠", styled);
    println!("{} {}", icon, msg);
}

/// Print an info message with a cyan **ℹ** prefix to stdout.
///
/// ```
/// wrcli::style::print_info("Listening on port 8080");
/// ```
pub fn print_info(msg: &str) {
    let styled = stdout_is_styled();
    let icon = Style::new().fg(Color::Cyan).bold().apply("ℹ", styled);
    println!("{} {}", icon, msg);
}

// ── Table ─────────────────────────────────────────────────────────────────────

/// Column text alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Align {
    /// Align to the left edge (default).
    #[default]
    Left,
    /// Center the text within the column.
    Center,
    /// Align to the right edge.
    Right,
}

/// A formatted table with optional Unicode box-drawing borders.
///
/// # Example
///
/// ```
/// use wrcli::style::{Table, Align};
///
/// let output = Table::new()
///     .headers(["Name", "Version", "Description"])
///     .row(["wrcli", "0.1.0", "CLI framework"])
///     .row(["serde", "1.0",   "Serialization"])
///     .render(false);
///
/// assert!(output.contains("wrcli"));
/// assert!(output.contains("serde"));
/// ```
pub struct Table {
    headers: Vec<String>,
    header_style: Style,
    rows: Vec<Vec<String>>,
    col_align: Vec<Align>,
    border: bool,
}

impl Table {
    /// Create a new empty `Table` with borders enabled.
    pub fn new() -> Self {
        Table {
            headers: Vec::new(),
            header_style: Style::new().bold().fg(Color::Cyan),
            rows: Vec::new(),
            col_align: Vec::new(),
            border: true,
        }
    }

    /// Set the table column headers.
    pub fn headers<I, S>(mut self, headers: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.headers = headers.into_iter().map(Into::into).collect();
        self
    }

    /// Append a data row.
    ///
    /// Extra cells beyond the header count are ignored; missing cells are
    /// rendered as empty strings.
    pub fn row<I, S>(mut self, row: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.rows.push(row.into_iter().map(Into::into).collect());
        self
    }

    /// Set per-column alignment. Unspecified columns default to [`Align::Left`].
    pub fn align(mut self, aligns: Vec<Align>) -> Self {
        self.col_align = aligns;
        self
    }

    /// Show or hide Unicode box-drawing borders (default: `true`).
    pub fn border(mut self, border: bool) -> Self {
        self.border = border;
        self
    }

    /// Override the style applied to header cells.
    pub fn header_style(mut self, style: Style) -> Self {
        self.header_style = style;
        self
    }

    /// Print the table to stdout using [`stdout_is_styled`] for color detection.
    pub fn print(&self) {
        print!("{}", self.render(stdout_is_styled()));
    }

    /// Render the table to a `String`.
    ///
    /// Pass `styled = true` to include ANSI escape sequences, or `false` for
    /// plain ASCII/Unicode output.
    pub fn render(&self, styled: bool) -> String {
        let col_count = self
            .headers
            .len()
            .max(self.rows.iter().map(|r| r.len()).max().unwrap_or(0));
        if col_count == 0 {
            return String::new();
        }

        // Calculate the display width of each column.
        let mut widths = vec![0usize; col_count];
        for (i, h) in self.headers.iter().enumerate() {
            widths[i] = widths[i].max(h.len());
        }
        for row in &self.rows {
            for (i, cell) in row.iter().enumerate() {
                if i < col_count {
                    widths[i] = widths[i].max(cell.len());
                }
            }
        }

        let mut buf = String::new();

        if self.border {
            // ┌──────┬──────┐
            let top: String = widths
                .iter()
                .map(|w| "─".repeat(w + 2))
                .collect::<Vec<_>>()
                .join("┬");
            buf.push_str(&format!("┌{}┐\n", top));
        }

        // Header row
        if !self.headers.is_empty() {
            let row_str = self.render_row(&self.headers, &widths, &self.header_style, styled);
            if self.border {
                buf.push_str(&format!("│{}│\n", row_str));
                // ├──────┼──────┤
                let sep: String = widths
                    .iter()
                    .map(|w| "─".repeat(w + 2))
                    .collect::<Vec<_>>()
                    .join("┼");
                buf.push_str(&format!("├{}┤\n", sep));
            } else {
                buf.push_str(&format!("{}\n", row_str));
                // plain separator
                let plain_sep: String = widths
                    .iter()
                    .map(|w| "-".repeat(*w))
                    .collect::<Vec<_>>()
                    .join("  ");
                buf.push_str(&format!("{}\n", plain_sep));
            }
        }

        // Data rows
        let plain_style = Style::new();
        for (idx, row) in self.rows.iter().enumerate() {
            let row_str = self.render_row(row, &widths, &plain_style, styled);
            if self.border {
                buf.push_str(&format!("│{}│\n", row_str));
                if idx < self.rows.len() - 1 {
                    // ├──────┼──────┤
                    let sep: String = widths
                        .iter()
                        .map(|w| "─".repeat(w + 2))
                        .collect::<Vec<_>>()
                        .join("┼");
                    buf.push_str(&format!("├{}┤\n", sep));
                }
            } else {
                buf.push_str(&format!("{}\n", row_str));
            }
        }

        if self.border {
            // └──────┴──────┘
            let bottom: String = widths
                .iter()
                .map(|w| "─".repeat(w + 2))
                .collect::<Vec<_>>()
                .join("┴");
            buf.push_str(&format!("└{}┘\n", bottom));
        }

        buf
    }

    fn render_row(
        &self,
        cells: &[String],
        widths: &[usize],
        style: &Style,
        styled: bool,
    ) -> String {
        let mut parts: Vec<String> = Vec::with_capacity(widths.len());
        for (i, &w) in widths.iter().enumerate() {
            let cell = cells.get(i).map(|s| s.as_str()).unwrap_or("");
            let align = self.col_align.get(i).copied().unwrap_or(Align::Left);
            let padded = match align {
                Align::Left   => format!(" {:<width$} ", cell, width = w),
                Align::Right  => format!(" {:>width$} ", cell, width = w),
                Align::Center => {
                    let pad = w.saturating_sub(cell.len());
                    let l = pad / 2;
                    let r = pad - l;
                    format!(" {}{}{} ", " ".repeat(l), cell, " ".repeat(r))
                }
            };
            parts.push(style.apply(&padded, styled));
        }
        parts.join("│")
    }
}

impl Default for Table {
    fn default() -> Self {
        Self::new()
    }
}

// ── Panel ─────────────────────────────────────────────────────────────────────

/// A bordered panel with an optional title, rendered using Unicode box-drawing
/// characters.
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
    /// Create a new `Panel` with the given content string.
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

    /// Set the panel title (shown in the top border).
    pub fn title(mut self, t: &str) -> Self {
        self.title = Some(t.to_owned());
        self
    }

    /// Override the style applied to border characters.
    pub fn border_style(mut self, s: Style) -> Self {
        self.border_style = s;
        self
    }

    /// Override the style applied to the title text.
    pub fn title_style(mut self, s: Style) -> Self {
        self.title_style = s;
        self
    }

    /// Set horizontal padding (spaces between border and content). Default: 1.
    pub fn padding(mut self, p: usize) -> Self {
        self.padding = p;
        self
    }

    /// Fix the inner content width. Defaults to the width of the longest content line.
    pub fn width(mut self, w: usize) -> Self {
        self.width = Some(w);
        self
    }

    /// Print the panel to stdout using [`stdout_is_styled`] for color detection.
    pub fn print(&self) {
        print!("{}", self.render(stdout_is_styled()));
    }

    /// Render the panel to a `String`.
    ///
    /// Pass `styled = true` to include ANSI escape sequences, or `false` for
    /// plain ASCII/Unicode output.
    pub fn render(&self, styled: bool) -> String {
        let lines: Vec<&str> = self.content.lines().collect();
        let content_width = lines.iter().map(|l| l.len()).max().unwrap_or(0);
        // title needs " title " + at least 2 dashes on each side
        let title_min = self
            .title
            .as_deref()
            .map(|t| t.len() + 2 + 4) // " title " + "── "
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

        // ── Top border ────────────────────────────────────────────────────────
        if let Some(ref title) = self.title {
            // ╭─ Title ──────────────────╮
            let title_part = format!(" {} ", title);
            let dashes_needed = inner_width + 2; // total inner space
            let left_dashes = 2;
            let right_dashes = dashes_needed.saturating_sub(left_dashes + title_part.len());
            buf.push_str(&format!(
                "{}{}{}{}{}",
                b("╭"),
                b(&"─".repeat(left_dashes)),
                t(&title_part),
                b(&"─".repeat(right_dashes)),
                b("╮"),
            ));
        } else {
            // ╭──────────────────────────╮
            buf.push_str(&format!(
                "{}{}{}",
                b("╭"),
                b(&"─".repeat(inner_width + pad * 2)),
                b("╮"),
            ));
        }
        buf.push('\n');

        // ── Content lines ─────────────────────────────────────────────────────
        let padding_str = " ".repeat(pad);
        for line in &lines {
            let right_fill = inner_width.saturating_sub(line.len());
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

        // ── Bottom border ─────────────────────────────────────────────────────
        // ╰──────────────────────────╯
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

// ── Rule ──────────────────────────────────────────────────────────────────────

/// A horizontal rule, optionally with a centered title.
///
/// # Example
///
/// ```
/// use wrcli::style::{Rule, Style, Color};
///
/// let rendered = Rule::new()
///     .title("Configuration")
///     .style(Style::new().fg(Color::Yellow))
///     .width(60)
///     .render(false);
///
/// assert!(rendered.contains("Configuration"));
/// ```
pub struct Rule {
    title: Option<String>,
    style: Style,
    title_style: Style,
    width: usize,
    line_char: char,
}

impl Rule {
    /// Create a new `Rule` with default settings (80 columns, `─` character).
    pub fn new() -> Self {
        Rule {
            title: None,
            style: Style::new().fg(Color::BrightBlack),
            title_style: Style::new(),
            width: 80,
            line_char: '─',
        }
    }

    /// Set the centered title text.
    pub fn title(mut self, t: &str) -> Self {
        self.title = Some(t.to_owned());
        self
    }

    /// Set the style applied to the rule line characters.
    pub fn style(mut self, s: Style) -> Self {
        self.style = s;
        self
    }

    /// Set the style applied to the title text.
    pub fn title_style(mut self, s: Style) -> Self {
        self.title_style = s;
        self
    }

    /// Set the total line width in columns (default: 80).
    pub fn width(mut self, w: usize) -> Self {
        self.width = w;
        self
    }

    /// Override the line drawing character (default: `─`).
    pub fn line_char(mut self, c: char) -> Self {
        self.line_char = c;
        self
    }

    /// Print the rule to stdout using [`stdout_is_styled`] for color detection.
    pub fn print(&self) {
        println!("{}", self.render(stdout_is_styled()));
    }

    /// Render the rule to a `String`.
    ///
    /// Pass `styled = true` to include ANSI escape sequences, or `false` for
    /// plain output.
    pub fn render(&self, styled: bool) -> String {
        let ch = self.line_char.to_string();
        if let Some(ref title) = self.title {
            let title_part = format!(" {} ", title);
            let remaining = self.width.saturating_sub(title_part.len());
            let left = remaining / 2;
            let right = remaining - left;
            format!(
                "{}{}{}",
                self.style.apply(&ch.repeat(left), styled),
                self.title_style.apply(&title_part, styled),
                self.style.apply(&ch.repeat(right), styled),
            )
        } else {
            self.style.apply(&ch.repeat(self.width), styled)
        }
    }
}

impl Default for Rule {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Style ─────────────────────────────────────────────────────────────────

    #[test]
    fn style_plain_passthrough() {
        // When styled=false, apply() must return the original text unchanged.
        let s = Style::new().fg(Color::Red).bold();
        assert_eq!(s.apply("hello", false), "hello");
    }

    #[test]
    fn style_ansi_codes_present_when_styled() {
        let s = Style::new().fg(Color::Green).bold();
        let out = s.apply("ok", true);
        assert!(out.contains("\x1b["), "expected ANSI escape in output");
        assert!(out.contains("ok"), "original text must be preserved");
        assert!(out.ends_with("\x1b[0m"), "must end with reset");
    }

    #[test]
    fn style_empty_is_passthrough() {
        // A style with no attributes should never emit escape codes.
        let s = Style::new();
        assert_eq!(s.apply("text", true), "text");
    }

    #[test]
    fn style_all_attributes() {
        let s = Style::new()
            .fg(Color::Red)
            .bg(Color::Blue)
            .bold()
            .dim()
            .italic()
            .underline()
            .blink()
            .reverse()
            .strikethrough();
        let out = s.apply("x", true);
        assert!(out.starts_with("\x1b["));
        assert!(out.contains("x"));
    }

    #[test]
    fn color_fg_codes_are_distinct() {
        assert_ne!(Color::Red.fg_code(), Color::Green.fg_code());
        assert_ne!(Color::Blue.fg_code(), Color::Cyan.fg_code());
    }

    #[test]
    fn color_rgb_code_format() {
        assert_eq!(Color::Rgb(10, 20, 30).fg_code(), "38;2;10;20;30");
        assert_eq!(Color::Fixed(42).fg_code(), "38;5;42");
    }

    #[test]
    fn color_bg_codes_are_distinct_from_fg() {
        // Background codes are in the 40-107 range; foreground in 30-97.
        let fg = Color::Red.fg_code();
        let bg = Color::Red.bg_code();
        assert_ne!(fg, bg);
    }

    // ── Table ─────────────────────────────────────────────────────────────────

    #[test]
    fn table_render_contains_data() {
        let out = Table::new()
            .headers(["Name", "Ver"])
            .row(["wrcli", "0.1.0"])
            .render(false);
        assert!(out.contains("Name"));
        assert!(out.contains("wrcli"));
        assert!(out.contains("0.1.0"));
    }

    #[test]
    fn table_render_borders() {
        let out = Table::new()
            .headers(["A", "B"])
            .row(["1", "2"])
            .render(false);
        assert!(out.contains('┌'), "missing top-left corner");
        assert!(out.contains('┘'), "missing bottom-right corner");
        assert!(out.contains('┼'), "missing intersection");
    }

    #[test]
    fn table_no_border() {
        let out = Table::new()
            .headers(["Col"])
            .row(["val"])
            .border(false)
            .render(false);
        assert!(!out.contains('┌'), "borders should be absent");
        assert!(out.contains("Col"));
        assert!(out.contains("val"));
    }

    #[test]
    fn table_empty_returns_empty_string() {
        let out = Table::new().render(false);
        assert!(out.is_empty());
    }

    #[test]
    fn table_align_right() {
        let out = Table::new()
            .headers(["Amount"])
            .row(["42"])
            .align(vec![Align::Right])
            .render(false);
        // Right-aligned: "42" is at the right side of the column
        assert!(out.contains("42"));
    }

    #[test]
    fn table_align_center() {
        let out = Table::new()
            .headers(["X"])
            .row(["Y"])
            .align(vec![Align::Center])
            .render(false);
        assert!(out.contains("X"));
        assert!(out.contains("Y"));
    }

    #[test]
    fn table_multiple_rows() {
        let out = Table::new()
            .headers(["id"])
            .row(["1"])
            .row(["2"])
            .row(["3"])
            .render(false);
        assert!(out.contains('1'));
        assert!(out.contains('2'));
        assert!(out.contains('3'));
    }

    #[test]
    fn table_missing_cells_use_empty() {
        // Row with fewer cells than headers — should not panic.
        let out = Table::new()
            .headers(["A", "B", "C"])
            .row(["only-a"])
            .render(false);
        assert!(out.contains("only-a"));
    }

    // ── Panel ─────────────────────────────────────────────────────────────────

    #[test]
    fn panel_render_contains_content() {
        let out = Panel::new("Hello, World!").render(false);
        assert!(out.contains("Hello, World!"));
    }

    #[test]
    fn panel_render_title() {
        let out = Panel::new("body").title("My Panel").render(false);
        assert!(out.contains("My Panel"));
        assert!(out.contains("body"));
    }

    #[test]
    fn panel_render_borders() {
        let out = Panel::new("content").render(false);
        assert!(out.contains('╭'), "missing top-left corner");
        assert!(out.contains('╯'), "missing bottom-right corner");
        assert!(out.contains('│'), "missing vertical bar");
    }

    #[test]
    fn panel_multiline_content() {
        let out = Panel::new("line one\nline two\nline three").render(false);
        assert!(out.contains("line one"));
        assert!(out.contains("line two"));
        assert!(out.contains("line three"));
    }

    #[test]
    fn panel_fixed_width() {
        let out = Panel::new("hi").width(40).render(false);
        // Every rendered line (except empty) should be at most 40 + border chars wide.
        assert!(out.contains("hi"));
    }

    // ── Rule ──────────────────────────────────────────────────────────────────

    #[test]
    fn rule_plain_line() {
        let out = Rule::new().width(10).render(false);
        assert_eq!(out, "─".repeat(10));
    }

    #[test]
    fn rule_with_title() {
        let out = Rule::new().title("Hello").width(20).render(false);
        assert!(out.contains("Hello"), "title must appear in output");
        assert!(out.contains('─'), "rule characters must be present");
    }

    #[test]
    fn rule_title_centered() {
        let out = Rule::new().title("X").width(11).render(false);
        // " X " is 3 chars; 11 - 3 = 8 remaining → 4 left, 4 right
        let expected = format!("{} X {}", "─".repeat(4), "─".repeat(4));
        assert_eq!(out, expected);
    }

    #[test]
    fn rule_custom_char() {
        let out = Rule::new().line_char('=').width(5).render(false);
        assert_eq!(out, "=====");
    }

    #[test]
    fn rule_ansi_codes_present_when_styled() {
        let out = Rule::new()
            .style(Style::new().fg(Color::Red))
            .width(5)
            .render(true);
        assert!(out.contains("\x1b["));
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    #[test]
    fn stdout_is_styled_returns_bool() {
        // We just ensure it doesn't panic and returns a bool.
        let _ = stdout_is_styled();
        let _ = stderr_is_styled();
    }
}

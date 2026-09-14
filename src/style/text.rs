use super::{Color, Style, stdout_is_styled};

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

    /// Parses inline markup tags (e.g. `"[bold green]Success:[/] [cyan]test.rs[/]"`).
    ///
    /// Supported tags include:
    /// - Text attributes: `bold`, `dim`, `italic`, `underline`, `strikethrough`, `reverse`
    /// - Colors: `red`, `green`, `blue`, `yellow`, `magenta`, `cyan`, `white`, `black`,
    ///   or any color supported by [`Color::from_name`].
    /// - Background colors with `on_`: e.g. `on_red`, `on_blue`, `on_white`.
    /// - Closing tags: `[/]` or `[/tag]`.
    /// - Escaping: `[[` becomes `[`.
    pub fn from_markup(input: &str) -> Self {
        let mut text = Text::new();
        let mut style_stack: Vec<Style> = vec![Style::new()];
        let mut current_literal = String::new();

        let mut chars = input.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '[' {
                if chars.peek() == Some(&'[') {
                    chars.next();
                    current_literal.push('[');
                    continue;
                }
                let mut tag_content = String::new();
                let mut found_close = false;
                for tc in chars.by_ref() {
                    if tc == ']' {
                        found_close = true;
                        break;
                    }
                    tag_content.push(tc);
                }

                if !found_close {
                    current_literal.push('[');
                    current_literal.push_str(&tag_content);
                    continue;
                }

                let tag = tag_content.trim();
                if tag.starts_with('/') {
                    if !current_literal.is_empty() {
                        let cur_style = style_stack.last().cloned().unwrap_or_default();
                        text = text.span(&current_literal, cur_style);
                        current_literal.clear();
                    }
                    if style_stack.len() > 1 {
                        style_stack.pop();
                    } else if let Some(first) = style_stack.first_mut() {
                        *first = Style::new();
                    }
                } else {
                    let mut new_style = style_stack.last().cloned().unwrap_or_default();
                    let mut is_style_tag = false;
                    for token in tag.split_whitespace() {
                        if let Some(bg_color_name) = token
                            .strip_prefix("on_")
                            .or_else(|| token.strip_prefix("on-"))
                            && let Some(c) = Color::from_name(bg_color_name)
                        {
                            new_style = new_style.bg(c);
                            is_style_tag = true;
                            continue;
                        }

                        match token.to_ascii_lowercase().as_str() {
                            "bold" => {
                                new_style = new_style.bold();
                                is_style_tag = true;
                            }
                            "dim" => {
                                new_style = new_style.dim();
                                is_style_tag = true;
                            }
                            "italic" => {
                                new_style = new_style.italic();
                                is_style_tag = true;
                            }
                            "underline" => {
                                new_style = new_style.underline();
                                is_style_tag = true;
                            }
                            "reverse" => {
                                new_style = new_style.reverse();
                                is_style_tag = true;
                            }
                            "strikethrough" | "strike" => {
                                new_style = new_style.strikethrough();
                                is_style_tag = true;
                            }
                            other => {
                                if let Some(color) = Color::from_name(other) {
                                    new_style = new_style.fg(color);
                                    is_style_tag = true;
                                }
                            }
                        }
                    }

                    if is_style_tag {
                        if !current_literal.is_empty() {
                            let cur_style = style_stack.last().cloned().unwrap_or_default();
                            text = text.span(&current_literal, cur_style);
                            current_literal.clear();
                        }
                        style_stack.push(new_style);
                    } else {
                        current_literal.push('[');
                        current_literal.push_str(&tag_content);
                        current_literal.push(']');
                    }
                }
            } else if c == ']' && chars.peek() == Some(&']') {
                chars.next();
                current_literal.push(']');
            } else {
                current_literal.push(c);
            }
        }

        if !current_literal.is_empty() {
            let cur_style = style_stack.last().cloned().unwrap_or_default();
            text = text.span(&current_literal, cur_style);
        }

        text
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

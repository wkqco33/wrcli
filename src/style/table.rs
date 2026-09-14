use super::{BoxStyle, Color, Style, display_width, stdout_is_styled};

/// Column text alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Align {
    #[default]
    Left,
    Center,
    Right,
}

/// A table that draws borders with Unicode box-drawing characters.
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
    box_style: BoxStyle,
    row_separator: bool,
    border_style: Style,
}

impl Table {
    pub fn new() -> Self {
        Table {
            headers: Vec::new(),
            header_style: Style::new().bold().fg(Color::Cyan),
            rows: Vec::new(),
            col_align: Vec::new(),
            border: true,
            box_style: BoxStyle::Square,
            row_separator: true,
            border_style: Style::new(),
        }
    }

    pub fn headers<I, S>(mut self, headers: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.headers = headers.into_iter().map(Into::into).collect();
        self
    }

    pub fn row<I, S>(mut self, row: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.rows.push(row.into_iter().map(Into::into).collect());
        self
    }

    pub fn align(mut self, aligns: Vec<Align>) -> Self {
        self.col_align = aligns;
        self
    }

    pub fn border(mut self, border: bool) -> Self {
        self.border = border;
        self
    }

    pub fn box_style(mut self, bs: BoxStyle) -> Self {
        self.box_style = bs;
        self
    }

    pub fn row_separator(mut self, enable: bool) -> Self {
        self.row_separator = enable;
        self
    }

    pub fn border_style(mut self, style: Style) -> Self {
        self.border_style = style;
        self
    }

    pub fn header_style(mut self, style: Style) -> Self {
        self.header_style = style;
        self
    }

    /// Prints, auto-detecting whether stdout is a TTY.
    pub fn print(&self) {
        print!("{}", self.render(stdout_is_styled()));
    }

    /// Renders the table as a `String`.
    ///
    /// When `styled = true`, includes ANSI escape sequences.
    pub fn render(&self, styled: bool) -> String {
        use std::fmt::Write as _;

        let col_count = self
            .headers
            .len()
            .max(self.rows.iter().map(|r| r.len()).max().unwrap_or(0));
        if col_count == 0 {
            return String::new();
        }

        // Widths must be computed with display_width so alignment holds for non-ASCII characters.
        let mut widths = vec![0usize; col_count];
        for (i, h) in self.headers.iter().enumerate() {
            widths[i] = widths[i].max(display_width(h));
        }
        for row in &self.rows {
            for (i, cell) in row.iter().enumerate() {
                if i < col_count {
                    widths[i] = widths[i].max(display_width(cell));
                }
            }
        }

        // Compute the border strings only once.
        let b = |s: &str| self.border_style.apply(s, styled);
        let bs = self.box_style;
        let v_str = bs.vertical().to_string();

        let (top_line, header_sep_line, row_sep_line, bottom_line, plain_sep_line) = if self.border
        {
            if bs == BoxStyle::Markdown {
                let mut hsep = String::from("|");
                for &w in &widths {
                    let dashes: String = "-".repeat(w + 2);
                    hsep.push_str(&dashes);
                    hsep.push('|');
                }
                hsep.push('\n');
                (
                    String::new(),
                    b(&hsep),
                    String::new(),
                    String::new(),
                    String::new(),
                )
            } else {
                let mut top = String::from(bs.top_left());
                let mut hsep = String::from(bs.t_left());
                let mut rsep = String::from(bs.t_left());
                let mut bot = String::from(bs.bottom_left());
                for (i, &w) in widths.iter().enumerate() {
                    if i > 0 {
                        top.push(bs.t_top());
                        hsep.push(bs.cross());
                        rsep.push(bs.cross());
                        bot.push(bs.t_bottom());
                    }
                    let dashes: String = std::iter::repeat_n(bs.horizontal(), w + 2).collect();
                    top.push_str(&dashes);
                    hsep.push_str(&dashes);
                    rsep.push_str(&dashes);
                    bot.push_str(&dashes);
                }
                top.push(bs.top_right());
                top.push('\n');
                hsep.push(bs.t_right());
                hsep.push('\n');
                rsep.push(bs.t_right());
                rsep.push('\n');
                bot.push(bs.bottom_right());
                bot.push('\n');
                (b(&top), b(&hsep), b(&rsep), b(&bot), String::new())
            }
        } else {
            let plain: String = widths
                .iter()
                .enumerate()
                .fold(String::new(), |mut s, (i, &w)| {
                    if i > 0 {
                        s.push_str("  ");
                    }
                    s.extend(std::iter::repeat_n('-', w));
                    s
                });
            (
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                plain + "\n",
            )
        };

        let mut buf = String::new();

        if self.border && !top_line.is_empty() {
            buf.push_str(&top_line);
        }

        if !self.headers.is_empty() {
            let row_str = self.render_row(&self.headers, &widths, &self.header_style, styled);
            if self.border {
                writeln!(buf, "{}{}{}", b(&v_str), row_str, b(&v_str)).unwrap();
                buf.push_str(&header_sep_line);
            } else {
                writeln!(buf, "{}", row_str).unwrap();
                buf.push_str(&plain_sep_line);
            }
        }

        let plain_style = Style::new();
        for (idx, row) in self.rows.iter().enumerate() {
            let row_str = self.render_row(row, &widths, &plain_style, styled);
            if self.border {
                writeln!(buf, "{}{}{}", b(&v_str), row_str, b(&v_str)).unwrap();
                if self.row_separator && idx < self.rows.len() - 1 {
                    buf.push_str(&row_sep_line);
                }
            } else {
                writeln!(buf, "{}", row_str).unwrap();
            }
        }

        if self.border && !bottom_line.is_empty() {
            buf.push_str(&bottom_line);
        }

        buf
    }

    /// Rendering for `--plain`: one record per line (tab-separated), with no borders or alignment.
    ///
    /// Can be piped directly into tools like `grep`/`awk`.
    pub fn render_plain(&self) -> String {
        use std::fmt::Write as _;
        let mut out = String::new();
        if !self.headers.is_empty() {
            let _ = writeln!(out, "{}", self.headers.join("\t"));
        }
        for row in &self.rows {
            let _ = writeln!(out, "{}", row.join("\t"));
        }
        out
    }

    fn render_row(
        &self,
        cells: &[String],
        widths: &[usize],
        style: &Style,
        styled: bool,
    ) -> String {
        let v_str = self.box_style.vertical().to_string();
        let border_v = self.border_style.apply(&v_str, styled);
        let mut parts: Vec<String> = Vec::with_capacity(widths.len());
        for (i, &w) in widths.iter().enumerate() {
            let cell = cells.get(i).map(|s| s.as_str()).unwrap_or("");
            let align = self.col_align.get(i).copied().unwrap_or(Align::Left);
            let cell_w = display_width(cell);
            let pad = w.saturating_sub(cell_w);
            let padded = if self.border {
                match align {
                    Align::Left => format!(" {}{} ", cell, " ".repeat(pad)),
                    Align::Right => format!(" {}{} ", " ".repeat(pad), cell),
                    Align::Center => {
                        let l = pad / 2;
                        let r = pad - l;
                        format!(" {}{}{} ", " ".repeat(l), cell, " ".repeat(r))
                    }
                }
            } else {
                match align {
                    Align::Left => format!("{}{}", cell, " ".repeat(pad)),
                    Align::Right => format!("{}{}", " ".repeat(pad), cell),
                    Align::Center => {
                        let l = pad / 2;
                        let r = pad - l;
                        format!("{}{}{}", " ".repeat(l), cell, " ".repeat(r))
                    }
                }
            };
            parts.push(style.apply(&padded, styled));
        }
        if self.border {
            parts.join(&border_v)
        } else {
            parts.join("  ")
        }
    }
}

impl Default for Table {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_contains_data() {
        let out = Table::new()
            .headers(["Name", "Ver"])
            .row(["wrcli", "0.1.0"])
            .render(false);
        assert!(out.contains("Name"));
        assert!(out.contains("wrcli"));
        assert!(out.contains("0.1.0"));
    }

    #[test]
    fn render_borders() {
        let out = Table::new()
            .headers(["A", "B"])
            .row(["1", "2"])
            .render(false);
        assert!(out.contains('┌'));
        assert!(out.contains('┘'));
        assert!(out.contains('┼'));
    }

    #[test]
    fn no_border() {
        let out = Table::new()
            .headers(["Col1", "Col2"])
            .row(["val1", "val2"])
            .border(false)
            .render(false);
        assert!(!out.contains('┌'));
        assert!(!out.contains('│'));
        assert!(out.contains("Col1"));
        assert!(out.contains("val1"));
    }

    #[test]
    fn empty_returns_empty_string() {
        let out = Table::new().render(false);
        assert!(out.is_empty());
    }

    #[test]
    fn align_right() {
        let out = Table::new()
            .headers(["Amount"])
            .row(["42"])
            .align(vec![Align::Right])
            .render(false);
        assert!(out.contains("42"));
    }

    #[test]
    fn align_center() {
        let out = Table::new()
            .headers(["X"])
            .row(["Y"])
            .align(vec![Align::Center])
            .render(false);
        assert!(out.contains("X"));
        assert!(out.contains("Y"));
    }

    #[test]
    fn multiple_rows() {
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
    fn missing_cells_use_empty() {
        let out = Table::new()
            .headers(["A", "B", "C"])
            .row(["only-a"])
            .render(false);
        assert!(out.contains("only-a"));
    }

    #[test]
    fn non_ascii_cells_stay_aligned() {
        // With display_width, the border (│) of every row must align at the same position.
        let out = Table::new()
            .headers(["이름", "설명"])
            .row(["wrcli", "설명 텍스트"])
            .row(["ab", "x"])
            .render(false);
        let border_positions: Vec<Vec<usize>> = out
            .lines()
            .map(|line| {
                let mut pos = 0usize;
                line.chars()
                    .filter_map(|c| {
                        let w = crate::style::display_width(&c.to_string());
                        let p = pos;
                        pos += w;
                        if c == '│' { Some(p) } else { None }
                    })
                    .collect()
            })
            .filter(|v: &Vec<usize>| !v.is_empty())
            .collect();
        assert!(border_positions.windows(2).all(|w| w[0] == w[1]));
    }

    #[test]
    fn styled_cells_stay_aligned() {
        let styled_cell = Style::new().bold().fg(Color::Red).apply("wrcli", true);
        let out = Table::new()
            .headers(["Name", "Desc"])
            .row([styled_cell, "CLI framework".to_string()])
            .row(["plain".to_string(), "plain desc".to_string()])
            .render(true);
        let border_positions: Vec<Vec<usize>> = out
            .lines()
            .map(|line| {
                let mut pos = 0usize;
                let mut chars = line.chars().peekable();
                let mut positions = Vec::new();
                while let Some(c) = chars.next() {
                    if c == '\x1b'
                        && let Some(&'[') = chars.peek()
                    {
                        chars.next();
                        while let Some(&seq_c) = chars.peek() {
                            chars.next();
                            if (seq_c as u32) >= 0x40 && (seq_c as u32) <= 0x7E {
                                break;
                            }
                        }
                        continue;
                    }

                    let w = crate::style::display_width(&c.to_string());
                    if c == '│' {
                        positions.push(pos);
                    }
                    pos += w;
                }
                positions
            })
            .filter(|v: &Vec<usize>| !v.is_empty())
            .collect();
        assert!(
            border_positions.windows(2).all(|w| w[0] == w[1]),
            "mismatched borders: {border_positions:?}"
        );
    }
}

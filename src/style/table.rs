use super::{Style, Color, stdout_is_styled};

/// 컬럼 텍스트 정렬.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Align {
    #[default]
    Left,
    Center,
    Right,
}

/// Unicode 박스 그리기 문자로 테두리를 표시하는 테이블.
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
    pub fn new() -> Self {
        Table {
            headers: Vec::new(),
            header_style: Style::new().bold().fg(Color::Cyan),
            rows: Vec::new(),
            col_align: Vec::new(),
            border: true,
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

    pub fn header_style(mut self, style: Style) -> Self {
        self.header_style = style;
        self
    }

    /// stdout 이 TTY인지 자동 감지해서 출력.
    pub fn print(&self) {
        print!("{}", self.render(stdout_is_styled()));
    }

    /// 테이블을 `String`으로 렌더링.
    ///
    /// `styled = true`이면 ANSI 이스케이프 시퀀스 포함.
    pub fn render(&self, styled: bool) -> String {
        let col_count = self
            .headers
            .len()
            .max(self.rows.iter().map(|r| r.len()).max().unwrap_or(0));
        if col_count == 0 {
            return String::new();
        }

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
            let top: String = widths
                .iter()
                .map(|w| "─".repeat(w + 2))
                .collect::<Vec<_>>()
                .join("┬");
            buf.push_str(&format!("┌{}┐\n", top));
        }

        if !self.headers.is_empty() {
            let row_str = self.render_row(&self.headers, &widths, &self.header_style, styled);
            if self.border {
                buf.push_str(&format!("│{}│\n", row_str));
                let sep: String = widths
                    .iter()
                    .map(|w| "─".repeat(w + 2))
                    .collect::<Vec<_>>()
                    .join("┼");
                buf.push_str(&format!("├{}┤\n", sep));
            } else {
                buf.push_str(&format!("{}\n", row_str));
                let plain_sep: String = widths
                    .iter()
                    .map(|w| "-".repeat(*w))
                    .collect::<Vec<_>>()
                    .join("  ");
                buf.push_str(&format!("{}\n", plain_sep));
            }
        }

        let plain_style = Style::new();
        for (idx, row) in self.rows.iter().enumerate() {
            let row_str = self.render_row(row, &widths, &plain_style, styled);
            if self.border {
                buf.push_str(&format!("│{}│\n", row_str));
                if idx < self.rows.len() - 1 {
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
                Align::Left => format!(" {:<width$} ", cell, width = w),
                Align::Right => format!(" {:>width$} ", cell, width = w),
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
            .headers(["Col"])
            .row(["val"])
            .border(false)
            .render(false);
        assert!(!out.contains('┌'));
        assert!(out.contains("Col"));
        assert!(out.contains("val"));
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
}

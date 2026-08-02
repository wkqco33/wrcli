use super::{Color, Style, display_width, stdout_is_styled};

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
        use std::fmt::Write as _;

        let col_count = self
            .headers
            .len()
            .max(self.rows.iter().map(|r| r.len()).max().unwrap_or(0));
        if col_count == 0 {
            return String::new();
        }

        // 문자 수 기준으로 폭을 계산해야 format!의 {:<width$} 패딩과 일치함
        // (str::len()은 바이트 길이라 비-ASCII 문자에서 정렬이 깨짐).
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

        // border 문자열을 한 번만 계산 (separator는 행마다 동일)
        let (top_line, header_sep_line, row_sep_line, bottom_line, plain_sep_line) = if self.border
        {
            let mut top = String::from("┌");
            let mut hsep = String::from("├");
            let mut rsep = String::from("├");
            let mut bot = String::from("└");
            for (i, &w) in widths.iter().enumerate() {
                if i > 0 {
                    top.push('┬');
                    hsep.push('┼');
                    rsep.push('┼');
                    bot.push('┴');
                }
                let dashes: String = "─".repeat(w + 2);
                top.push_str(&dashes);
                hsep.push_str(&dashes);
                rsep.push_str(&dashes);
                bot.push_str(&dashes);
            }
            top.push_str("┐\n");
            hsep.push_str("┤\n");
            rsep.push_str("┤\n");
            bot.push_str("┘\n");
            (top, hsep, rsep, bot, String::new())
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

        if self.border {
            buf.push_str(&top_line);
        }

        if !self.headers.is_empty() {
            let row_str = self.render_row(&self.headers, &widths, &self.header_style, styled);
            if self.border {
                writeln!(buf, "│{}│", row_str).unwrap();
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
                writeln!(buf, "│{}│", row_str).unwrap();
                if idx < self.rows.len() - 1 {
                    buf.push_str(&row_sep_line);
                }
            } else {
                writeln!(buf, "{}", row_str).unwrap();
            }
        }

        if self.border {
            buf.push_str(&bottom_line);
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
            let cell_w = display_width(cell);
            let pad = w.saturating_sub(cell_w);
            let padded = match align {
                Align::Left => {
                    let s = format!(" {}{} ", cell, " ".repeat(pad));
                    s
                }
                Align::Right => {
                    let s = format!(" {}{} ", " ".repeat(pad), cell);
                    s
                }
                Align::Center => {
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

    #[test]
    fn non_ascii_cells_stay_aligned() {
        // 회귀 테스트: display_width 기준으로 패딩이 계산되어야
        // 모든 행의 테두리(│)가 같은 표시폭 위치에 정렬됨.
        let out = Table::new()
            .headers(["이름", "설명"])
            .row(["wrcli", "설명 텍스트"])
            .row(["ab", "x"])
            .render(false);
        // display_width 기준으로 │ 위치가 모든 행에서 일치해야 함
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
}

use super::{Style, Color, stdout_is_styled};

/// Unicode 박스 그리기 문자로 테두리를 표시하는 패널 (선택적 제목 포함).
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

    /// stdout 이 TTY인지 자동 감지해서 출력.
    pub fn print(&self) {
        print!("{}", self.render(stdout_is_styled()));
    }

    /// 패널을 `String`으로 렌더링.
    ///
    /// `styled = true`이면 ANSI 이스케이프 시퀀스 포함.
    pub fn render(&self, styled: bool) -> String {
        // 문자 수 기준으로 폭을 계산 (str::len()은 바이트 길이라 비-ASCII에서 정렬이 깨짐).
        let lines: Vec<&str> = self.content.lines().collect();
        let content_width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
        let title_min = self
            .title
            .as_deref()
            .map(|t| t.chars().count() + 2 + 4)
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
            let dashes_needed = inner_width + 2;
            let left_dashes = 2;
            let right_dashes = dashes_needed.saturating_sub(left_dashes + title_part.chars().count());
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
            let right_fill = inner_width.saturating_sub(line.chars().count());
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
    fn non_ascii_lines_stay_aligned() {
        // 회귀 테스트: 폭 계산이 바이트 길이 기준이면 한글이 포함된 줄의
        // 우측 테두리(│)가 다른 줄과 다른 문자 위치에 오게 됨.
        let out = Panel::new("한글 콘텐츠\nshort").render(false);
        let right_border_col: Vec<usize> = out
            .lines()
            .filter(|l| l.starts_with('│'))
            .map(|l| l.chars().count() - 1)
            .collect();
        assert!(right_border_col.windows(2).all(|w| w[0] == w[1]));
    }
}

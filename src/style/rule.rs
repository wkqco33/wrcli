use super::{Style, Color, stdout_is_styled};

/// 선택적으로 중앙 제목이 있는 수평 구분선.
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
    /// 기본 설정(80컬럼, `─` 문자)으로 새 Rule 생성.
    pub fn new() -> Self {
        Rule {
            title: None,
            style: Style::new().fg(Color::BrightBlack),
            title_style: Style::new(),
            width: 80,
            line_char: '─',
        }
    }

    pub fn title(mut self, t: &str) -> Self {
        self.title = Some(t.to_owned());
        self
    }

    pub fn style(mut self, s: Style) -> Self {
        self.style = s;
        self
    }

    pub fn title_style(mut self, s: Style) -> Self {
        self.title_style = s;
        self
    }

    pub fn width(mut self, w: usize) -> Self {
        self.width = w;
        self
    }

    pub fn line_char(mut self, c: char) -> Self {
        self.line_char = c;
        self
    }

    /// stdout 이 TTY인지 자동 감지해서 출력.
    pub fn print(&self) {
        println!("{}", self.render(stdout_is_styled()));
    }

    /// 구분선을 `String`으로 렌더링.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_line() {
        let out = Rule::new().width(10).render(false);
        assert_eq!(out, "─".repeat(10));
    }

    #[test]
    fn with_title() {
        let out = Rule::new().title("Hello").width(20).render(false);
        assert!(out.contains("Hello"));
        assert!(out.contains('─'));
    }

    #[test]
    fn title_centered() {
        let out = Rule::new().title("X").width(11).render(false);
        // " X " 3자; 11 - 3 = 8 → 좌 4, 우 4
        let expected = format!("{} X {}", "─".repeat(4), "─".repeat(4));
        assert_eq!(out, expected);
    }

    #[test]
    fn custom_char() {
        let out = Rule::new().line_char('=').width(5).render(false);
        assert_eq!(out, "=====");
    }

    #[test]
    fn ansi_codes_when_styled() {
        let out = Rule::new()
            .style(Style::new().fg(Color::Red))
            .width(5)
            .render(true);
        assert!(out.contains("\x1b["));
    }
}

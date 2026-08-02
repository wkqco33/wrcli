use super::{Style, stdout_is_styled};

/// 서로 다른 스타일의 스팬(span)을 이어붙여 하나의 텍스트로 렌더링.
///
/// rich 라이브러리의 `Text`에서 영감을 받음. 각 스팬에 개별 스타일을 적용하고,
/// ANSI 이스케이프 코드로 감싸 병합한다.
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

    /// 기본 스타일(스타일 없음)로 텍스트 추가.
    pub fn plain(mut self, s: &str) -> Self {
        self.spans.push((s.to_owned(), Style::new()));
        self
    }

    /// 주어진 스타일로 텍스트 추가.
    pub fn span(mut self, s: &str, style: Style) -> Self {
        self.spans.push((s.to_owned(), style));
        self
    }

    /// 명시적 스타일로 텍스트 추가. (`plain`의 스타일 버전.)
    pub fn plain_styled(mut self, s: &str, style: Style) -> Self {
        self.spans.push((s.to_owned(), style));
        self
    }

    /// stdout 이 TTY인지 자동 감지해서 출력.
    pub fn print(&self) {
        print!("{}", self.render(stdout_is_styled()));
    }

    /// 텍스트를 `String`으로 렌더링.
    ///
    /// `styled = true`이면 ANSI 이스케이프 시퀀스 포함.
    pub fn render(&self, styled: bool) -> String {
        let mut buf = String::new();
        for (text, style) in &self.spans {
            buf.push_str(&style.apply(text, styled));
        }
        buf
    }
}

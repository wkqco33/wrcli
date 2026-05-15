use super::Color;

/// 텍스트 스타일 속성 집합 (색상 + 장식).
///
/// 플루언트 빌더 API로 구성 후 [`Style::apply`]로 ANSI 이스케이프 문자열 생성.
///
/// # Example
///
/// ```
/// use wrcli::style::{Style, Color};
///
/// let s = Style::new().fg(Color::Green).bold().underline();
/// let text = s.apply("Success", false);
/// assert_eq!(text, "Success"); // TTY 아닐 때는 원본 반환
/// ```
#[derive(Debug, Clone, Default)]
pub struct Style {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
    pub underline: bool,
    pub blink: bool,
    pub reverse: bool,
    pub strikethrough: bool,
}

impl Style {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn fg(mut self, c: Color) -> Self {
        self.fg = Some(c);
        self
    }

    pub fn bg(mut self, c: Color) -> Self {
        self.bg = Some(c);
        self
    }

    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    pub fn dim(mut self) -> Self {
        self.dim = true;
        self
    }

    pub fn italic(mut self) -> Self {
        self.italic = true;
        self
    }

    pub fn underline(mut self) -> Self {
        self.underline = true;
        self
    }

    pub fn blink(mut self) -> Self {
        self.blink = true;
        self
    }

    pub fn reverse(mut self) -> Self {
        self.reverse = true;
        self
    }

    pub fn strikethrough(mut self) -> Self {
        self.strikethrough = true;
        self
    }

    /// 스타일을 `text`에 적용.
    ///
    /// `styled`가 `true`이면 ANSI 이스케이프 시퀀스로 감싼 문자열 반환.
    /// `false`이면 원본 텍스트 그대로 반환.
    pub fn apply(&self, text: &str, styled: bool) -> String {
        if !styled || self.is_plain() {
            return text.to_owned();
        }
        let mut codes: Vec<String> = Vec::new();
        if self.bold { codes.push("1".to_owned()); }
        if self.dim { codes.push("2".to_owned()); }
        if self.italic { codes.push("3".to_owned()); }
        if self.underline { codes.push("4".to_owned()); }
        if self.blink { codes.push("5".to_owned()); }
        if self.reverse { codes.push("7".to_owned()); }
        if self.strikethrough { codes.push("9".to_owned()); }
        if let Some(fg) = self.fg { codes.push(fg.fg_code()); }
        if let Some(bg) = self.bg { codes.push(bg.bg_code()); }

        if codes.is_empty() {
            text.to_owned()
        } else {
            format!("\x1b[{}m{}\x1b[0m", codes.join(";"), text)
        }
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_passthrough() {
        let s = Style::new().fg(Color::Red).bold();
        assert_eq!(s.apply("hello", false), "hello");
    }

    #[test]
    fn ansi_codes_present_when_styled() {
        let s = Style::new().fg(Color::Green).bold();
        let out = s.apply("ok", true);
        assert!(out.contains("\x1b["));
        assert!(out.contains("ok"));
        assert!(out.ends_with("\x1b[0m"));
    }

    #[test]
    fn empty_style_is_passthrough() {
        let s = Style::new();
        assert_eq!(s.apply("text", true), "text");
    }

    #[test]
    fn all_attributes() {
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
}

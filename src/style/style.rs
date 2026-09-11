use super::Color;
use std::fmt::Write as _;

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
    pub overline: bool,
    pub blink: bool,
    pub reverse: bool,
    pub hide: bool,
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

    pub fn overline(mut self) -> Self {
        self.overline = true;
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

    pub fn hide(mut self) -> Self {
        self.hide = true;
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

        let mut codes = String::with_capacity(16);
        let mut separator = false;
        macro_rules! push_code {
            ($($arg:tt)*) => {{
                if separator {
                    codes.push(';');
                }
                let _ = write!(codes, $($arg)*);
                separator = true;
            }};
        }

        if self.bold {
            push_code!("1");
        }
        if self.dim {
            push_code!("2");
        }
        if self.italic {
            push_code!("3");
        }
        if self.underline {
            push_code!("4");
        }
        if self.blink {
            push_code!("5");
        }
        if self.reverse {
            push_code!("7");
        }
        if self.hide {
            push_code!("8");
        }
        if self.strikethrough {
            push_code!("9");
        }
        if self.overline {
            push_code!("53");
        }
        if let Some(fg) = self.fg {
            if separator {
                codes.push(';');
            }
            fg.write_fg_code(&mut codes);
            separator = true;
        }
        if let Some(bg) = self.bg {
            if separator {
                codes.push(';');
            }
            bg.write_bg_code(&mut codes);
        }

        format!("\x1b[{}m{}\x1b[0m", codes, text)
    }

    fn is_plain(&self) -> bool {
        self.fg.is_none()
            && self.bg.is_none()
            && !self.bold
            && !self.dim
            && !self.italic
            && !self.underline
            && !self.overline
            && !self.blink
            && !self.reverse
            && !self.hide
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

    #[test]
    fn overline_and_hide_codes() {
        let s = Style::new().overline().hide();
        let out = s.apply("x", true);
        assert!(out.starts_with("\x1b[8;53m"), "got: {}", out);
        assert!(out.contains("x"));
    }

    #[test]
    fn overline_hide_passthrough_when_unstyled() {
        let s = Style::new().overline().hide();
        assert_eq!(s.apply("x", false), "x");
    }
}

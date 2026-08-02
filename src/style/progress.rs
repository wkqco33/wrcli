use super::{Color, Style};

/// 터미널 진행률 표시줄.
///
/// rich 라이브러리의 `Progress`/`ProgressBar`에서 영감을 받은 단순한 비차단 렌더링.
///
/// # Example
///
/// ```
/// use wrcli::style::Progress;
///
/// let bar = Progress::new(100).progress(42).width(20).render(false);
/// assert!(bar.contains("42%"));
/// ```
pub struct Progress {
    total: f64,
    current: f64,
    width: usize,
    label: String,
    bar_style: Style,
    filled: char,
    empty: char,
}

impl Progress {
    pub fn new(total: u64) -> Self {
        Progress {
            total: total as f64,
            current: 0.0,
            width: 30,
            label: String::new(),
            bar_style: Style::new().fg(Color::Green),
            filled: '#',
            empty: '-',
        }
    }

    /// 진행률(0..total) 설정.
    pub fn progress(mut self, current: u64) -> Self {
        self.current = current as f64;
        self
    }

    /// 표시줄 너비 (기본 30).
    pub fn width(mut self, w: usize) -> Self {
        self.width = w;
        self
    }

    /// 표시줄 앞에 붙는 라벨.
    pub fn label(mut self, label: &str) -> Self {
        self.label = label.to_owned();
        self
    }

    /// 채워진 부분의 스타일 (기본: 녹색).
    pub fn bar_style(mut self, s: Style) -> Self {
        self.bar_style = s;
        self
    }

    /// 채워진 문자 (기본 `#`).
    pub fn filled_char(mut self, c: char) -> Self {
        self.filled = c;
        self
    }

    /// 빈 문자 (기본 `-`).
    pub fn empty_char(mut self, c: char) -> Self {
        self.empty = c;
        self
    }

    /// 진행률을 `String`으로 렌더링.
    pub fn render(&self, styled: bool) -> String {
        let ratio = if self.total <= 0.0 {
            0.0
        } else {
            (self.current / self.total).clamp(0.0, 1.0)
        };
        let pct = (ratio * 100.0).round() as u64;
        let filled_len = (ratio * self.width as f64).round() as usize;

        let mut bar = String::with_capacity(self.width + 2);
        bar.push('[');
        let filled: String = std::iter::repeat_n(self.filled, filled_len).collect();
        let empty: String =
            std::iter::repeat_n(self.empty, self.width.saturating_sub(filled_len)).collect();
        bar.push_str(&self.bar_style.apply(&filled, styled));
        bar.push_str(&empty);
        bar.push(']');

        if self.label.is_empty() {
            format!("{} {:>3}%", bar, pct)
        } else {
            format!("{} {} {:>3}%", self.label, bar, pct)
        }
    }
}

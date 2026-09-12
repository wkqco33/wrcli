use super::{Color, Style, stdout_is_styled, stdout_is_terminal};
use std::io::Write;

/// Terminal progress bar.
///
/// Simple, non-blocking rendering inspired by the rich library's `Progress`/`ProgressBar`.
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

    /// Sets the progress (0..total).
    pub fn progress(mut self, current: u64) -> Self {
        self.current = current as f64;
        self
    }

    /// Bar width (default 30).
    pub fn width(mut self, w: usize) -> Self {
        self.width = w;
        self
    }

    /// Label placed before the bar.
    pub fn label(mut self, label: &str) -> Self {
        self.label = label.to_owned();
        self
    }

    /// Style of the filled portion (default: green).
    pub fn bar_style(mut self, s: Style) -> Self {
        self.bar_style = s;
        self
    }

    /// Filled character (default `#`).
    pub fn filled_char(mut self, c: char) -> Self {
        self.filled = c;
        self
    }

    /// Empty character (default `-`).
    pub fn empty_char(mut self, c: char) -> Self {
        self.empty = c;
        self
    }

    /// Renders the progress bar as a `String`.
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

    /// Draws the progress bar.
    ///
    /// On a TTY it overwrites the current line (`\r`), while on pipes and CI logs
    /// it prints nothing (clig.dev: no animation on a non-TTY).
    pub fn draw(&self) {
        if stdout_is_terminal() {
            print!("\r{}", self.render(stdout_is_styled()));
            let _ = std::io::stdout().flush();
        }
    }

    /// Finishes the progress bar.
    ///
    /// On a TTY it completes the current line; on a non-TTY it prints the final state on one line.
    pub fn finish(&self) {
        if stdout_is_terminal() {
            println!("\r{}", self.render(stdout_is_styled()));
        } else {
            println!("{}", self.render(false));
        }
    }
}

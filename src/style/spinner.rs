use super::{Color, Style, stdout_is_styled, stdout_is_terminal};
use std::io::Write;

const DEFAULT_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// A terminal spinner for long-running operations.
///
/// Follows clig.dev animation rules: animated via `\r` when connected to a TTY,
/// but suppresses animations in non-TTY environments (pipes and CI).
///
/// # Example
///
/// ```
/// use wrcli::style::Spinner;
///
/// let mut sp = Spinner::new("Downloading assets...");
/// sp.step();
/// ```
pub struct Spinner {
    message: String,
    frame: usize,
    frames: &'static [&'static str],
    style: Style,
    message_style: Style,
}

impl Spinner {
    pub fn new(message: &str) -> Self {
        Spinner {
            message: message.to_owned(),
            frame: 0,
            frames: DEFAULT_FRAMES,
            style: Style::new().fg(Color::Cyan).bold(),
            message_style: Style::new(),
        }
    }

    /// Sets custom frames for the spinner animation.
    pub fn frames(mut self, frames: &'static [&'static str]) -> Self {
        self.frames = frames;
        self
    }

    /// Sets the spinner symbol style.
    pub fn style(mut self, s: Style) -> Self {
        self.style = s;
        self
    }

    /// Sets the message style.
    pub fn message_style(mut self, s: Style) -> Self {
        self.message_style = s;
        self
    }

    /// Updates the message text.
    pub fn set_message(&mut self, msg: &str) {
        self.message = msg.to_owned();
    }

    /// Advances the frame counter without printing.
    pub fn step(&mut self) {
        if !self.frames.is_empty() {
            self.frame = (self.frame + 1) % self.frames.len();
        }
    }

    /// Renders the spinner string for the current frame.
    pub fn render(&self, styled: bool) -> String {
        let sym = if self.frames.is_empty() {
            ""
        } else {
            self.frames[self.frame % self.frames.len()]
        };
        let sym_styled = self.style.apply(sym, styled);
        let msg_styled = self.message_style.apply(&self.message, styled);
        format!("{} {}", sym_styled, msg_styled)
    }

    /// Advances the spinner and writes to stdout if connected to a TTY.
    pub fn tick(&mut self) {
        self.step();
        if stdout_is_terminal() {
            print!("\r{}", self.render(stdout_is_styled()));
            let _ = std::io::stdout().flush();
        }
    }

    /// Finishes the spinner. On a TTY clears/ends the line, otherwise prints nothing extra.
    pub fn finish(&self) {
        if stdout_is_terminal() {
            println!();
        }
    }

    /// Finishes the spinner and prints a final completion message.
    pub fn finish_with_message(&self, msg: &str) {
        if stdout_is_terminal() {
            println!("\r{}", msg);
        } else {
            println!("{}", msg);
        }
    }
}

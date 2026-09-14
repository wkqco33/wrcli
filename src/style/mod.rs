//! Rich terminal styling — colors, text attributes, tables, panels, rules.
//!
//! Inspired by Python's [rich](https://github.com/Textualize/rich) library.
//!
//! Styling is automatically disabled when:
//! - `NO_COLOR` is set to a non-empty value
//! - `TERM=dumb`
//! - an app-specific `<APP>_NO_COLOR` (e.g. `MYAPP_NO_COLOR`) is set
//! - the output stream is not connected to a terminal (pipe, etc.)
//!
//! `FORCE_COLOR` (non-empty) bypasses the detection logic and turns color on,
//! while the global override set via `set_color_choice` (`--no-color` / `--color=<when>`)
//! has the highest priority.
//!
//! # Quick start
//!
//! ```
//! use wrcli::style::{Color, Style, Panel, Table, Rule, Tree, Text, Progress, Align};
//!
//! // Styled text
//! let s = Style::new().fg(Color::Green).bold();
//! let text = s.apply("Hello, World!", false);
//! assert_eq!(text, "Hello, World!");
//!
//! // Table
//! let table = Table::new()
//!     .headers(["Name", "Version"])
//!     .row(["wrcli", "0.1.0"]);
//! let rendered = table.render(false);
//! assert!(rendered.contains("wrcli"));
//!
//! // Panel
//! let panel = Panel::new("Content here").title("Info");
//! let rendered = panel.render(false);
//! assert!(rendered.contains("Content here"));
//!
//! // Rule
//! let rule = Rule::new().title("Section");
//! let rendered = rule.render(false);
//! assert!(rendered.contains("Section"));
//! ```

use std::io::IsTerminal;
use std::sync::RwLock;
use std::sync::atomic::{AtomicU8, Ordering};

mod badge;
mod box_style;
mod color;
mod keyval;
mod list;
pub mod pager;
mod panel;
mod progress;
mod rule;
mod spinner;
#[allow(clippy::module_inception)]
mod style;
mod table;
mod text;
mod tree;

pub use badge::Badge;
pub use box_style::BoxStyle;
pub use color::Color;
pub use keyval::KeyVal;
pub use list::{List, ListMarker};
pub use panel::Panel;
pub use progress::Progress;
pub use rule::Rule;
pub use spinner::Spinner;
pub use style::Style;
pub use table::{Align, Table};
pub use text::Text;
pub use tree::Tree;

// ── Color policy ─────────────────────────────────────────────────────────────────

/// Color output policy. Corresponds to the value of `--color=<when>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorChoice {
    /// Follows TTY/environment-variable detection (default).
    #[default]
    Auto,
    /// Always use color.
    Always,
    /// Never use color.
    Never,
}

static COLOR_CHOICE: AtomicU8 = AtomicU8::new(0);
static NO_COLOR_ENV: RwLock<Option<String>> = RwLock::new(None);

/// Sets the global color override. `--no-color` / `--color=<when>` use this value.
pub fn set_color_choice(choice: ColorChoice) {
    let raw = match choice {
        ColorChoice::Auto => 0,
        ColorChoice::Always => 1,
        ColorChoice::Never => 2,
    };
    COLOR_CHOICE.store(raw, Ordering::Relaxed);
}

/// Resets the global color override to [`ColorChoice::Auto`].
pub fn reset_color_choice() {
    set_color_choice(ColorChoice::Auto);
}

/// The current global color override.
pub fn color_choice() -> ColorChoice {
    match COLOR_CHOICE.load(Ordering::Relaxed) {
        1 => ColorChoice::Always,
        2 => ColorChoice::Never,
        _ => ColorChoice::Auto,
    }
}

/// Registers the name of the app-specific color-disabling environment variable (e.g. `Some("MYAPP_NO_COLOR")`).
pub fn set_no_color_env(name: Option<&str>) {
    *NO_COLOR_ENV.write().unwrap() = name.map(str::to_owned);
}

fn app_no_color_set() -> bool {
    let guard = NO_COLOR_ENV.read().unwrap();
    guard
        .as_deref()
        .and_then(std::env::var_os)
        .is_some_and(|v| !v.is_empty())
}

/// Inputs needed to decide color usage. Split out so it can be tested with the pure function [`should_use_color`].
#[derive(Debug, Clone, Default)]
pub struct ColorEnv {
    pub choice: ColorChoice,
    pub is_terminal: bool,
    pub no_color: Option<String>,
    pub force_color: Option<String>,
    pub term: Option<String>,
    pub app_no_color: bool,
}

impl ColorEnv {
    /// Collects from the process environment variables and the given TTY status.
    pub fn from_process(is_terminal: bool) -> Self {
        fn var(key: &str) -> Option<String> {
            std::env::var_os(key).map(|v| v.to_string_lossy().into_owned())
        }
        ColorEnv {
            choice: color_choice(),
            is_terminal,
            no_color: var("NO_COLOR"),
            force_color: var("FORCE_COLOR"),
            term: var("TERM"),
            app_no_color: app_no_color_set(),
        }
    }
}

/// Whether to use color according to clig.dev rules.
///
/// Priority: explicit override → `FORCE_COLOR` → `NO_COLOR` → `TERM=dumb`
/// → app-specific `*_NO_COLOR` → TTY status.
pub fn should_use_color(env: &ColorEnv) -> bool {
    match env.choice {
        ColorChoice::Never => return false,
        ColorChoice::Always => return true,
        ColorChoice::Auto => {}
    }
    let non_empty = |v: &Option<String>| v.as_deref().is_some_and(|s| !s.is_empty());
    if non_empty(&env.force_color) {
        return true;
    }
    if non_empty(&env.no_color) {
        return false;
    }
    if env.term.as_deref() == Some("dumb") {
        return false;
    }
    if env.app_no_color {
        return false;
    }
    env.is_terminal
}

/// Returns `true` if stdout supports ANSI styling.
pub fn stdout_is_styled() -> bool {
    should_use_color(&ColorEnv::from_process(std::io::stdout().is_terminal()))
}

/// Returns `true` if stderr supports ANSI styling.
pub fn stderr_is_styled() -> bool {
    should_use_color(&ColorEnv::from_process(std::io::stderr().is_terminal()))
}

/// `true` if stdin is an interactive terminal. Used to decide whether to use prompts.
pub fn stdin_is_terminal() -> bool {
    std::io::stdin().is_terminal()
}

/// `true` if stdout is a terminal (regardless of whether color is used).
///
/// Used when the TTY status is needed directly, such as for pagers and animations.
pub fn stdout_is_terminal() -> bool {
    std::io::stdout().is_terminal()
}

/// Terminal display width that counts CJK characters (Hangul/Han/Kana, etc.) as 2 columns and the rest as 1,
/// ignoring ANSI escape sequences.
pub fn display_width(s: &str) -> usize {
    let mut width = 0;
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b'
            && let Some(&next_c) = chars.peek()
        {
            if next_c == '[' {
                chars.next();
                while let Some(&seq_c) = chars.peek() {
                    chars.next();
                    if (seq_c as u32) >= 0x40 && (seq_c as u32) <= 0x7E {
                        break;
                    }
                }
                continue;
            } else if next_c == ']' || next_c == '_' || next_c == 'P' || next_c == '^' {
                chars.next();
                while let Some(seq_c) = chars.next() {
                    if seq_c == '\x07' {
                        break;
                    }
                    if seq_c == '\x1b' && chars.peek() == Some(&'\\') {
                        chars.next();
                        break;
                    }
                }
                continue;
            } else {
                chars.next();
                continue;
            }
        }

        width += if is_cjk(c) { 2 } else { 1 };
    }
    width
}

fn is_cjk(c: char) -> bool {
    let u = c as u32;
    matches!(u,
        // Hangul Jamo
        0x1100..=0x115F |
        // Hangul Jamo Extended-A
        0xA960..=0xA97C |
        // Hangul Syllables
        0xAC00..=0xD7AF |
        // Hangul Jamo Extended-B
        0xD7B0..=0xD7FF |
        // CJK Radicals Supplement / Kangxi Radicals
        0x2E80..=0x303E |
        // Hiragana
        0x3040..=0x309F |
        // Katakana
        0x30A0..=0x30FF |
        // Bopomofo
        0x3100..=0x312F |
        // Hangul Compatibility Jamo
        0x3130..=0x318F |
        // Kanbun / CJK Strokes / Enclosed CJK
        0x3190..=0x31FF |
        // CJK Compatibility
        0x3200..=0x33FF |
        // CJK Unified Extension A
        0x3400..=0x4DBF |
        // CJK Unified Ideographs
        0x4E00..=0x9FFF |
        // Yi
        0xA000..=0xA4CF |
        // CJK Compatibility Ideographs
        0xF900..=0xFAFF |
        // Vertical Forms / CJK Compatibility Forms
        0xFE10..=0xFE6F |
        // Fullwidth Forms
        0xFF01..=0xFF60 |
        0xFFE0..=0xFFE6 |
        0x2600..=0x27BF |
        0x1F300..=0x1FAFF |
        0x1B000..=0x1B12F |
        0x20000..=0x2FA1F |
        0x30000..=0x3134F
    )
}

// ── Convenience output helpers ───────────────────────────────────────────────────────────

/// Prints a success message to stdout with a green **✓** prefix.
pub fn print_success(msg: &str) {
    let styled = stdout_is_styled();
    let icon = Style::new().fg(Color::Green).bold().apply("✓", styled);
    println!("{} {}", icon, msg);
}

/// Prints an error message to stderr with a red **✗** prefix.
pub fn print_error(msg: &str) {
    let styled = stderr_is_styled();
    let icon = Style::new().fg(Color::Red).bold().apply("✗", styled);
    eprintln!("{} {}", icon, msg);
}

/// Prints a warning message to stderr with a yellow **⚠** prefix.
pub fn print_warning(msg: &str) {
    let styled = stderr_is_styled();
    let icon = Style::new().fg(Color::Yellow).bold().apply("⚠", styled);
    eprintln!("{} {}", icon, msg);
}

/// Prints an info message to stderr with a cyan **ℹ** prefix.
pub fn print_info(msg: &str) {
    let styled = stderr_is_styled();
    let icon = Style::new().fg(Color::Cyan).bold().apply("ℹ", styled);
    eprintln!("{} {}", icon, msg);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(is_terminal: bool) -> ColorEnv {
        ColorEnv {
            is_terminal,
            ..ColorEnv::default()
        }
    }

    #[test]
    fn stdout_is_styled_returns_bool() {
        let _ = stdout_is_styled();
        let _ = stderr_is_styled();
    }

    #[test]
    fn tty_enables_color_by_default() {
        assert!(should_use_color(&env(true)));
        assert!(!should_use_color(&env(false)));
    }

    #[test]
    fn empty_no_color_does_not_disable() {
        let mut e = env(true);
        e.no_color = Some(String::new());
        assert!(should_use_color(&e));

        e.no_color = Some("1".to_owned());
        assert!(!should_use_color(&e));
    }

    #[test]
    fn empty_force_color_does_not_enable() {
        let mut e = env(false);
        e.force_color = Some(String::new());
        assert!(!should_use_color(&e));

        e.force_color = Some("1".to_owned());
        assert!(should_use_color(&e));
    }

    #[test]
    fn force_color_beats_no_color() {
        let mut e = env(false);
        e.force_color = Some("1".to_owned());
        e.no_color = Some("1".to_owned());
        assert!(should_use_color(&e));
    }

    #[test]
    fn dumb_terminal_disables_color() {
        let mut e = env(true);
        e.term = Some("dumb".to_owned());
        assert!(!should_use_color(&e));
    }

    #[test]
    fn app_no_color_disables_color() {
        let mut e = env(true);
        e.app_no_color = true;
        assert!(!should_use_color(&e));
    }

    #[test]
    fn explicit_choice_overrides_everything() {
        let mut e = env(false);
        e.no_color = Some("1".to_owned());
        e.choice = ColorChoice::Always;
        assert!(should_use_color(&e));

        let mut e2 = env(true);
        e2.force_color = Some("1".to_owned());
        e2.choice = ColorChoice::Never;
        assert!(!should_use_color(&e2));
    }

    #[test]
    fn display_width_ignores_ansi_escapes() {
        assert_eq!(display_width("\x1b[31mhello\x1b[0m"), 5);
        assert_eq!(display_width("\x1b[1;32m한글\x1b[0m"), 4);
        assert_eq!(display_width("\x1b[38;2;255;0;0mRGB\x1b[0m test"), 8);
        assert_eq!(display_width("plain text"), 10);
        assert_eq!(display_width("\x1b[32m🚀\x1b[0m 한글 wrcli"), 13);
    }
}

//! 풍부한 터미널 스타일링 — 색상, 텍스트 속성, 테이블, 패널, 구분선.
//!
//! Python의 [rich](https://github.com/Textualize/rich) 라이브러리에서 영감을 받음.
//!
//! 스타일링은 다음 경우 자동으로 비활성화됨:
//! - `NO_COLOR` 환경변수가 설정된 경우
//! - 출력 스트림이 터미널에 연결되지 않은 경우 (파이프 등)
//!
//! # Quick start
//!
//! ```
//! use wrcli::style::{Color, Style, Panel, Table, Rule, Tree, Text, Progress, Align};
//!
//! // 스타일 텍스트
//! let s = Style::new().fg(Color::Green).bold();
//! let text = s.apply("Hello, World!", false);
//! assert_eq!(text, "Hello, World!");
//!
//! // 테이블
//! let table = Table::new()
//!     .headers(["Name", "Version"])
//!     .row(["wrcli", "0.1.0"]);
//! let rendered = table.render(false);
//! assert!(rendered.contains("wrcli"));
//!
//! // 패널
//! let panel = Panel::new("Content here").title("Info");
//! let rendered = panel.render(false);
//! assert!(rendered.contains("Content here"));
//!
//! // 구분선
//! let rule = Rule::new().title("Section");
//! let rendered = rule.render(false);
//! assert!(rendered.contains("Section"));
//! ```

use std::io::IsTerminal;

mod color;
mod panel;
mod progress;
mod rule;
#[allow(clippy::module_inception)]
mod style;
mod table;
mod text;
mod tree;

pub use color::Color;
pub use panel::Panel;
pub use progress::Progress;
pub use rule::Rule;
pub use style::Style;
pub use table::{Align, Table};
pub use text::Text;
pub use tree::Tree;

// ── TTY 감지 ─────────────────────────────────────────────────────────────────

/// stdout이 ANSI 스타일링을 지원하면 `true` 반환.
///
/// `NO_COLOR` 환경변수가 설정되거나 stdout이 터미널이 아닌 경우 `false`.
pub fn stdout_is_styled() -> bool {
    if std::env::var_os("NO_COLOR").is_some() {
        return false;
    }
    std::io::stdout().is_terminal()
}

/// stderr가 ANSI 스타일링을 지원하면 `true` 반환.
pub fn stderr_is_styled() -> bool {
    if std::env::var_os("NO_COLOR").is_some() {
        return false;
    }
    std::io::stderr().is_terminal()
}

/// CJK 문자(한글/한자/가나 등)를 2칸, 나머지를 1칸으로 계산하는 터미널 표시폭.
pub fn display_width(s: &str) -> usize {
    s.chars().map(|c| if is_cjk(c) { 2 } else { 1 }).sum()
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
        // CJK Extension B ~ H
        0x1B000..=0x1B12F |
        0x20000..=0x2FA1F |
        0x30000..=0x3134F
    )
}

// ── 편의 출력 헬퍼 ───────────────────────────────────────────────────────────

/// 녹색 **✓** 접두사를 붙여 성공 메시지를 stdout에 출력.
pub fn print_success(msg: &str) {
    let styled = stdout_is_styled();
    let icon = Style::new().fg(Color::Green).bold().apply("✓", styled);
    println!("{} {}", icon, msg);
}

/// 빨간색 **✗** 접두사를 붙여 에러 메시지를 stderr에 출력.
pub fn print_error(msg: &str) {
    let styled = stderr_is_styled();
    let icon = Style::new().fg(Color::Red).bold().apply("✗", styled);
    eprintln!("{} {}", icon, msg);
}

/// 노란색 **⚠** 접두사를 붙여 경고 메시지를 stdout에 출력.
pub fn print_warning(msg: &str) {
    let styled = stdout_is_styled();
    let icon = Style::new().fg(Color::Yellow).bold().apply("⚠", styled);
    println!("{} {}", icon, msg);
}

/// 청록색 **ℹ** 접두사를 붙여 정보 메시지를 stdout에 출력.
pub fn print_info(msg: &str) {
    let styled = stdout_is_styled();
    let icon = Style::new().fg(Color::Cyan).bold().apply("ℹ", styled);
    println!("{} {}", icon, msg);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stdout_is_styled_returns_bool() {
        let _ = stdout_is_styled();
        let _ = stderr_is_styled();
    }
}

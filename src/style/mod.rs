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
//! use wrcli::style::{Color, Style, Panel, Table, Rule, Align};
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
#[allow(clippy::module_inception)]
mod style;
mod table;
mod panel;
mod rule;

pub use color::Color;
pub use style::Style;
pub use table::{Align, Table};
pub use panel::Panel;
pub use rule::Rule;

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

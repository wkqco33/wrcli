//! 풍부한 터미널 스타일링 — 색상, 텍스트 속성, 테이블, 패널, 구분선.
//!
//! Python의 [rich](https://github.com/Textualize/rich) 라이브러리에서 영감을 받음.
//!
//! 스타일링은 다음 경우 자동으로 비활성화됨:
//! - `NO_COLOR`이 비어 있지 않게 설정된 경우
//! - `TERM=dumb`인 경우
//! - 앱 전용 `<APP>_NO_COLOR`(예: `MYAPP_NO_COLOR`)가 설정된 경우
//! - 출력 스트림이 터미널에 연결되지 않은 경우 (파이프 등)
//!
//! `FORCE_COLOR`(비어 있지 않음)는 감지 로직을 무시하고 색상을 켜며,
//! `set_color_choice`로 설정한 전역 override(`--no-color` / `--color=<when>`)가
//! 가장 높은 우선순위를 가진다.
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
use std::sync::RwLock;
use std::sync::atomic::{AtomicU8, Ordering};

mod color;
pub mod pager;
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

// ── 색상 정책 ─────────────────────────────────────────────────────────────────

/// 색상 출력 정책. `--color=<when>`의 값에 대응한다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorChoice {
    /// TTY·환경변수 감지에 따른다 (기본).
    #[default]
    Auto,
    /// 항상 색상을 사용한다.
    Always,
    /// 색상을 사용하지 않는다.
    Never,
}

static COLOR_CHOICE: AtomicU8 = AtomicU8::new(0);
static NO_COLOR_ENV: RwLock<Option<String>> = RwLock::new(None);

/// 전역 색상 override를 설정한다. `--no-color` / `--color=<when>`이 이 값을 쓴다.
pub fn set_color_choice(choice: ColorChoice) {
    let raw = match choice {
        ColorChoice::Auto => 0,
        ColorChoice::Always => 1,
        ColorChoice::Never => 2,
    };
    COLOR_CHOICE.store(raw, Ordering::Relaxed);
}

/// 전역 색상 override를 [`ColorChoice::Auto`]로 되돌린다.
pub fn reset_color_choice() {
    set_color_choice(ColorChoice::Auto);
}

/// 현재 전역 색상 override.
pub fn color_choice() -> ColorChoice {
    match COLOR_CHOICE.load(Ordering::Relaxed) {
        1 => ColorChoice::Always,
        2 => ColorChoice::Never,
        _ => ColorChoice::Auto,
    }
}

/// 앱 전용 색상 비활성화 환경변수 이름 등록 (예: `Some("MYAPP_NO_COLOR")`).
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

/// 색상 판정에 필요한 입력. 순수 함수 [`should_use_color`]로 테스트 가능하게 분리.
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
    /// 프로세스 환경변수와 주어진 TTY 여부로부터 수집.
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

/// clig.dev 규칙에 따른 색상 사용 여부.
///
/// 우선순위: 명시적 override → `FORCE_COLOR` → `NO_COLOR` → `TERM=dumb`
/// → 앱 전용 `*_NO_COLOR` → TTY 여부.
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

/// stdout이 ANSI 스타일링을 지원하면 `true` 반환.
pub fn stdout_is_styled() -> bool {
    should_use_color(&ColorEnv::from_process(std::io::stdout().is_terminal()))
}

/// stderr가 ANSI 스타일링을 지원하면 `true` 반환.
pub fn stderr_is_styled() -> bool {
    should_use_color(&ColorEnv::from_process(std::io::stderr().is_terminal()))
}

/// stdin이 대화형 터미널이면 `true`. 프롬프트 사용 여부 판단에 쓴다.
pub fn stdin_is_terminal() -> bool {
    std::io::stdin().is_terminal()
}

/// stdout이 터미널이면 `true` (색상 사용 여부와 무관).
///
/// 페이저·애니메이션처럼 TTY 여부가 직접 필요한 경우에 쓴다.
pub fn stdout_is_terminal() -> bool {
    std::io::stdout().is_terminal()
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

/// 노란색 **⚠** 접두사를 붙여 경고 메시지를 stderr에 출력.
pub fn print_warning(msg: &str) {
    let styled = stderr_is_styled();
    let icon = Style::new().fg(Color::Yellow).bold().apply("⚠", styled);
    eprintln!("{} {}", icon, msg);
}

/// 청록색 **ℹ** 접두사를 붙여 정보 메시지를 stderr에 출력.
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
}

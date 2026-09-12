//! clig.dev 색상 비활성화 규칙 테스트.
//!
//! - `FORCE_COLOR`(비어 있지 않음)는 TTY 감지를 무시하고 색상을 켠다.
//! - `NO_COLOR`(비어 있지 않음)는 색상을 끈다.
//! - `TERM=dumb`은 색상을 끈다.
//! - 앱 전용 `<APP>_NO_COLOR`는 색상을 끈다.
//! - `--no-color` / `--color=<when>`은 전역 override로 최우선 적용된다.

mod common;

use common::EnvGuard;
use wrcli::style::{
    ColorChoice, reset_color_choice, set_color_choice, set_no_color_env, stderr_is_styled,
    stdin_is_terminal, stdout_is_styled,
};

#[test]
fn force_color_enables_color_without_tty() {
    let _env = EnvGuard::set("FORCE_COLOR", "1");
    assert!(stdout_is_styled());
    assert!(stderr_is_styled());
}

#[test]
fn color_choice_override_has_highest_precedence() {
    // 전역 override를 다루므로 env lock으로 다른 env 테스트와 직렬화한다.
    let _env = EnvGuard::set_many(&[("FORCE_COLOR", "1"), ("NO_COLOR", "1"), ("TERM", "dumb")]);

    set_color_choice(ColorChoice::Never);
    assert!(!stdout_is_styled(), "Never must win over FORCE_COLOR");

    set_color_choice(ColorChoice::Always);
    assert!(
        stdout_is_styled(),
        "Always must win over NO_COLOR/TERM=dumb"
    );
    assert!(stderr_is_styled());

    reset_color_choice();
    assert!(stdout_is_styled(), "Auto falls back to FORCE_COLOR");
}

#[test]
fn app_no_color_env_is_respected() {
    let _env = EnvGuard::set_many(&[("FORCE_COLOR", ""), ("MYAPP_NO_COLOR", "1")]);
    set_no_color_env(Some("MYAPP_NO_COLOR"));
    set_color_choice(ColorChoice::Always);
    // Always override는 앱 전용 env보다 우선한다.
    assert!(stdout_is_styled());
    set_color_choice(ColorChoice::Auto);
    // Auto에서는 앱 전용 env가 색상을 끈다.
    assert!(!stdout_is_styled());
    reset_color_choice();
    set_no_color_env(None);
}

#[test]
fn stdin_is_terminal_returns_bool() {
    let _ = stdin_is_terminal();
}

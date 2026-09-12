//! Tests for the clig.dev color-disabling rules.
//!
//! - `FORCE_COLOR` (non-empty) ignores TTY detection and turns color on.
//! - `NO_COLOR` (non-empty) turns color off.
//! - `TERM=dumb` turns color off.
//! - The app-specific `<APP>_NO_COLOR` turns color off.
//! - `--no-color` / `--color=<when>` are global overrides and take the highest precedence.

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
    // Handles a global override, so it serializes with other env tests via the env lock.
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
    // The Always override takes precedence over the app-specific env.
    assert!(stdout_is_styled());
    set_color_choice(ColorChoice::Auto);
    // In Auto, the app-specific env turns color off.
    assert!(!stdout_is_styled());
    reset_color_choice();
    set_no_color_env(None);
}

#[test]
fn stdin_is_terminal_returns_bool() {
    let _ = stdin_is_terminal();
}

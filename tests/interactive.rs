//! clig.dev Interactivity 규칙 테스트.
//!
//! 테스트 프로세스의 stdin은 TTY가 아니므로, 프롬프트 없이 명확한 오류를 내는
//! 경로가 결정적으로 검증된다.

mod common;

use common::args;
use wrcli::{Command, Flag, FlagValue, WrCliError};

#[test]
fn confirm_with_force_flag_returns_true() {
    Command::new("app")
        .standard_flags()
        .on_run_e(|ctx| {
            assert!(ctx.confirm("Delete everything?")?);
            Ok(())
        })
        .execute_with(args("--force"))
        .unwrap();
}

#[test]
fn confirm_without_tty_errors_instead_of_hanging() {
    let err = Command::new("app")
        .on_run_e(|ctx| {
            ctx.confirm("Delete everything?")?;
            Ok(())
        })
        .execute_with(args(""))
        .unwrap_err();
    assert!(
        matches!(err, WrCliError::InteractiveInputRequired { .. }),
        "got {err:?}"
    );
    assert!(err.to_string().contains("--force"), "hint missing: {err}");
}

#[test]
fn no_input_flag_blocks_confirmation() {
    let err = Command::new("app")
        .standard_flags()
        .on_run_e(|ctx| {
            ctx.confirm("Delete everything?")?;
            Ok(())
        })
        .execute_with(args("--no-input"))
        .unwrap_err();
    assert!(matches!(err, WrCliError::InteractiveInputRequired { .. }));
}

#[test]
fn confirm_severe_accepts_matching_confirm_flag() {
    Command::new("app")
        .standard_flags()
        .on_run_e(|ctx| {
            assert!(ctx.confirm_severe("myapp")?);
            Ok(())
        })
        .execute_with(args("--confirm myapp"))
        .unwrap();
}

#[test]
fn confirm_severe_rejects_wrong_confirm_flag() {
    let err = Command::new("app")
        .standard_flags()
        .on_run_e(|ctx| {
            ctx.confirm_severe("myapp")?;
            Ok(())
        })
        .execute_with(args("--confirm other"))
        .unwrap_err();
    assert!(matches!(err, WrCliError::ConfirmationFailed { .. }));
}

#[test]
fn confirm_severe_without_tty_errors() {
    let err = Command::new("app")
        .standard_flags()
        .on_run_e(|ctx| {
            ctx.confirm_severe("myapp")?;
            Ok(())
        })
        .execute_with(args(""))
        .unwrap_err();
    assert!(matches!(err, WrCliError::InteractiveInputRequired { .. }));
}

#[test]
fn is_interactive_is_false_without_tty() {
    Command::new("app")
        .standard_flags()
        .on_run_e(|ctx| {
            assert!(!ctx.is_interactive());
            Ok(())
        })
        .execute_with(args(""))
        .unwrap();
}

#[cfg(unix)]
#[test]
fn prompt_password_without_tty_errors() {
    let err = Command::new("app")
        .on_run_e(|ctx| {
            ctx.prompt_password("Password: ")?;
            Ok(())
        })
        .execute_with(args(""))
        .unwrap_err();
    assert!(matches!(err, WrCliError::InteractiveInputRequired { .. }));
}

#[test]
fn optional_value_none_resets_to_empty() {
    let out = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
    let out2 = out.clone();
    Command::new("app")
        .flag(
            Flag::new(
                "config",
                FlagValue::String("/etc/app.toml".to_owned()),
                "config path",
            )
            .optional_value(),
        )
        .on_run(move |ctx| {
            *out2.lock().unwrap() = ctx.flags.get_string("config").unwrap_or("").to_owned();
        })
        .execute_with(args("--config none"))
        .unwrap();
    assert_eq!(*out.lock().unwrap(), "");
}

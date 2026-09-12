//! clig.dev Help 규약 테스트: 내장 `help` 서브커맨드와 러너 없는 커맨드 정책.

mod common;

use common::args;
use std::sync::{Arc, Mutex};
use wrcli::{Command, Flag, FlagValue, WrCliError};

#[test]
fn help_subcommand_shows_root_help() {
    Command::new("app")
        .short("My app")
        .subcommand(Command::new("serve").short("serve it"))
        .execute_with(args("help"))
        .unwrap();
}

#[test]
fn help_subcommand_works_for_pure_leaf() {
    Command::new("app")
        .on_run(|_| {})
        .execute_with(args("help"))
        .unwrap();
}

#[test]
fn help_subcommand_resolves_nested_path() {
    Command::new("app")
        .subcommand(
            Command::new("config")
                .short("config ops")
                .subcommand(Command::new("get").short("get a value").on_run(|_| {})),
        )
        .execute_with(args("help config get"))
        .unwrap();
}

#[test]
fn help_subcommand_accepts_alias() {
    Command::new("app")
        .subcommand(
            Command::new("serve")
                .alias("srv")
                .short("serve it")
                .on_run(|_| {}),
        )
        .execute_with(args("help srv"))
        .unwrap();
}

#[test]
fn help_subcommand_reports_unknown_target() {
    let err = Command::new("app")
        .subcommand(Command::new("serve").short("serve it"))
        .execute_with(args("help ghost"))
        .unwrap_err();
    assert!(matches!(err, WrCliError::UnknownSubcommand { name, .. } if name == "ghost"));
}

#[test]
fn help_subcommand_suggests_close_target() {
    let err = Command::new("app")
        .subcommand(Command::new("serve").short("serve it"))
        .execute_with(args("help serv"))
        .unwrap_err();
    assert!(err.to_string().contains("serve"), "msg: {err}");
}

#[test]
fn user_defined_help_is_not_shadowed() {
    let called = Arc::new(Mutex::new(false));
    let called2 = called.clone();
    Command::new("app")
        .subcommand(Command::new("help").on_run(move |_| *called2.lock().unwrap() = true))
        .execute_with(args("help"))
        .unwrap();
    assert!(*called.lock().unwrap(), "user command must win");
}

#[test]
fn parent_with_subcommands_and_no_runner_is_ok() {
    Command::new("app")
        .subcommand(Command::new("serve").on_run(|_| {}))
        .execute_with(args(""))
        .unwrap();
}

#[test]
fn leaf_without_runner_still_errors() {
    let err = Command::new("app").execute_with(args("")).unwrap_err();
    assert!(matches!(err, WrCliError::CommandHasNoRunner(_)));
}

#[test]
fn help_on_missing_runner_opt_in() {
    Command::new("app")
        .short("My app")
        .help_on_missing_runner()
        .execute_with(args(""))
        .unwrap();
}

#[test]
fn examples_are_recorded() {
    let cmd = Command::new("app")
        .example("app greet Alice")
        .example("app greet Bob");
    assert_eq!(cmd.examples().len(), 2);
}

#[test]
fn invalid_flag_value_mentions_help() {
    let err = Command::new("app")
        .flag(Flag::new("count", FlagValue::Int(0), "count"))
        .on_run(|_| {})
        .execute_with(args("--count nope"))
        .unwrap_err();
    assert!(matches!(err, WrCliError::InvalidFlagValue { .. }));
    assert!(
        err.to_string().contains("--help"),
        "usage hint missing: {err}"
    );
}

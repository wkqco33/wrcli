//! 서브커맨드 라우팅 및 persistent flag 통합 테스트.

mod common;
use common::args;

use std::sync::{Arc, Mutex};
use wrcli::args::exact_args;
use wrcli::{Command, Flag, FlagValue, WrCliError};

#[test]
fn subcommand_basic_dispatch() {
    let ran = Arc::new(Mutex::new(false));
    let ran2 = ran.clone();
    Command::new("app")
        .subcommand(Command::new("sub").on_run(move |_| *ran2.lock().unwrap() = true))
        .execute_with(args("sub"))
        .unwrap();
    assert!(*ran.lock().unwrap());
}

#[test]
fn subcommand_nested_two_levels() {
    let path = Arc::new(Mutex::new(vec![]));
    let path2 = path.clone();
    Command::new("app")
        .subcommand(
            Command::new("config").subcommand(
                Command::new("get")
                    .args(exact_args(1))
                    .on_run(move |ctx| *path2.lock().unwrap() = ctx.command_path.clone()),
            ),
        )
        .execute_with(args("config get mykey"))
        .unwrap();
    assert_eq!(*path.lock().unwrap(), vec!["app", "config", "get"]);
}

#[test]
fn subcommand_alias() {
    let ran = Arc::new(Mutex::new(false));
    let ran2 = ran.clone();
    Command::new("app")
        .subcommand(
            Command::new("serve")
                .alias("server")
                .alias("s")
                .on_run(move |_| *ran2.lock().unwrap() = true),
        )
        .execute_with(args("s"))
        .unwrap();
    assert!(*ran.lock().unwrap());
}

#[test]
fn subcommand_with_flags_before_name() {
    let flag_val = Arc::new(Mutex::new(false));
    let flag2 = flag_val.clone();
    Command::new("app")
        .persistent_flag(Flag::new("verbose", FlagValue::Bool(false), "verbose").short('v'))
        .subcommand(Command::new("sub").on_run(move |ctx| {
            *flag2.lock().unwrap() = ctx.flags.get_bool("verbose").unwrap_or(false)
        }))
        .execute_with(args("--verbose sub"))
        .unwrap();
    assert!(*flag_val.lock().unwrap());
}

#[test]
fn unknown_subcommand_returns_error() {
    let err = Command::new("app")
        .subcommand(Command::new("sub").on_run(|_| {}))
        .execute_with(args("ghost"))
        .unwrap_err();
    assert!(matches!(err, WrCliError::UnknownSubcommand { .. }));
}

#[test]
fn persistent_flag_visible_in_leaf() {
    let val = Arc::new(Mutex::new(false));
    let val2 = val.clone();
    Command::new("app")
        .persistent_flag(Flag::new("debug", FlagValue::Bool(false), "debug").short('d'))
        .subcommand(
            Command::new("sub").subcommand(Command::new("deep").on_run(move |ctx| {
                *val2.lock().unwrap() = ctx.flags.get_bool("debug").unwrap_or(false)
            })),
        )
        .execute_with(args("sub deep --debug"))
        .unwrap();
    assert!(*val.lock().unwrap());
}

#[test]
fn local_flag_not_visible_in_sibling_subcommand() {
    let err = Command::new("app")
        .subcommand(
            Command::new("a")
                .flag(Flag::new("local", FlagValue::Bool(false), "x"))
                .on_run(|_| {}),
        )
        .subcommand(Command::new("b").on_run(|_| {}))
        .execute_with(args("b --local"))
        .unwrap_err();
    assert!(matches!(err, WrCliError::UnknownFlag { .. }));
}

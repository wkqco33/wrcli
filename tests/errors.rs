//! 에러 핸들링 및 help/version 스모크 테스트.

mod common;
use common::args;

use wrcli::{Command, Flag, FlagValue, WrCliError};

#[test]
fn user_error_from_run_e() {
    use std::fmt;
    #[derive(Debug)]
    struct MyErr;
    impl fmt::Display for MyErr {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "my error")
        }
    }
    impl std::error::Error for MyErr {}

    let err = Command::new("app")
        .on_run_e(|_| Err(WrCliError::user(MyErr)))
        .execute_with(args(""))
        .unwrap_err();
    assert!(err.to_string().contains("my error"));
}

#[test]
fn from_io_error() {
    let io_err: WrCliError =
        std::io::Error::new(std::io::ErrorKind::NotFound, "file missing").into();
    assert!(matches!(io_err, WrCliError::Io(_)));
}

#[test]
fn help_flag_no_panic() {
    Command::new("app")
        .short("My app")
        .flag(Flag::new("verbose", FlagValue::Bool(false), "verbose").short('v'))
        .subcommand(Command::new("sub").short("A subcommand").on_run(|_| {}))
        .on_run(|_| {})
        .execute_with(args("--help"))
        .unwrap();
}

#[test]
fn help_subcommand_flag_no_panic() {
    Command::new("app")
        .subcommand(
            Command::new("sub")
                .short("A subcommand")
                .flag(Flag::new("count", FlagValue::Int(0), "count").short('c'))
                .on_run(|_| {}),
        )
        .execute_with(args("sub --help"))
        .unwrap();
}

#[test]
fn version_flag() {
    Command::new("app")
        .version("2.3.4")
        .on_run(|_| {})
        .execute_with(args("--version"))
        .unwrap();
}

//! 위치 인자 수집 및 ArgValidator 통합 테스트.

mod common;
use common::args;

use std::sync::{Arc, Mutex};
use wrcli::args::{
    arbitrary_args, exact_args, maximum_n_args, minimum_n_args, no_args, range_args, valid_args,
};
use wrcli::{Command, Flag, FlagValue, WrCliError};

#[test]
fn positional_args_collected() {
    let out: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
    let out2 = out.clone();
    Command::new("app")
        .on_run(move |ctx| *out2.lock().unwrap() = ctx.args.clone())
        .execute_with(args("foo bar baz"))
        .unwrap();
    assert_eq!(*out.lock().unwrap(), vec!["foo", "bar", "baz"]);
}

#[test]
fn positional_interleaved_with_flags() {
    let (name, rest): (Arc<Mutex<String>>, Arc<Mutex<Vec<String>>>) = (
        Arc::new(Mutex::new(String::new())),
        Arc::new(Mutex::new(vec![])),
    );
    let (n2, r2) = (name.clone(), rest.clone());
    Command::new("app")
        .flag(Flag::new("name", FlagValue::String(String::new()), "name").short('n'))
        .on_run(move |ctx| {
            *n2.lock().unwrap() = ctx.flags.get_string("name").unwrap().to_owned();
            *r2.lock().unwrap() = ctx.args.clone();
        })
        .execute_with(args("file1 --name Alice file2"))
        .unwrap();
    assert_eq!(*name.lock().unwrap(), "Alice");
    assert_eq!(*rest.lock().unwrap(), vec!["file1", "file2"]);
}

#[test]
fn arg_validator_no_args_pass() {
    Command::new("app")
        .args(no_args())
        .on_run(|_| {})
        .execute_with(args(""))
        .unwrap();
}

#[test]
fn arg_validator_no_args_fail() {
    let err = Command::new("app")
        .args(no_args())
        .on_run(|_| {})
        .execute_with(args("extra"))
        .unwrap_err();
    assert!(matches!(err, WrCliError::ArgValidationFailed(_)));
}

#[test]
fn arg_validator_exact_args() {
    Command::new("app")
        .args(exact_args(2))
        .on_run(|_| {})
        .execute_with(args("a b"))
        .unwrap();

    let err = Command::new("app")
        .args(exact_args(2))
        .on_run(|_| {})
        .execute_with(args("a"))
        .unwrap_err();
    assert!(matches!(err, WrCliError::ArgValidationFailed(_)));
}

#[test]
fn arg_validator_minimum_n_args() {
    Command::new("app")
        .args(minimum_n_args(1))
        .on_run(|_| {})
        .execute_with(args("a b"))
        .unwrap();

    let err = Command::new("app")
        .args(minimum_n_args(2))
        .on_run(|_| {})
        .execute_with(args("a"))
        .unwrap_err();
    assert!(matches!(err, WrCliError::ArgValidationFailed(_)));
}

#[test]
fn arg_validator_maximum_n_args() {
    Command::new("app")
        .args(maximum_n_args(3))
        .on_run(|_| {})
        .execute_with(args("a b c"))
        .unwrap();

    let err = Command::new("app")
        .args(maximum_n_args(2))
        .on_run(|_| {})
        .execute_with(args("a b c"))
        .unwrap_err();
    assert!(matches!(err, WrCliError::ArgValidationFailed(_)));
}

#[test]
fn arg_validator_arbitrary_args() {
    Command::new("app")
        .args(arbitrary_args())
        .on_run(|_| {})
        .execute_with(args("a b c d e"))
        .unwrap();
}

#[test]
fn arg_validator_range_args_pass() {
    Command::new("app")
        .args(range_args(2, 4))
        .on_run(|_| {})
        .execute_with(args("a b"))
        .unwrap();
    Command::new("app")
        .args(range_args(2, 4))
        .on_run(|_| {})
        .execute_with(args("a b c d"))
        .unwrap();
}

#[test]
fn arg_validator_range_args_fail() {
    let err = Command::new("app")
        .args(range_args(2, 4))
        .on_run(|_| {})
        .execute_with(args("a"))
        .unwrap_err();
    assert!(matches!(err, WrCliError::ArgValidationFailed(_)));

    let err = Command::new("app")
        .args(range_args(2, 4))
        .on_run(|_| {})
        .execute_with(args("a b c d e"))
        .unwrap_err();
    assert!(matches!(err, WrCliError::ArgValidationFailed(_)));
}

#[test]
fn arg_validator_valid_args_pass() {
    Command::new("app")
        .args(valid_args(vec!["foo".to_owned(), "bar".to_owned()]))
        .on_run(|_| {})
        .execute_with(args("foo bar"))
        .unwrap();
}

#[test]
fn arg_validator_valid_args_fail() {
    let err = Command::new("app")
        .args(valid_args(vec!["foo".to_owned()]))
        .on_run(|_| {})
        .execute_with(args("baz"))
        .unwrap_err();
    assert!(matches!(err, WrCliError::ArgValidationFailed(_)));
}

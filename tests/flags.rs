//! 플래그 파싱 통합 테스트.

mod common;
use common::args;

use std::sync::{Arc, Mutex};
use wrcli::{Command, Flag, FlagValue, WrCliError};

#[test]
#[should_panic(expected = "already registered")]
fn duplicate_short_flag_panics() {
    Command::new("app")
        .flag(Flag::new("verbose", FlagValue::Bool(false), "verbose").short('v'))
        .flag(Flag::new("version", FlagValue::Bool(false), "version").short('v'));
}

#[test]
#[should_panic(expected = "already registered")]
fn duplicate_flag_name_panics() {
    Command::new("app")
        .flag(Flag::new("name", FlagValue::String(String::new()), "name"))
        .flag(Flag::new("name", FlagValue::Int(0), "name again"));
}

#[test]
fn flag_long_space() {
    let out = Arc::new(Mutex::new(String::new()));
    let out2 = out.clone();
    Command::new("app")
        .flag(Flag::new("name", FlagValue::String(String::new()), "name"))
        .on_run(move |ctx| *out2.lock().unwrap() = ctx.flags.get_string("name").unwrap().to_owned())
        .execute_with(args("--name Alice"))
        .unwrap();
    assert_eq!(*out.lock().unwrap(), "Alice");
}

#[test]
fn flag_long_equals() {
    let out = Arc::new(Mutex::new(String::new()));
    let out2 = out.clone();
    Command::new("app")
        .flag(Flag::new("output", FlagValue::String(String::new()), "output"))
        .on_run(move |ctx| {
            *out2.lock().unwrap() = ctx.flags.get_string("output").unwrap().to_owned()
        })
        .execute_with(args("--output=result.txt"))
        .unwrap();
    assert_eq!(*out.lock().unwrap(), "result.txt");
}

#[test]
fn flag_short() {
    let out = Arc::new(Mutex::new(String::new()));
    let out2 = out.clone();
    Command::new("app")
        .flag(Flag::new("name", FlagValue::String(String::new()), "name").short('n'))
        .on_run(move |ctx| *out2.lock().unwrap() = ctx.flags.get_string("name").unwrap().to_owned())
        .execute_with(args("-n Bob"))
        .unwrap();
    assert_eq!(*out.lock().unwrap(), "Bob");
}

#[test]
fn flag_bool_implicit_true() {
    let out = Arc::new(Mutex::new(false));
    let out2 = out.clone();
    Command::new("app")
        .flag(Flag::new("verbose", FlagValue::Bool(false), "verbose").short('v'))
        .on_run(move |ctx| *out2.lock().unwrap() = ctx.flags.get_bool("verbose").unwrap())
        .execute_with(args("--verbose"))
        .unwrap();
    assert!(*out.lock().unwrap());
}

#[test]
fn flag_bool_explicit_false() {
    let out = Arc::new(Mutex::new(true));
    let out2 = out.clone();
    Command::new("app")
        .flag(Flag::new("verbose", FlagValue::Bool(true), "verbose"))
        .on_run(move |ctx| *out2.lock().unwrap() = ctx.flags.get_bool("verbose").unwrap())
        .execute_with(args("--verbose=false"))
        .unwrap();
    assert!(!*out.lock().unwrap());
}

#[test]
fn flag_combined_short_bools() {
    let (v, d, q) = (
        Arc::new(Mutex::new(false)),
        Arc::new(Mutex::new(false)),
        Arc::new(Mutex::new(false)),
    );
    let (v2, d2, q2) = (v.clone(), d.clone(), q.clone());
    Command::new("app")
        .flag(Flag::new("verbose", FlagValue::Bool(false), "v").short('v'))
        .flag(Flag::new("debug", FlagValue::Bool(false), "d").short('d'))
        .flag(Flag::new("quiet", FlagValue::Bool(false), "q").short('q'))
        .on_run(move |ctx| {
            *v2.lock().unwrap() = ctx.flags.get_bool("verbose").unwrap();
            *d2.lock().unwrap() = ctx.flags.get_bool("debug").unwrap();
            *q2.lock().unwrap() = ctx.flags.get_bool("quiet").unwrap();
        })
        .execute_with(args("-vdq"))
        .unwrap();
    assert!(*v.lock().unwrap());
    assert!(*d.lock().unwrap());
    assert!(*q.lock().unwrap());
}

#[test]
fn flag_int() {
    let out = Arc::new(Mutex::new(0i64));
    let out2 = out.clone();
    Command::new("app")
        .flag(Flag::new("count", FlagValue::Int(0), "count").short('c'))
        .on_run(move |ctx| *out2.lock().unwrap() = ctx.flags.get_int("count").unwrap())
        .execute_with(args("--count 99"))
        .unwrap();
    assert_eq!(*out.lock().unwrap(), 99);
}

#[test]
fn flag_float() {
    let out = Arc::new(Mutex::new(0f64));
    let out2 = out.clone();
    Command::new("app")
        .flag(Flag::new("ratio", FlagValue::Float(0.0), "ratio"))
        .on_run(move |ctx| *out2.lock().unwrap() = ctx.flags.get_float("ratio").unwrap())
        .execute_with(args("--ratio 2.5"))
        .unwrap();
    assert!(((*out.lock().unwrap()) - 2.5).abs() < 1e-10);
}

#[test]
fn flag_string_vec() {
    let out: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
    let out2 = out.clone();
    Command::new("app")
        .flag(Flag::new("tag", FlagValue::StringVec(vec![]), "tags"))
        .on_run(move |ctx| {
            *out2.lock().unwrap() = ctx.flags.get_string_vec("tag").unwrap().to_vec();
        })
        .execute_with(args("--tag alpha --tag beta --tag gamma"))
        .unwrap();
    assert_eq!(*out.lock().unwrap(), vec!["alpha", "beta", "gamma"]);
}

#[test]
fn flag_double_dash_sentinel() {
    let out: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
    let out2 = out.clone();
    Command::new("app")
        .on_run(move |ctx| *out2.lock().unwrap() = ctx.args.clone())
        .execute_with(args("-- --not-a-flag positional"))
        .unwrap();
    assert_eq!(*out.lock().unwrap(), vec!["--not-a-flag", "positional"]);
}

#[test]
fn flag_default_used_when_absent() {
    let out = Arc::new(Mutex::new(0i64));
    let out2 = out.clone();
    Command::new("app")
        .flag(Flag::new("port", FlagValue::Int(8080), "port"))
        .on_run(move |ctx| *out2.lock().unwrap() = ctx.flags.get_int("port").unwrap())
        .execute_with(args(""))
        .unwrap();
    assert_eq!(*out.lock().unwrap(), 8080);
}

#[test]
fn flag_required_missing_returns_error() {
    let err = Command::new("app")
        .flag(Flag::new("name", FlagValue::String(String::new()), "name").required())
        .on_run(|_| {})
        .execute_with(args(""))
        .unwrap_err();
    assert!(matches!(err, WrCliError::MissingRequiredFlag(n) if n == "name"));
}

#[test]
fn flag_unknown_returns_error() {
    let err = Command::new("app")
        .on_run(|_| {})
        .execute_with(args("--unknown-flag"))
        .unwrap_err();
    assert!(matches!(err, WrCliError::UnknownFlag { .. }));
}

#[test]
fn flag_invalid_int_returns_error() {
    let err = Command::new("app")
        .flag(Flag::new("port", FlagValue::Int(0), "port"))
        .on_run(|_| {})
        .execute_with(args("--port not-a-number"))
        .unwrap_err();
    assert!(matches!(err, WrCliError::InvalidFlagValue { .. }));
}

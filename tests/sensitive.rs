//! Verifies that sensitive flags do not expose their values.
//!
//! clig.dev: "Do not read secrets directly from flags." At the very least, keep the value from leaking
//! into error messages and help defaults.

mod common;

use common::args;
use std::sync::{Arc, Mutex};
use wrcli::{Command, Flag, FlagValue};

#[test]
fn sensitive_flag_value_is_still_usable() {
    let out = Arc::new(Mutex::new(String::new()));
    let out2 = out.clone();
    Command::new("app")
        .flag(Flag::new("password", FlagValue::String(String::new()), "secret").sensitive())
        .on_run(move |ctx| {
            *out2.lock().unwrap() = ctx.flags.get_string("password").unwrap_or("").to_owned();
        })
        .execute_with(args("--password hunter2"))
        .unwrap();
    assert_eq!(*out.lock().unwrap(), "hunter2");
}

#[test]
fn sensitive_value_never_appears_in_errors() {
    let err = Command::new("app")
        .flag(Flag::new("port", FlagValue::Int(0), "sensitive port").sensitive())
        .on_run(|_| {})
        .execute_with(args("--port hunter2"))
        .unwrap_err();
    let msg = err.to_string();
    assert!(!msg.contains("hunter2"), "secret leaked: {msg}");
    assert!(msg.contains("***"), "expected redaction: {msg}");
}

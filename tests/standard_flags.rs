//! clig.dev 표준 플래그(`-q/--quiet`, `--no-input`, `-f/--force`, `--no-color`,
//! `--plain`, `--json`)와 [`OutputFormat`] 테스트.

mod common;

use common::args;
use std::sync::{Arc, Mutex};
use wrcli::{Command, OutputFormat, WrCliError};

fn capture<T: Send + 'static>(
    cmd: Command,
    argv: &str,
    f: impl Fn(&wrcli::CommandContext) -> T + Send + Sync + 'static,
) -> T {
    let out = Arc::new(Mutex::new(None));
    let out2 = out.clone();
    cmd.on_run(move |ctx| {
        *out2.lock().unwrap() = Some(f(ctx));
    })
    .execute_with(args(argv))
    .unwrap();
    Arc::try_unwrap(out)
        .ok()
        .unwrap()
        .into_inner()
        .unwrap()
        .unwrap()
}

#[test]
fn standard_flags_expose_quiet_force_no_input() {
    let (q, f, n) = capture(
        Command::new("app").standard_flags(),
        "--quiet --force --no-input",
        |ctx| (ctx.is_quiet(), ctx.is_force(), ctx.no_input()),
    );
    assert_eq!((q, f, n), (true, true, true));
}

#[test]
fn quiet_short_flag_works() {
    let q = capture(Command::new("app").standard_flags(), "-q", |ctx| {
        ctx.is_quiet()
    });
    assert!(q);
}

#[test]
fn output_format_defaults_to_human() {
    let fmt = capture(Command::new("app").standard_flags(), "", |ctx| {
        ctx.output_format()
    });
    assert_eq!(fmt, OutputFormat::Human);
}

#[test]
fn output_format_plain_and_json() {
    let plain = capture(Command::new("app").standard_flags(), "--plain", |ctx| {
        ctx.output_format()
    });
    assert_eq!(plain, OutputFormat::Plain);
    let json = capture(Command::new("app").standard_flags(), "--json", |ctx| {
        ctx.output_format()
    });
    assert_eq!(json, OutputFormat::Json);
}

#[test]
fn plain_and_json_are_mutually_exclusive() {
    let err = Command::new("app")
        .standard_flags()
        .on_run(|_| {})
        .execute_with(args("--plain --json"))
        .unwrap_err();
    assert!(
        matches!(err, WrCliError::MutuallyExclusiveFlags { .. }),
        "got {err:?}"
    );
}

#[test]
fn standard_flag_names_are_not_registered_by_default() {
    let err = Command::new("app")
        .on_run(|_| {})
        .execute_with(args("--quiet"))
        .unwrap_err();
    assert!(matches!(err, WrCliError::UnknownFlag { .. }));
}

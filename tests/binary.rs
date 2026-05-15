//! Binary integration tests using assert_cmd.
//!
//! These tests spawn the `testapp` binary as a real process and verify
//! stdout, stderr, and exit codes — something library-direct tests cannot do.

use assert_cmd::Command;
use predicates::prelude::*;

fn app() -> Command {
    Command::cargo_bin("testapp").expect("testapp binary not found")
}

// ── greet subcommand ─────────────────────────────────────────────────────────

#[test]
fn greet_basic() {
    app()
        .args(["greet", "Alice"])
        .assert()
        .success()
        .stdout("Hello, Alice!\n");
}

#[test]
fn greet_upper_flag() {
    app()
        .args(["greet", "--upper", "Alice"])
        .assert()
        .success()
        .stdout("HELLO, ALICE!\n");
}

#[test]
fn greet_count_flag() {
    app()
        .args(["greet", "--count", "3", "Bob"])
        .assert()
        .success()
        .stdout("Hello, Bob!\nHello, Bob!\nHello, Bob!\n");
}

#[test]
fn greet_verbose_writes_to_stderr() {
    app()
        .args(["--verbose", "greet", "Carol"])
        .assert()
        .success()
        .stdout("Hello, Carol!\n")
        .stderr(predicate::str::contains("greeted Carol"));
}

#[test]
fn greet_missing_arg_fails() {
    app()
        .args(["greet"])
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

// ── echo subcommand ──────────────────────────────────────────────────────────

#[test]
fn echo_multiple_args() {
    app()
        .args(["echo", "foo", "bar", "baz"])
        .assert()
        .success()
        .stdout("foo bar baz\n");
}

#[test]
fn echo_no_args_prints_empty_line() {
    app().args(["echo"]).assert().success().stdout("\n");
}

// ── fail subcommand ──────────────────────────────────────────────────────────

#[test]
fn fail_exits_nonzero() {
    app()
        .args(["fail"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("failing with code 1"));
}

#[test]
fn fail_custom_message_contains_code() {
    app()
        .args(["fail", "--code", "42"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("failing with code 42"));
}

// ── tags subcommand (StringVec flag) ─────────────────────────────────────────

#[test]
fn tags_single() {
    app()
        .args(["tags", "--tag", "alpha"])
        .assert()
        .success()
        .stdout("alpha\n");
}

#[test]
fn tags_multiple_repeated_flags() {
    app()
        .args(["tags", "--tag", "alpha", "--tag", "beta", "--tag", "gamma"])
        .assert()
        .success()
        .stdout("alpha\nbeta\ngamma\n");
}

// ── version / help ───────────────────────────────────────────────────────────

#[test]
fn version_flag() {
    app()
        .args(["--version"])
        .assert()
        .success()
        .stdout(predicate::str::contains("0.1.0"));
}

#[test]
fn help_flag() {
    app()
        .args(["--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("testapp"))
        .stdout(predicate::str::contains("greet"))
        .stdout(predicate::str::contains("echo"))
        .stdout(predicate::str::contains("fail"));
}

#[test]
fn unknown_flag_exits_nonzero() {
    app()
        .args(["--no-such-flag"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown flag"));
}

#[test]
fn unknown_subcommand_exits_nonzero() {
    app()
        .args(["nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown command"));
}

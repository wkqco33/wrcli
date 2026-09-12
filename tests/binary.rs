//! Binary integration tests using assert_cmd.
//!
//! These tests spawn the `testapp` binary as a real process and verify
//! stdout, stderr, and exit codes — something library-direct tests cannot do.

use assert_cmd::Command;
use predicates::prelude::*;

fn app() -> Command {
    Command::cargo_bin("testapp").expect("testapp binary not found")
}

fn exitapp() -> Command {
    Command::cargo_bin("exitapp").expect("exitapp binary not found")
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

// ── hidden / deprecated / global flags ───────────────────────────────────────

#[test]
fn hidden_command_runs_but_is_not_listed() {
    app().args(["secret"]).assert().success().stdout("secret\n");
    app()
        .args(["--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("secret").not())
        .stdout(predicate::str::contains("--internal").not());
}

#[test]
fn subcommand_help_lists_global_flags() {
    app()
        .args(["greet", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Global Flags:"))
        .stdout(predicate::str::contains("--verbose"));
}

#[test]
fn deprecated_command_warns_on_stderr() {
    app()
        .args(["old"])
        .assert()
        .success()
        .stdout("old\n")
        .stderr(predicate::str::contains("deprecated"));
}

#[test]
fn usage_shows_args_hint() {
    app()
        .args(["greet", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("testapp greet <name> [flags]"));
}

#[test]
fn help_marks_deprecated_flag() {
    app()
        .args(["greet", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("(deprecated)"));
}

#[test]
fn deprecated_flag_warns_on_stderr() {
    app()
        .args(["greet", "--legacy", "Alice"])
        .assert()
        .success()
        .stdout("Hello, Alice!\n")
        .stderr(predicate::str::contains("deprecated"));
}

// ── execute_or_exit ──────────────────────────────────────────────────────────

#[test]
fn execute_or_exit_success() {
    exitapp().args(["ok"]).assert().success().stdout("ok\n");
}

#[test]
fn execute_or_exit_usage_error_exits_two() {
    exitapp()
        .args(["--bogus"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("Error:"));
}

#[test]
fn execute_or_exit_user_error_exits_one() {
    exitapp()
        .args(["fail"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("boom"));
}

// ── messaging stream conventions (clig.dev) ──────────────────────────────────

#[test]
fn messaging_streams_follow_clig_conventions() {
    app()
        .args(["messages"])
        .assert()
        .success()
        .stdout(predicate::str::contains("done"))
        .stdout(predicate::str::contains("careful").not())
        .stdout(predicate::str::contains("note").not())
        .stdout(predicate::str::contains("bad").not())
        .stderr(predicate::str::contains("careful"))
        .stderr(predicate::str::contains("note"))
        .stderr(predicate::str::contains("bad"));
}

// ── help subcommand / no-args / examples / support (clig.dev) ────────────────

#[test]
fn builtin_help_subcommand_shows_root_help() {
    app()
        .args(["help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Available Commands:"))
        .stdout(predicate::str::contains("greet"));
}

#[test]
fn builtin_help_subcommand_resolves_subcommand() {
    app()
        .args(["help", "greet"])
        .assert()
        .success()
        .stdout(predicate::str::contains("testapp greet <name> [flags]"));
}

#[test]
fn builtin_help_subcommand_unknown_target_fails() {
    app()
        .args(["help", "ghost"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown command"));
}

#[test]
fn no_args_parent_prints_help_and_succeeds() {
    app()
        .assert()
        .success()
        .stdout(predicate::str::contains("Available Commands:"));
}

#[test]
fn help_leads_with_examples_before_description() {
    let out = app().args(["help"]).output().expect("run testapp");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let ex = stdout.find("Examples:").expect("Examples section missing");
    let flags = stdout.find("Flags:").expect("Flags section missing");
    assert!(ex < flags, "examples must precede flags:\n{stdout}");
    assert!(
        stdout.contains("testapp greet Alice"),
        "example text missing:\n{stdout}"
    );
}

#[test]
fn help_shows_support_and_docs_links() {
    app()
        .args(["help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Support:"))
        .stdout(predicate::str::contains(
            "https://github.com/wkqco33/wrcli/issues",
        ))
        .stdout(predicate::str::contains("Documentation:"))
        .stdout(predicate::str::contains("https://docs.example.com/testapp"));
}

#[test]
fn subcommand_help_docs_url_substitutes_command_path() {
    app()
        .args(["greet", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("docs.example.com/testapp greet"));
}

// ── standard flags / color / plain (clig.dev) ────────────────────────────────

#[test]
fn force_color_env_enables_color_in_help() {
    app()
        .env_remove("NO_COLOR")
        .env("FORCE_COLOR", "1")
        .args(["--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\u{1b}["));
}

#[test]
fn no_color_flag_wins_over_force_color() {
    app()
        .env_remove("NO_COLOR")
        .env("FORCE_COLOR", "1")
        .args(["--no-color", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\u{1b}[").not());
}

#[test]
fn color_flag_never_disables_color() {
    app()
        .env_remove("NO_COLOR")
        .env("FORCE_COLOR", "1")
        .args(["--color", "never", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\u{1b}[").not());
}

#[test]
fn plain_flag_renders_tab_separated_records() {
    app()
        .args(["list", "--plain"])
        .assert()
        .success()
        .stdout("Name\tVersion\nwrcli\t0.3.0\nserde\t1.0\n");
}

#[test]
fn standard_flags_are_listed_in_help() {
    app()
        .args(["--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("-q, --quiet"))
        .stdout(predicate::str::contains("--no-input"))
        .stdout(predicate::str::contains("--plain"))
        .stdout(predicate::str::contains("--json"));
}

// ── sensitive flags (clig.dev: don't leak secrets) ───────────────────────────

#[test]
fn sensitive_flag_default_is_hidden_but_flag_is_listed() {
    app()
        .args(["--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--token"))
        .stdout(predicate::str::contains("super-secret").not());
}

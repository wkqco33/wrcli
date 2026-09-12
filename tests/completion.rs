//! Completion script generation integration tests.

use wrcli::{Command, Flag, FlagValue};

fn app() -> Command {
    Command::new("myapp")
        .flag(Flag::new("verbose", FlagValue::Bool(false), "verbose").short('v'))
        .subcommand(Command::new("serve").flag(Flag::new("port", FlagValue::Int(0), "port")))
        .subcommand(Command::new("config"))
}

#[test]
fn bash_completion_contains_subcommands() {
    let script = app().gen_completion("bash").unwrap();
    assert!(script.contains("myapp"));
    assert!(script.contains("serve"));
    assert!(script.contains("config"));
}

#[test]
fn bash_completion_contains_flags() {
    let script = app().gen_completion("bash").unwrap();
    assert!(script.contains("--verbose"));
    assert!(script.contains("--port"));
}

#[test]
fn zsh_completion_contains_subcommands() {
    let script = app().gen_completion("zsh").unwrap();
    assert!(script.contains("serve"));
    assert!(script.contains("config"));
}

#[test]
fn fish_completion_contains_subcommands() {
    let script = app().gen_completion("fish").unwrap();
    assert!(script.contains("serve"));
    assert!(script.contains("config"));
}

#[test]
fn unsupported_shell_returns_error() {
    let err = app().gen_completion("powershell").unwrap_err();
    assert!(matches!(
        err,
        wrcli::WrCliError::UnsupportedCompletionShell(_)
    ));
}

#[test]
fn bash_completion_ends_with_complete_builtin() {
    let script = app().gen_completion("bash").unwrap();
    assert!(
        script.trim_end().ends_with("complete -F _myapp myapp"),
        "malformed bash tail: {:?}",
        script.lines().last()
    );
}

#[test]
fn zsh_completion_has_single_compdef() {
    let script = app().gen_completion("zsh").unwrap();
    assert_eq!(script.matches("#compdef").count(), 1);
}

#[test]
fn zsh_completion_continues_argument_lines() {
    let script = app().gen_completion("zsh").unwrap();
    let specs: Vec<&str> = script
        .lines()
        .filter(|l| l.trim_start().starts_with('\''))
        .collect();
    assert!(!specs.is_empty());
    for line in &specs[..specs.len() - 1] {
        assert!(
            line.trim_end().ends_with('\\'),
            "missing continuation: {line:?}"
        );
    }
}

#[test]
fn fish_completion_includes_short_flag() {
    let script = app().gen_completion("fish").unwrap();
    assert!(script.contains("-s v"), "short flag missing: {script}");
}

#[test]
fn completion_excludes_hidden_entries() {
    let script = Command::new("myapp")
        .flag(Flag::new("secret", FlagValue::Bool(false), "secret").hidden())
        .subcommand(Command::new("hidden").hidden())
        .subcommand(Command::new("visible"))
        .gen_completion("bash")
        .unwrap();
    assert!(
        !script.contains("hidden"),
        "hidden command leaked: {script}"
    );
    assert!(!script.contains("--secret"), "hidden flag leaked: {script}");
    assert!(script.contains("visible"));
}

// ── 동적 completion API ──────────────────────────────────────────────────────

#[test]
fn complete_suggests_subcommands() {
    let cmd = Command::new("myapp")
        .subcommand(Command::new("serve"))
        .subcommand(Command::new("config"));
    let mut c = cmd.complete(&["".to_owned()]);
    c.sort();
    assert_eq!(c, vec!["config", "help", "serve"]);
}

#[test]
fn complete_filters_subcommands_by_prefix() {
    let cmd = Command::new("myapp")
        .subcommand(Command::new("serve"))
        .subcommand(Command::new("search"))
        .subcommand(Command::new("config"));
    let mut c = cmd.complete(&["se".to_owned()]);
    c.sort();
    assert_eq!(c, vec!["search", "serve"]);
}

#[test]
fn complete_suggests_flags_by_prefix() {
    let cmd = Command::new("myapp")
        .flag(Flag::new("verbose", FlagValue::Bool(false), "verbose"))
        .flag(Flag::new("version", FlagValue::Bool(false), "version"));
    assert_eq!(cmd.complete(&["--verb".to_owned()]), vec!["--verbose"]);
}

#[test]
fn complete_walks_into_subcommand() {
    let cmd = Command::new("myapp").subcommand(Command::new("serve").flag(Flag::new(
        "port",
        FlagValue::Int(0),
        "port",
    )));
    assert_eq!(
        cmd.complete(&["serve".to_owned(), "--po".to_owned()]),
        vec!["--port"]
    );
}

#[test]
fn complete_skips_flag_values_when_walking() {
    let cmd = Command::new("myapp")
        .flag(Flag::new("name", FlagValue::String(String::new()), "name"))
        .subcommand(Command::new("serve"));
    assert_eq!(
        cmd.complete(&["--name".to_owned(), "value".to_owned(), "se".to_owned()]),
        vec!["serve"]
    );
}

#[test]
fn complete_uses_arg_candidates() {
    let cmd = Command::new("myapp").subcommand(
        Command::new("run").arg_candidates(|_| vec!["alpha".to_owned(), "beta".to_owned()]),
    );
    assert_eq!(
        cmd.complete(&["run".to_owned(), "a".to_owned()]),
        vec!["alpha"]
    );
}

#[test]
fn complete_hides_hidden_entries() {
    let cmd = Command::new("myapp")
        .subcommand(Command::new("visible"))
        .subcommand(Command::new("secret").hidden())
        .flag(Flag::new("shown", FlagValue::Bool(false), "shown"))
        .flag(Flag::new("skipped", FlagValue::Bool(false), "skipped").hidden());
    let mut subs = cmd.complete(&["".to_owned()]);
    subs.sort();
    assert_eq!(subs, vec!["help", "visible"]);
    assert_eq!(cmd.complete(&["--s".to_owned()]), vec!["--shown"]);
}

#[test]
fn completion_request_intercepts_marker() {
    let cmd = Command::new("myapp").subcommand(Command::new("serve"));
    let out = cmd.completion_request(vec!["__complete".to_owned(), "".to_owned()]);
    assert!(out.is_some());
    assert!(out.unwrap().contains(&"serve".to_owned()));
    assert!(cmd.completion_request(vec!["serve".to_owned()]).is_none());
}

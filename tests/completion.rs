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

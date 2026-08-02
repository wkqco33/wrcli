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

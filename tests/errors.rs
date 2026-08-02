//! 에러 핸들링 및 help/version 스모크 테스트.

mod common;
use common::args;

#[cfg(any(feature = "toml-config", feature = "json-config"))]
use std::io::Write;
use wrcli::{Command, Config, Flag, FlagValue, WrCliError};

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
fn command_has_no_runner_returns_error() {
    let err = Command::new("app")
        .short("My app")
        .execute_with(args(""))
        .unwrap_err();
    assert!(matches!(err, WrCliError::CommandHasNoRunner(n) if n == "app"));
}

#[test]
fn subcommand_has_no_runner_returns_error() {
    let err = Command::new("app")
        .subcommand(Command::new("sub").short("a subcommand"))
        .execute_with(args("sub"))
        .unwrap_err();
    assert!(matches!(err, WrCliError::CommandHasNoRunner(n) if n == "sub"));
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

#[test]
fn help_flag_wins_over_unrecognized_leading_token() {
    // 회귀 테스트: 인식 불가 토큰이 --help보다 먼저 와도 help가 출력되어야 함
    // (dispatch가 첫 매치에서 조기 break하면 UnknownSubcommand 에러로 새 나감).
    Command::new("app")
        .subcommand(Command::new("sub").on_run(|_| {}))
        .execute_with(args("ghost --help"))
        .unwrap();
}

#[test]
fn error_display_unknown_flag() {
    let err = Command::new("app")
        .on_run(|_| {})
        .execute_with(args("--bogus"))
        .unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("unknown flag"));
    assert!(msg.contains("--bogus"));
    assert!(msg.contains("app"));
}

#[test]
fn error_display_unknown_subcommand() {
    let err = Command::new("app")
        .subcommand(Command::new("valid").on_run(|_| {}))
        .execute_with(args("ghost"))
        .unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("unknown command"));
    assert!(msg.contains("ghost"));
    assert!(msg.contains("app"));
}

#[test]
fn error_display_missing_required_flag() {
    let err = Command::new("app")
        .flag(Flag::new("name", FlagValue::String(String::new()), "name").required())
        .on_run(|_| {})
        .execute_with(args(""))
        .unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("required flag"));
    assert!(msg.contains("--name"));
}

#[test]
fn error_display_missing_flag_value() {
    let err = Command::new("app")
        .flag(Flag::new("name", FlagValue::String(String::new()), "name"))
        .on_run(|_| {})
        .execute_with(args("--name"))
        .unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("requires a value"));
    assert!(msg.contains("--name"));
}

#[test]
fn error_display_invalid_flag_value() {
    let err = Command::new("app")
        .flag(Flag::new("port", FlagValue::Int(0), "port"))
        .on_run(|_| {})
        .execute_with(args("--port not-a-number"))
        .unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("invalid value"));
    assert!(msg.contains("--port"));
    assert!(msg.contains("integer"));
}

#[test]
fn error_display_config_file_not_found() {
    let mut cfg = Config::new()
        .set_config_name("nonexistent")
        .set_config_type("toml")
        .add_config_path("/tmp");
    let err = cfg.read_in_config().unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("not found"));
    assert!(msg.contains("nonexistent.toml"));
}

#[test]
fn error_display_unsupported_config_format() {
    let err = WrCliError::UnsupportedConfigFormat("ini".to_owned());
    let msg = err.to_string();
    assert!(msg.contains("unsupported config format"));
    assert!(msg.contains("ini"));
}

#[test]
fn error_display_command_has_no_runner() {
    let err = WrCliError::CommandHasNoRunner("serve".to_owned());
    let msg = err.to_string();
    assert!(msg.contains("no run handler"));
    assert!(msg.contains("serve"));
}

#[test]
fn error_display_arg_validation_failed() {
    let err = WrCliError::ArgValidationFailed("requires at least 2 argument(s)".to_owned());
    let msg = err.to_string();
    assert!(msg.contains("requires at least 2 argument(s)"));
}

#[test]
fn error_display_io() {
    let err: WrCliError =
        std::io::Error::new(std::io::ErrorKind::NotFound, "file not found").into();
    let msg = err.to_string();
    assert!(msg.contains("io error"));
    assert!(msg.contains("file not found"));
}

#[cfg(feature = "toml-config")]
#[test]
fn config_parse_error_malformed_toml() {
    use common::tempdir;
    let dir = tempdir();
    let path = dir.path().join("bad.toml");
    let mut f = std::fs::File::create(&path).unwrap();
    writeln!(f, "key = = broken").unwrap();

    let mut cfg = Config::new()
        .set_config_name("bad")
        .set_config_type("toml")
        .add_config_path(dir.path().to_path_buf());
    let err = cfg.read_in_config().unwrap_err();
    assert!(matches!(err, WrCliError::ConfigParseError { .. }));
    let msg = err.to_string();
    assert!(msg.contains("failed to parse config"));
    assert!(msg.contains("bad.toml"));
}

#[cfg(feature = "json-config")]
#[test]
fn config_parse_error_malformed_json() {
    use common::tempdir;
    let dir = tempdir();
    let path = dir.path().join("bad.json");
    let mut f = std::fs::File::create(&path).unwrap();
    writeln!(f, "{{ broken json }}").unwrap();

    let mut cfg = Config::new()
        .set_config_name("bad")
        .set_config_type("json")
        .add_config_path(dir.path().to_path_buf());
    let err = cfg.read_in_config().unwrap_err();
    assert!(matches!(err, WrCliError::ConfigParseError { .. }));
    let msg = err.to_string();
    assert!(msg.contains("failed to parse config"));
    assert!(msg.contains("bad.json"));
}

//! Integration tests for wrcli.
//!
//! These tests exercise the full command dispatch, flag parsing, config, and
//! lifecycle hook chain end-to-end as a library user would.

use std::sync::{Arc, Mutex};
use wrcli::{Command, Config, Flag, FlagValue, WrCliError};
use wrcli::args::{arbitrary_args, exact_args, maximum_n_args, minimum_n_args, no_args};

fn args(s: &str) -> Vec<String> {
    if s.trim().is_empty() {
        vec![]
    } else {
        s.split_whitespace().map(str::to_owned).collect()
    }
}

// ── Flag parsing ──────────────────────────────────────────────────────────────

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
        .on_run(move |ctx| *out2.lock().unwrap() = ctx.flags.get_string("output").unwrap().to_owned())
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
        .flag(Flag::new("debug",   FlagValue::Bool(false), "d").short('d'))
        .flag(Flag::new("quiet",   FlagValue::Bool(false), "q").short('q'))
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
        .execute_with(args("--ratio 3.14"))
        .unwrap();
    assert!(((*out.lock().unwrap()) - 3.14).abs() < 1e-10);
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

// ── Positional args ───────────────────────────────────────────────────────────

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

// ── Arg validators ────────────────────────────────────────────────────────────

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

// ── Subcommand routing ────────────────────────────────────────────────────────

#[test]
fn subcommand_basic_dispatch() {
    let ran = Arc::new(Mutex::new(false));
    let ran2 = ran.clone();
    Command::new("app")
        .subcommand(
            Command::new("sub")
                .on_run(move |_| *ran2.lock().unwrap() = true),
        )
        .execute_with(args("sub"))
        .unwrap();
    assert!(*ran.lock().unwrap());
}

#[test]
fn subcommand_nested_two_levels() {
    let path = Arc::new(Mutex::new(vec![]));
    let path2 = path.clone();
    Command::new("app")
        .subcommand(
            Command::new("config")
                .subcommand(
                    Command::new("get")
                        .args(exact_args(1))
                        .on_run(move |ctx| *path2.lock().unwrap() = ctx.command_path.clone()),
                ),
        )
        .execute_with(args("config get mykey"))
        .unwrap();
    assert_eq!(*path.lock().unwrap(), vec!["app", "config", "get"]);
}

#[test]
fn subcommand_alias() {
    let ran = Arc::new(Mutex::new(false));
    let ran2 = ran.clone();
    Command::new("app")
        .subcommand(
            Command::new("serve")
                .alias("server")
                .alias("s")
                .on_run(move |_| *ran2.lock().unwrap() = true),
        )
        .execute_with(args("s"))
        .unwrap();
    assert!(*ran.lock().unwrap());
}

#[test]
fn subcommand_with_flags_before_name() {
    // Global flags before subcommand name: `app --verbose sub`
    let flag_val = Arc::new(Mutex::new(false));
    let flag2 = flag_val.clone();
    Command::new("app")
        .persistent_flag(Flag::new("verbose", FlagValue::Bool(false), "verbose").short('v'))
        .subcommand(
            Command::new("sub")
                .on_run(move |ctx| *flag2.lock().unwrap() = ctx.flags.get_bool("verbose").unwrap_or(false)),
        )
        .execute_with(args("--verbose sub"))
        .unwrap();
    // Note: --verbose is parsed by the leaf after persistent flag injection
    assert!(*flag_val.lock().unwrap());
}

#[test]
fn unknown_subcommand_returns_error() {
    let err = Command::new("app")
        .subcommand(Command::new("sub").on_run(|_| {}))
        .execute_with(args("ghost"))
        .unwrap_err();
    assert!(matches!(err, WrCliError::UnknownSubcommand { .. }));
}

// ── Persistent flags ──────────────────────────────────────────────────────────

#[test]
fn persistent_flag_visible_in_leaf() {
    let val = Arc::new(Mutex::new(false));
    let val2 = val.clone();
    Command::new("app")
        .persistent_flag(Flag::new("debug", FlagValue::Bool(false), "debug").short('d'))
        .subcommand(
            Command::new("sub")
                .subcommand(
                    Command::new("deep")
                        .on_run(move |ctx| *val2.lock().unwrap() = ctx.flags.get_bool("debug").unwrap_or(false)),
                ),
        )
        .execute_with(args("sub deep --debug"))
        .unwrap();
    assert!(*val.lock().unwrap());
}

#[test]
fn local_flag_not_visible_in_sibling_subcommand() {
    // Local flags should NOT leak to other commands.
    let err = Command::new("app")
        .subcommand(Command::new("a").flag(Flag::new("local", FlagValue::Bool(false), "x")).on_run(|_| {}))
        .subcommand(Command::new("b").on_run(|_| {}))
        .execute_with(args("b --local"))
        .unwrap_err();
    assert!(matches!(err, WrCliError::UnknownFlag { .. }));
}

// ── Lifecycle hooks ───────────────────────────────────────────────────────────

#[test]
fn lifecycle_hooks_full_order() {
    let log: Arc<Mutex<Vec<&'static str>>> = Arc::new(Mutex::new(vec![]));
    let (l1, l2, l3, l4, l5, l6_root, l6_sub) = (
        log.clone(), log.clone(), log.clone(),
        log.clone(), log.clone(), log.clone(), log.clone(),
    );
    Command::new("app")
        .on_persistent_pre_run(move |_| l1.lock().unwrap().push("root:persistent_pre"))
        .on_persistent_post_run(move |_| l6_root.lock().unwrap().push("root:persistent_post"))
        .subcommand(
            Command::new("sub")
                .on_persistent_pre_run(move |_| l2.lock().unwrap().push("sub:persistent_pre"))
                .on_pre_run(move |_| l3.lock().unwrap().push("sub:pre"))
                .on_run(move |_| l4.lock().unwrap().push("sub:run"))
                .on_post_run(move |_| l5.lock().unwrap().push("sub:post"))
                .on_persistent_post_run(move |_| l6_sub.lock().unwrap().push("sub:persistent_post")),
        )
        .execute_with(args("sub"))
        .unwrap();

    assert_eq!(
        *log.lock().unwrap(),
        vec![
            "root:persistent_pre",
            "sub:persistent_pre",
            "sub:pre",
            "sub:run",
            "sub:post",
            "sub:persistent_post",
            "root:persistent_post",
        ]
    );
}

#[test]
fn run_e_error_aborts_post_hooks() {
    let post_called = Arc::new(Mutex::new(false));
    let post2 = post_called.clone();
    let err = Command::new("app")
        .on_run_e(|_| Err(WrCliError::ArgValidationFailed("fail".to_owned())))
        .on_post_run(move |_| *post2.lock().unwrap() = true)
        .execute_with(args(""))
        .unwrap_err();
    assert!(matches!(err, WrCliError::ArgValidationFailed(_)));
    assert!(!*post_called.lock().unwrap());
}

// ── Config / Viper ────────────────────────────────────────────────────────────

#[test]
fn config_default_value() {
    let val = Arc::new(Mutex::new(0i64));
    let val2 = val.clone();
    Command::new("app")
        .with_config(Config::new().set_default("timeout", 30i64))
        .on_run(move |ctx| *val2.lock().unwrap() = ctx.config.get_int("timeout").unwrap())
        .execute_with(args(""))
        .unwrap();
    assert_eq!(*val.lock().unwrap(), 30);
}

#[test]
fn config_env_var_override() {
    // SAFETY: single-threaded test, no concurrent env access
    unsafe { std::env::set_var("WRCLI_TEST_PORT", "9999"); }
    let val = Arc::new(Mutex::new(0i64));
    let val2 = val.clone();
    Command::new("app")
        .with_config(
            Config::new()
                .set_default("port", 8080i64)
                .automatic_env()
                .set_env_prefix("WRCLI_TEST"),
        )
        .on_run(move |ctx| *val2.lock().unwrap() = ctx.config.get_int("port").unwrap())
        .execute_with(args(""))
        .unwrap();
    unsafe { std::env::remove_var("WRCLI_TEST_PORT"); }
    assert_eq!(*val.lock().unwrap(), 9999);
}

#[test]
fn config_explicit_env_binding() {
    // SAFETY: single-threaded test, no concurrent env access
    unsafe { std::env::set_var("MY_CUSTOM_VAR", "hello"); }
    let val = Arc::new(Mutex::new(String::new()));
    let val2 = val.clone();
    Command::new("app")
        .with_config(Config::new().bind_env("greeting", "MY_CUSTOM_VAR"))
        .on_run(move |ctx| *val2.lock().unwrap() = ctx.config.get_string("greeting").unwrap())
        .execute_with(args(""))
        .unwrap();
    unsafe { std::env::remove_var("MY_CUSTOM_VAR"); }
    assert_eq!(*val.lock().unwrap(), "hello");
}

#[test]
fn config_toml_file() {
    use std::io::Write;
    let dir = tempdir();
    let file_path = dir.path().join("myapp.toml");
    let mut f = std::fs::File::create(&file_path).unwrap();
    writeln!(f, "[server]\nport = 7777\nhost = \"0.0.0.0\"").unwrap();

    let val = Arc::new(Mutex::new(0i64));
    let val2 = val.clone();
    let mut cfg = Config::new()
        .set_config_name("myapp")
        .set_config_type("toml")
        .add_config_path(dir.path().to_path_buf());
    cfg.read_in_config().unwrap();

    Command::new("app")
        .with_config(cfg)
        .on_run(move |ctx| *val2.lock().unwrap() = ctx.config.get_int("server.port").unwrap())
        .execute_with(args(""))
        .unwrap();
    assert_eq!(*val.lock().unwrap(), 7777);
}

#[test]
fn config_json_file() {
    use std::io::Write;
    let dir = tempdir();
    let file_path = dir.path().join("app.json");
    let mut f = std::fs::File::create(&file_path).unwrap();
    writeln!(f, r#"{{"database": {{"url": "postgres://localhost/db"}}}}"#).unwrap();

    let val = Arc::new(Mutex::new(String::new()));
    let val2 = val.clone();
    let mut cfg = Config::new()
        .set_config_name("app")
        .set_config_type("json")
        .add_config_path(dir.path().to_path_buf());
    cfg.read_in_config().unwrap();

    Command::new("app")
        .with_config(cfg)
        .on_run(move |ctx| *val2.lock().unwrap() = ctx.config.get_string("database.url").unwrap())
        .execute_with(args(""))
        .unwrap();
    assert_eq!(*val.lock().unwrap(), "postgres://localhost/db");
}

#[test]
fn config_missing_file_is_error() {
    let mut cfg = Config::new()
        .set_config_name("nonexistent")
        .set_config_type("toml")
        .add_config_path("/tmp");
    let err = cfg.read_in_config().unwrap_err();
    assert!(matches!(err, wrcli::WrCliError::ConfigFileNotFound { .. }));
}

#[test]
fn config_ctx_get_fallthrough_flags_then_config() {
    // ctx.get_int("port") should prefer flag over config default
    let val = Arc::new(Mutex::new(0i64));
    let val2 = val.clone();
    Command::new("app")
        .flag(Flag::new("port", FlagValue::Int(0), "port"))
        .with_config(Config::new().set_default("port", 8080i64))
        .on_run(move |ctx| {
            // flag is set to 3000, should override config default 8080
            *val2.lock().unwrap() = ctx.get_int("port").unwrap();
        })
        .execute_with(args("--port 3000"))
        .unwrap();
    assert_eq!(*val.lock().unwrap(), 3000);
}

// ── Error handling ────────────────────────────────────────────────────────────

#[test]
fn user_error_from_run_e() {
    use std::fmt;
    #[derive(Debug)]
    struct MyErr;
    impl fmt::Display for MyErr { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "my error") } }
    impl std::error::Error for MyErr {}

    let err = Command::new("app")
        .on_run_e(|_| Err(WrCliError::user(MyErr)))
        .execute_with(args(""))
        .unwrap_err();
    assert!(err.to_string().contains("my error"));
}

#[test]
fn from_io_error() {
    let io_err: WrCliError = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing").into();
    assert!(matches!(io_err, WrCliError::Io(_)));
}

// ── Help / Version (smoke tests — just verify no panic) ───────────────────────

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

// ── Flag→Config automatic binding ────────────────────────────────────────────

#[test]
fn flag_value_visible_via_config_get() {
    // After `--port 3000`, ctx.config.get_int("port") should also return 3000
    // (not the default), because the flag is injected into config's layer 4.
    let (flag_val, config_val) = (
        Arc::new(Mutex::new(0i64)),
        Arc::new(Mutex::new(0i64)),
    );
    let (fv2, cv2) = (flag_val.clone(), config_val.clone());
    Command::new("app")
        .flag(Flag::new("port", FlagValue::Int(0), "port"))
        .with_config(Config::new().set_default("port", 8080i64))
        .on_run(move |ctx| {
            *fv2.lock().unwrap() = ctx.flags.get_int("port").unwrap();
            *cv2.lock().unwrap() = ctx.config.get_int("port").unwrap();
        })
        .execute_with(args("--port 3000"))
        .unwrap();
    assert_eq!(*flag_val.lock().unwrap(), 3000);
    assert_eq!(*config_val.lock().unwrap(), 3000, "flag should shadow config default");
}

#[test]
fn flag_default_does_not_shadow_config_file() {
    // When a flag is NOT set by the user, its default must NOT override a
    // config-file value.  Only user-provided flags override config.
    use std::io::Write;
    let dir = tempdir();
    let mut f = std::fs::File::create(dir.path().join("app.toml")).unwrap();
    writeln!(f, "port = 9090").unwrap();

    let config_val = Arc::new(Mutex::new(0i64));
    let cv2 = config_val.clone();
    let mut cfg = Config::new()
        .set_config_name("app")
        .set_config_type("toml")
        .add_config_path(dir.path().to_path_buf());
    cfg.read_in_config().unwrap();

    Command::new("app")
        // Flag has default 8080 but user does NOT pass it
        .flag(Flag::new("port", FlagValue::Int(8080), "port"))
        .with_config(cfg)
        .on_run(move |ctx| {
            // config file says 9090, flag default is 8080 (not set by user)
            // config.get_int should return the file value 9090
            *cv2.lock().unwrap() = ctx.config.get_int("port").unwrap();
        })
        .execute_with(args(""))   // <-- no --port flag
        .unwrap();
    assert_eq!(*config_val.lock().unwrap(), 9090, "flag default must not override config file");
}

#[test]
fn flag_overrides_config_file_value() {
    // When the user DOES pass --port, it should shadow even the config-file value.
    use std::io::Write;
    let dir = tempdir();
    let mut f = std::fs::File::create(dir.path().join("app.toml")).unwrap();
    writeln!(f, "port = 9090").unwrap();

    let config_val = Arc::new(Mutex::new(0i64));
    let cv2 = config_val.clone();
    let mut cfg = Config::new()
        .set_config_name("app")
        .set_config_type("toml")
        .add_config_path(dir.path().to_path_buf());
    cfg.read_in_config().unwrap();

    Command::new("app")
        .flag(Flag::new("port", FlagValue::Int(8080), "port"))
        .with_config(cfg)
        .on_run(move |ctx| {
            *cv2.lock().unwrap() = ctx.config.get_int("port").unwrap();
        })
        .execute_with(args("--port 5000"))
        .unwrap();
    assert_eq!(*config_val.lock().unwrap(), 5000, "explicit flag must override config file");
}

#[test]
fn persistent_flag_bound_into_config() {
    // Persistent flags set by the user should also be visible via config.
    let config_val = Arc::new(Mutex::new(false));
    let cv2 = config_val.clone();
    Command::new("app")
        .persistent_flag(Flag::new("verbose", FlagValue::Bool(false), "verbose").short('v'))
        .subcommand(
            Command::new("sub")
                .on_run(move |ctx| {
                    *cv2.lock().unwrap() = ctx.config.get_bool("verbose").unwrap_or(false);
                }),
        )
        .execute_with(args("sub --verbose"))
        .unwrap();
    assert!(*config_val.lock().unwrap(), "persistent flag should appear in config");
}

// ── YAML config ───────────────────────────────────────────────────────────────

#[cfg(feature = "yaml-config")]
#[test]
fn config_yaml_file() {
    use std::io::Write;
    let dir = tempdir();
    let mut f = std::fs::File::create(dir.path().join("app.yaml")).unwrap();
    writeln!(f, "server:\n  host: example.com\n  port: 4321").unwrap();

    let val = Arc::new(Mutex::new(String::new()));
    let val2 = val.clone();
    let mut cfg = Config::new()
        .set_config_name("app")
        .set_config_type("yaml")
        .add_config_path(dir.path().to_path_buf());
    cfg.read_in_config().unwrap();

    Command::new("app")
        .with_config(cfg)
        .on_run(move |ctx| {
            let host = ctx.config.get_string("server.host").unwrap();
            let port = ctx.config.get_int("server.port").unwrap();
            *val2.lock().unwrap() = format!("{}:{}", host, port);
        })
        .execute_with(args(""))
        .unwrap();
    assert_eq!(*val.lock().unwrap(), "example.com:4321");
}

// ── Feature: unsupported format error ────────────────────────────────────────

#[test]
fn config_unsupported_format_error() {
    let dir = tempdir();
    std::fs::File::create(dir.path().join("app.ini")).unwrap();
    let mut cfg = Config::new()
        .set_config_name("app")
        .set_config_type("ini")
        .add_config_path(dir.path().to_path_buf());
    // File exists but format is unsupported → UnsupportedConfigFormat
    let err = cfg.read_in_config().unwrap_err();
    assert!(matches!(err, wrcli::WrCliError::UnsupportedConfigFormat(_)));
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Minimal temp-directory helper (avoids the `tempfile` crate dependency).
struct TempDir(std::path::PathBuf);

impl TempDir {
    fn path(&self) -> &std::path::Path { &self.0 }
}

impl Drop for TempDir {
    fn drop(&mut self) { let _ = std::fs::remove_dir_all(&self.0); }
}

fn tempdir() -> TempDir {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().subsec_nanos();
    let path = std::env::temp_dir().join(format!("wrcli_test_{}", ts));
    std::fs::create_dir_all(&path).unwrap();
    TempDir(path)
}

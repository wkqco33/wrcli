//! Config 설정 시스템 및 Flag↔Config 바인딩 통합 테스트.

mod common;
use common::{EnvGuard, args, tempdir};

use std::sync::{Arc, Mutex};
use wrcli::{Command, Config, Flag, FlagValue};

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
    let _g = EnvGuard::set("WRCLI_TEST_PORT", "9999");
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
    assert_eq!(*val.lock().unwrap(), 9999);
}

#[test]
fn raw_get_includes_env_layer() {
    let _g = EnvGuard::set("WRCLI_TEST_RAW_GET", "42");
    let val = Arc::new(Mutex::new(String::new()));
    let val2 = val.clone();
    Command::new("app")
        .with_config(
            Config::new()
                .set_default("raw_get", 0i64)
                .automatic_env()
                .set_env_prefix("WRCLI_TEST"),
        )
        .on_run(move |ctx| {
            *val2.lock().unwrap() = ctx
                .config
                .get("raw_get")
                .and_then(|v| v.as_str().map(str::to_owned))
                .unwrap_or_default();
        })
        .execute_with(args(""))
        .unwrap();
    assert_eq!(*val.lock().unwrap(), "42");
}

#[test]
fn config_explicit_env_binding() {
    let _g = EnvGuard::set("MY_CUSTOM_VAR", "hello");
    let val = Arc::new(Mutex::new(String::new()));
    let val2 = val.clone();
    Command::new("app")
        .with_config(Config::new().bind_env("greeting", "MY_CUSTOM_VAR"))
        .on_run(move |ctx| *val2.lock().unwrap() = ctx.config.get_string("greeting").unwrap())
        .execute_with(args(""))
        .unwrap();
    assert_eq!(*val.lock().unwrap(), "hello");
}

#[cfg(feature = "toml-config")]
#[test]
fn config_toml_file() {
    use std::io::Write;
    let dir = tempdir();
    let mut f = std::fs::File::create(dir.path().join("myapp.toml")).unwrap();
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

#[cfg(feature = "json-config")]
#[test]
fn config_json_file() {
    use std::io::Write;
    let dir = tempdir();
    let mut f = std::fs::File::create(dir.path().join("app.json")).unwrap();
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
fn config_unsupported_format_error() {
    let dir = tempdir();
    std::fs::File::create(dir.path().join("app.ini")).unwrap();
    let mut cfg = Config::new()
        .set_config_name("app")
        .set_config_type("ini")
        .add_config_path(dir.path().to_path_buf());
    let err = cfg.read_in_config().unwrap_err();
    assert!(matches!(err, wrcli::WrCliError::UnsupportedConfigFormat(_)));
}

#[test]
fn config_ctx_get_fallthrough_flags_then_config() {
    let val = Arc::new(Mutex::new(0i64));
    let val2 = val.clone();
    Command::new("app")
        .flag(Flag::new("port", FlagValue::Int(0), "port"))
        .with_config(Config::new().set_default("port", 8080i64))
        .on_run(move |ctx| {
            *val2.lock().unwrap() = ctx.get_int("port").unwrap();
        })
        .execute_with(args("--port 3000"))
        .unwrap();
    assert_eq!(*val.lock().unwrap(), 3000);
}

#[test]
fn flag_value_visible_via_config_get() {
    let (flag_val, config_val) = (Arc::new(Mutex::new(0i64)), Arc::new(Mutex::new(0i64)));
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
    assert_eq!(
        *config_val.lock().unwrap(),
        3000,
        "flag should shadow config default"
    );
}

#[cfg(feature = "toml-config")]
#[test]
fn flag_default_does_not_shadow_config_file() {
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
        .execute_with(args(""))
        .unwrap();
    assert_eq!(
        *config_val.lock().unwrap(),
        9090,
        "flag default must not override config file"
    );
}

#[cfg(feature = "toml-config")]
#[test]
fn flag_overrides_config_file_value() {
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
    assert_eq!(
        *config_val.lock().unwrap(),
        5000,
        "explicit flag must override config file"
    );
}

#[test]
fn persistent_flag_bound_into_config() {
    let config_val = Arc::new(Mutex::new(false));
    let cv2 = config_val.clone();
    Command::new("app")
        .persistent_flag(Flag::new("verbose", FlagValue::Bool(false), "verbose").short('v'))
        .subcommand(Command::new("sub").on_run(move |ctx| {
            *cv2.lock().unwrap() = ctx.config.get_bool("verbose").unwrap_or(false);
        }))
        .execute_with(args("sub --verbose"))
        .unwrap();
    assert!(
        *config_val.lock().unwrap(),
        "persistent flag should appear in config"
    );
}

#[cfg(feature = "toml-config")]
#[test]
fn config_set_config_file_explicit_path() {
    use std::io::Write;
    let dir = tempdir();
    let mut f = std::fs::File::create(dir.path().join("custom.toml")).unwrap();
    writeln!(f, "port = 1234").unwrap();

    let val = Arc::new(Mutex::new(0i64));
    let val2 = val.clone();
    let mut cfg = Config::new().set_config_file(dir.path().join("custom.toml"));
    cfg.read_in_config().unwrap();

    Command::new("app")
        .with_config(cfg)
        .on_run(move |ctx| *val2.lock().unwrap() = ctx.config.get_int("port").unwrap())
        .execute_with(args(""))
        .unwrap();
    assert_eq!(*val.lock().unwrap(), 1234);
}

#[cfg(feature = "toml-config")]
#[test]
fn config_auto_detects_format_when_type_unset() {
    use std::io::Write;
    let dir = tempdir();
    let mut f = std::fs::File::create(dir.path().join("app.toml")).unwrap();
    writeln!(f, "timeout = 42").unwrap();

    let val = Arc::new(Mutex::new(0i64));
    let val2 = val.clone();
    let mut cfg = Config::new()
        .set_config_name("app")
        .add_config_path(dir.path().to_path_buf());
    cfg.read_in_config().unwrap();

    Command::new("app")
        .with_config(cfg)
        .on_run(move |ctx| *val2.lock().unwrap() = ctx.config.get_int("timeout").unwrap())
        .execute_with(args(""))
        .unwrap();
    assert_eq!(*val.lock().unwrap(), 42);
}

#[cfg(feature = "json-config")]
#[test]
fn config_auto_discovery_default_paths() {
    use std::io::Write;
    let home = tempdir();
    let cfg_dir = home.path().join(".config").join("myapp");
    std::fs::create_dir_all(&cfg_dir).unwrap();
    let mut f = std::fs::File::create(cfg_dir.join("myapp.json")).unwrap();
    writeln!(f, r#"{{"host": "auto.example.com"}}"#).unwrap();

    let _g = EnvGuard::set("HOME", home.path().to_str().unwrap());

    let val = Arc::new(Mutex::new(String::new()));
    let val2 = val.clone();
    let mut cfg = Config::new().set_config_name("myapp").automatic_env();
    cfg.read_in_config().unwrap();

    Command::new("app")
        .with_config(cfg)
        .on_run(move |ctx| *val2.lock().unwrap() = ctx.config.get_string("host").unwrap())
        .execute_with(args(""))
        .unwrap();
    assert_eq!(*val.lock().unwrap(), "auto.example.com");
}

#[test]
fn config_seeds_flag_default_when_flag_unset() {
    let val = Arc::new(Mutex::new(0i64));
    let val2 = val.clone();
    Command::new("app")
        .flag(Flag::new("port", FlagValue::Int(0), "port"))
        .with_config(Config::new().set_default("port", 8080i64))
        .on_run(move |ctx| *val2.lock().unwrap() = ctx.flags.get_int("port").unwrap())
        .execute_with(args(""))
        .unwrap();
    assert_eq!(
        *val.lock().unwrap(),
        8080,
        "flag value should fall back to config when not explicitly set"
    );
}

#[test]
fn config_does_not_seed_flag_when_explicitly_set() {
    let val = Arc::new(Mutex::new(0i64));
    let val2 = val.clone();
    Command::new("app")
        .flag(Flag::new("port", FlagValue::Int(0), "port"))
        .with_config(Config::new().set_default("port", 8080i64))
        .on_run(move |ctx| *val2.lock().unwrap() = ctx.flags.get_int("port").unwrap())
        .execute_with(args("--port 3000"))
        .unwrap();
    assert_eq!(*val.lock().unwrap(), 3000);
}

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

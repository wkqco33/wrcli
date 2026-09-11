//! Config 설정 시스템 및 Flag↔Config 바인딩 통합 테스트.

mod common;
use common::{EnvGuard, args, tempdir};

use std::sync::{Arc, Mutex};
use std::time::{Duration, UNIX_EPOCH};
use wrcli::config::SettingsEntry;
use wrcli::{Command, Config, ConfigValue, Flag, FlagValue};

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

    // XDG_CONFIG_HOME까지 임시 경로로 고정해야 외부 CI 환경(기본 XDG 설정)과
    // 무관하게 `~/.config/<name>` 탐지를 결정적으로 검증할 수 있다.
    let _g = EnvGuard::set_many(&[
        ("HOME", home.path().to_str().unwrap()),
        (
            "XDG_CONFIG_HOME",
            home.path().join(".config").to_str().unwrap(),
        ),
    ]);

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

// ── Phase 1: set / is_set / alias / delimiter / env 옵션 ─────────────────────

#[test]
fn set_overrides_default_and_env() {
    let _g = EnvGuard::set("WRCLI_SET_PORT", "1111");
    let cfg = Config::new()
        .set_default("port", 1i64)
        .automatic_env()
        .set_env_prefix("WRCLI_SET")
        .set("port", 9999i64);
    assert_eq!(cfg.get_int("port"), Some(9999));
}

#[test]
fn set_overrides_explicit_cli_flag() {
    let val = Arc::new(Mutex::new(0i64));
    let val2 = val.clone();
    Command::new("app")
        .flag(Flag::new("port", FlagValue::Int(0), "port"))
        .with_config(Config::new().set("port", 9999i64))
        .on_run(move |ctx| *val2.lock().unwrap() = ctx.config.get_int("port").unwrap_or(0))
        .execute_with(args("--port 1234"))
        .unwrap();
    assert_eq!(*val.lock().unwrap(), 9999);
}

#[test]
fn is_set_reflects_all_layers() {
    let _g = EnvGuard::set("WRCLI_ISSET_FLAG", "x");
    let cfg = Config::new()
        .set_default("a", 1i64)
        .set("b", 2i64)
        .automatic_env()
        .set_env_prefix("WRCLI_ISSET");
    assert!(cfg.is_set("a"), "default layer");
    assert!(cfg.is_set("b"), "explicit layer");
    assert!(cfg.is_set("flag"), "env layer");
    assert!(!cfg.is_set("missing"));
}

#[test]
fn register_alias_resolves_to_canonical_key() {
    let cfg = Config::new()
        .set_default("server.port", 8080i64)
        .register_alias("port", "server.port");
    assert_eq!(cfg.get_int("port"), Some(8080));
}

#[test]
fn register_alias_before_default_stores_canonical_key() {
    let cfg = Config::new()
        .register_alias("port", "server.port")
        .set_default("port", 8080i64);
    assert_eq!(cfg.get_int("server.port"), Some(8080));
}

#[test]
fn register_alias_chain_resolves() {
    let cfg = Config::new()
        .set_default("server.port", 8080i64)
        .register_alias("p", "port")
        .register_alias("port", "server.port");
    assert_eq!(cfg.get_int("p"), Some(8080));
}

#[test]
fn custom_key_delimiter_accesses_nested_keys() {
    let cfg = Config::new()
        .set_key_delimiter('/')
        .set_default("server.port", 8080i64);
    assert_eq!(cfg.get_int("server/port"), Some(8080));
}

#[test]
fn custom_env_key_replacer_maps_separator() {
    let _g = EnvGuard::set("WRCLI_REPL_DB__PORT", "1234");
    let cfg = Config::new()
        .automatic_env()
        .set_env_prefix("WRCLI_REPL")
        .set_env_key_replacer(&[(".", "__")]);
    assert_eq!(cfg.get_int("db.port"), Some(1234));
}

#[test]
fn allow_empty_env_default_uses_empty_value() {
    let _g = EnvGuard::set("WRCLI_EMPTY_VAL", "");
    let cfg = Config::new()
        .set_default("val", "fallback")
        .automatic_env()
        .set_env_prefix("WRCLI_EMPTY");
    assert_eq!(cfg.get_string("val"), Some(String::new()));
}

#[test]
fn allow_empty_env_false_falls_back_to_default() {
    let _g = EnvGuard::set("WRCLI_EMPTY_VAL", "");
    let cfg = Config::new()
        .set_default("val", "fallback")
        .automatic_env()
        .set_env_prefix("WRCLI_EMPTY")
        .allow_empty_env(false);
    assert_eq!(cfg.get_string("val"), Some("fallback".to_owned()));
}

// ── Phase 1: 추가 getter ─────────────────────────────────────────────────────

#[test]
fn get_int64_and_uint() {
    let cfg = Config::new()
        .set_default("a", 42i64)
        .set_default("neg", -1i64);
    assert_eq!(cfg.get_int64("a"), Some(42));
    assert_eq!(cfg.get_uint("a"), Some(42));
    assert_eq!(cfg.get_uint("neg"), None);
}

#[test]
fn get_string_slice_matches_string_vec() {
    let cfg = Config::new().set_default("tags", vec!["a", "b"]);
    assert_eq!(
        cfg.get_string_slice("tags"),
        Some(vec!["a".to_owned(), "b".to_owned()])
    );
}

#[test]
fn get_duration_parses_go_style_strings() {
    let cfg = Config::new()
        .set_default("timeout", "1h30m")
        .set_default("poll", "250ms")
        .set_default("secs", "30s");
    assert_eq!(cfg.get_duration("timeout"), Some(Duration::from_secs(5400)));
    assert_eq!(cfg.get_duration("poll"), Some(Duration::from_millis(250)));
    assert_eq!(cfg.get_duration("secs"), Some(Duration::from_secs(30)));
}

#[test]
fn get_duration_numeric_is_seconds() {
    let cfg = Config::new().set_default("t", 5i64);
    assert_eq!(cfg.get_duration("t"), Some(Duration::from_secs(5)));
}

#[test]
fn get_duration_invalid_returns_none() {
    let cfg = Config::new()
        .set_default("bare", "30")
        .set_default("junk", "not-a-duration");
    assert_eq!(cfg.get_duration("bare"), None);
    assert_eq!(cfg.get_duration("junk"), None);
}

#[test]
fn get_size_in_bytes_parses_units() {
    let cfg = Config::new()
        .set_default("a", "512")
        .set_default("b", "1KB")
        .set_default("c", "1.5MB")
        .set_default("d", "2GiB")
        .set_default("e", 4096i64);
    assert_eq!(cfg.get_size_in_bytes("a"), Some(512));
    assert_eq!(cfg.get_size_in_bytes("b"), Some(1024));
    assert_eq!(cfg.get_size_in_bytes("c"), Some(1_572_864));
    assert_eq!(cfg.get_size_in_bytes("d"), Some(2 * 1024 * 1024 * 1024));
    assert_eq!(cfg.get_size_in_bytes("e"), Some(4096));
}

#[test]
fn get_time_parses_unix_and_rfc3339() {
    let cfg = Config::new()
        .set_default("unix", 1_700_000_000i64)
        .set_default("rfc", "2023-11-14T22:13:20Z");
    let expected = UNIX_EPOCH + Duration::from_secs(1_700_000_000);
    assert_eq!(cfg.get_time("unix"), Some(expected));
    assert_eq!(cfg.get_time("rfc"), Some(expected));
}

#[test]
fn get_time_parses_offset_and_fraction() {
    let cfg = Config::new().set_default("t", "2023-11-15T07:13:20.500+09:00");
    let expected = UNIX_EPOCH + Duration::from_secs(1_700_000_000) + Duration::from_millis(500);
    assert_eq!(cfg.get_time("t"), Some(expected));
}

// ── Phase 2: all_keys / all_settings / map getter / sub / merge ──────────────

#[test]
fn all_keys_lists_sorted_union_of_layers() {
    let cfg = Config::new()
        .set_default("b", 1i64)
        .set("a", 2i64)
        .bind_env("token", "WRCLI_ALLKEYS_TOKEN");
    assert_eq!(
        cfg.all_keys(),
        vec!["a".to_owned(), "b".to_owned(), "token".to_owned()]
    );
}

#[test]
fn all_settings_reconstructs_nested_tree() {
    let cfg = Config::new()
        .set_default("server.host", "127.0.0.1")
        .set_default("server.port", 8080i64);
    let settings = cfg.all_settings();
    match settings.get("server") {
        Some(SettingsEntry::Map(inner)) => {
            assert_eq!(
                inner.get("host"),
                Some(&SettingsEntry::Value(ConfigValue::String(
                    "127.0.0.1".to_owned()
                )))
            );
            assert_eq!(
                inner.get("port"),
                Some(&SettingsEntry::Value(ConfigValue::Int(8080)))
            );
        }
        other => panic!("expected nested map, got {other:?}"),
    }
}

#[test]
fn get_string_map_string_returns_children() {
    let cfg = Config::new()
        .set_default("server.host", "127.0.0.1")
        .set_default("server.port", 8080i64);
    let map = cfg.get_string_map_string("server").unwrap();
    assert_eq!(map.get("host").map(String::as_str), Some("127.0.0.1"));
    assert_eq!(map.get("port").map(String::as_str), Some("8080"));
    assert!(cfg.get_string_map_string("missing").is_none());
}

#[test]
fn get_string_map_string_slice_returns_arrays() {
    let cfg = Config::new().set_default("allowed.ips", vec!["a", "b"]);
    let map = cfg.get_string_map_string_slice("allowed").unwrap();
    assert_eq!(map.get("ips"), Some(&vec!["a".to_owned(), "b".to_owned()]));
}

#[test]
fn sub_scopes_keys_to_prefix() {
    let cfg = Config::new()
        .set_default("server.host", "127.0.0.1")
        .set_default("server.port", 8080i64)
        .set_default("other", 1i64);
    let sub = cfg.sub("server");
    assert_eq!(sub.get_string("host"), Some("127.0.0.1".to_owned()));
    assert_eq!(sub.get_int("port"), Some(8080));
    assert_eq!(sub.get_int("other"), None);
}

#[cfg(feature = "toml-config")]
#[test]
fn read_config_from_reader_uses_config_type() {
    let mut cfg = Config::new().set_config_type("toml");
    cfg.read_config("[server]\nport = 9000\n".as_bytes())
        .unwrap();
    assert_eq!(cfg.get_int("server.port"), Some(9000));
}

#[cfg(feature = "toml-config")]
#[test]
fn merge_in_config_keeps_existing_values() {
    use std::io::Write;
    let dir = tempdir();
    let path = dir.path().join("extra.toml");
    let mut f = std::fs::File::create(&path).unwrap();
    writeln!(f, "port = 9999\nhost = \"merged\"").unwrap();
    drop(f);

    let mut cfg = Config::new().set_config_type("toml");
    cfg.read_config("port = 1111\n".as_bytes()).unwrap();
    cfg.merge_in_config(&path).unwrap();
    assert_eq!(cfg.get_int("port"), Some(1111));
    assert_eq!(cfg.get_string("host"), Some("merged".to_owned()));
}

#[test]
fn merge_config_map_adds_absent_keys() {
    let mut cfg = Config::new().set_default("a", 1i64);
    cfg.merge_config_map([("b".to_owned(), ConfigValue::Int(2))]);
    assert_eq!(cfg.get_int("b"), Some(2));
    assert_eq!(cfg.get_int("a"), Some(1));
}

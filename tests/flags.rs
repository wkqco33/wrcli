//! 플래그 파싱 통합 테스트.

mod common;
use common::args;

use std::sync::{Arc, Mutex};
use wrcli::{Command, Flag, FlagValue, WrCliError};

#[test]
#[should_panic(expected = "already registered")]
fn duplicate_short_flag_panics() {
    Command::new("app")
        .flag(Flag::new("verbose", FlagValue::Bool(false), "verbose").short('v'))
        .flag(Flag::new("version", FlagValue::Bool(false), "version").short('v'));
}

#[test]
#[should_panic(expected = "already registered")]
fn duplicate_flag_name_panics() {
    Command::new("app")
        .flag(Flag::new("name", FlagValue::String(String::new()), "name"))
        .flag(Flag::new("name", FlagValue::Int(0), "name again"));
}

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
        .flag(Flag::new(
            "output",
            FlagValue::String(String::new()),
            "output",
        ))
        .on_run(move |ctx| {
            *out2.lock().unwrap() = ctx.flags.get_string("output").unwrap().to_owned()
        })
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
        .flag(Flag::new("debug", FlagValue::Bool(false), "d").short('d'))
        .flag(Flag::new("quiet", FlagValue::Bool(false), "q").short('q'))
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
        .execute_with(args("--ratio 2.5"))
        .unwrap();
    assert!(((*out.lock().unwrap()) - 2.5).abs() < 1e-10);
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
fn flag_int_vec() {
    let out: Arc<Mutex<Vec<i64>>> = Arc::new(Mutex::new(vec![]));
    let out2 = out.clone();
    Command::new("app")
        .flag(Flag::new("num", FlagValue::IntVec(vec![]), "numbers"))
        .on_run(move |ctx| {
            *out2.lock().unwrap() = ctx.flags.get_int_vec("num").unwrap_or_default().to_vec();
        })
        .execute_with(args("--num 10 --num 20 --num 30"))
        .unwrap();
    assert_eq!(*out.lock().unwrap(), vec![10, 20, 30]);
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
fn flag_missing_value_returns_error() {
    let err = Command::new("app")
        .flag(Flag::new("name", FlagValue::String(String::new()), "name"))
        .on_run(|_| {})
        .execute_with(args("--name"))
        .unwrap_err();
    assert!(matches!(err, WrCliError::MissingFlagValue(n) if n == "name"));
}

#[test]
fn flag_short_missing_value_returns_error() {
    let err = Command::new("app")
        .flag(Flag::new("name", FlagValue::String(String::new()), "name").short('n'))
        .on_run(|_| {})
        .execute_with(args("-n"))
        .unwrap_err();
    assert!(matches!(err, WrCliError::MissingFlagValue(n) if n == "name"));
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

#[test]
fn flag_invalid_float_returns_error() {
    let err = Command::new("app")
        .flag(Flag::new("ratio", FlagValue::Float(0.0), "ratio"))
        .on_run(|_| {})
        .execute_with(args("--ratio not-a-float"))
        .unwrap_err();
    assert!(matches!(err, WrCliError::InvalidFlagValue { .. }));
}

#[test]
fn flag_bool_explicit_true_via_equals() {
    let out = Arc::new(Mutex::new(false));
    let out2 = out.clone();
    Command::new("app")
        .flag(Flag::new("verbose", FlagValue::Bool(false), "verbose"))
        .on_run(move |ctx| *out2.lock().unwrap() = ctx.flags.get_bool("verbose").unwrap())
        .execute_with(args("--verbose=true"))
        .unwrap();
    assert!(*out.lock().unwrap());
}

#[test]
fn flag_bool_unknown_value_defaults_to_false() {
    let out = Arc::new(Mutex::new(true));
    let out2 = out.clone();
    Command::new("app")
        .flag(Flag::new("verbose", FlagValue::Bool(true), "verbose"))
        .on_run(move |ctx| *out2.lock().unwrap() = ctx.flags.get_bool("verbose").unwrap())
        .execute_with(args("--verbose=maybe"))
        .unwrap();
    assert!(!*out.lock().unwrap());
}

#[test]
fn flag_is_set_returns_true_when_provided() {
    let is_set = Arc::new(Mutex::new(false));
    let is_set2 = is_set.clone();
    Command::new("app")
        .flag(Flag::new("port", FlagValue::Int(8080), "port"))
        .on_run(move |ctx| *is_set2.lock().unwrap() = ctx.flags.is_set("port"))
        .execute_with(args("--port 3000"))
        .unwrap();
    assert!(*is_set.lock().unwrap());
}

#[test]
fn flag_is_set_returns_false_when_default() {
    let is_set = Arc::new(Mutex::new(true));
    let is_set2 = is_set.clone();
    Command::new("app")
        .flag(Flag::new("port", FlagValue::Int(8080), "port"))
        .on_run(move |ctx| *is_set2.lock().unwrap() = ctx.flags.is_set("port"))
        .execute_with(args(""))
        .unwrap();
    assert!(!*is_set.lock().unwrap());
}

#[test]
fn flag_empty_value_after_equals_parsed_as_empty_string() {
    let out = Arc::new(Mutex::new(String::new()));
    let out2 = out.clone();
    Command::new("app")
        .flag(Flag::new("name", FlagValue::String(String::new()), "name"))
        .on_run(move |ctx| {
            *out2.lock().unwrap() = ctx.flags.get_string("name").unwrap_or("").to_owned()
        })
        .execute_with(args("--name="))
        .unwrap();
    assert_eq!(*out.lock().unwrap(), "");
}

#[test]
fn flag_value_starting_with_dash() {
    let out = Arc::new(Mutex::new(String::new()));
    let out2 = out.clone();
    Command::new("app")
        .flag(Flag::new("name", FlagValue::String(String::new()), "name"))
        .on_run(move |ctx| *out2.lock().unwrap() = ctx.flags.get_string("name").unwrap().to_owned())
        .execute_with(args("--name -foo"))
        .unwrap();
    assert_eq!(*out.lock().unwrap(), "-foo");
}

// ── hidden / deprecated / 제약 그룹 ──────────────────────────────────────────

#[test]
fn hidden_flag_still_parses() {
    let out = Arc::new(Mutex::new(false));
    let out2 = out.clone();
    Command::new("app")
        .flag(Flag::new("secret", FlagValue::Bool(false), "secret").hidden())
        .on_run(move |ctx| *out2.lock().unwrap() = ctx.flags.get_bool("secret").unwrap())
        .execute_with(args("--secret"))
        .unwrap();
    assert!(*out.lock().unwrap());
}

#[test]
fn mutually_exclusive_flags_reject_both() {
    let err = Command::new("app")
        .flag(Flag::new("json", FlagValue::Bool(false), "json"))
        .flag(Flag::new("yaml", FlagValue::Bool(false), "yaml"))
        .mutually_exclusive(&["json", "yaml"])
        .on_run(|_| {})
        .execute_with(args("--json --yaml"))
        .unwrap_err();
    assert!(matches!(err, WrCliError::MutuallyExclusiveFlags { .. }));
    let msg = err.to_string();
    assert!(msg.contains("--json"));
    assert!(msg.contains("--yaml"));
}

#[test]
fn mutually_exclusive_flags_allow_single() {
    Command::new("app")
        .flag(Flag::new("json", FlagValue::Bool(false), "json"))
        .flag(Flag::new("yaml", FlagValue::Bool(false), "yaml"))
        .mutually_exclusive(&["json", "yaml"])
        .on_run(|_| {})
        .execute_with(args("--json"))
        .unwrap();
}

#[test]
fn mutually_exclusive_flags_allow_none() {
    Command::new("app")
        .flag(Flag::new("json", FlagValue::Bool(false), "json"))
        .flag(Flag::new("yaml", FlagValue::Bool(false), "yaml"))
        .mutually_exclusive(&["json", "yaml"])
        .on_run(|_| {})
        .execute_with(args(""))
        .unwrap();
}

#[test]
fn required_together_flags_reject_partial() {
    let err = Command::new("app")
        .flag(Flag::new("user", FlagValue::String(String::new()), "user"))
        .flag(Flag::new("pass", FlagValue::String(String::new()), "pass"))
        .required_together(&["user", "pass"])
        .on_run(|_| {})
        .execute_with(args("--user alice"))
        .unwrap_err();
    assert!(matches!(err, WrCliError::RequiredFlagsTogether { .. }));
    assert!(err.to_string().contains("--pass"));
}

#[test]
fn required_together_flags_allow_all() {
    Command::new("app")
        .flag(Flag::new("user", FlagValue::String(String::new()), "user"))
        .flag(Flag::new("pass", FlagValue::String(String::new()), "pass"))
        .required_together(&["user", "pass"])
        .on_run(|_| {})
        .execute_with(args("--user alice --pass s3cret"))
        .unwrap();
}

#[test]
fn one_required_flags_reject_none() {
    let err = Command::new("app")
        .flag(Flag::new("json", FlagValue::Bool(false), "json"))
        .flag(Flag::new("yaml", FlagValue::Bool(false), "yaml"))
        .one_required(&["json", "yaml"])
        .on_run(|_| {})
        .execute_with(args(""))
        .unwrap_err();
    assert!(matches!(err, WrCliError::OneFlagRequired { .. }));
}

#[test]
fn one_required_flags_allow_one() {
    Command::new("app")
        .flag(Flag::new("json", FlagValue::Bool(false), "json"))
        .flag(Flag::new("yaml", FlagValue::Bool(false), "yaml"))
        .one_required(&["json", "yaml"])
        .on_run(|_| {})
        .execute_with(args("--yaml"))
        .unwrap();
}

#[test]
fn flag_constraints_ignore_config_seeded_values() {
    // 설정에서 시드된 값은 "사용자가 지정한 것"이 아니므로 제약을 발동시키지 않는다.
    Command::new("app")
        .flag(Flag::new("json", FlagValue::Bool(false), "json"))
        .flag(Flag::new("yaml", FlagValue::Bool(false), "yaml"))
        .mutually_exclusive(&["json", "yaml"])
        .with_config(
            wrcli::Config::new()
                .set_default("json", true)
                .set_default("yaml", true),
        )
        .on_run(|_| {})
        .execute_with(args(""))
        .unwrap();
}

#[test]
fn parent_local_constraint_does_not_break_subcommand() {
    // 부모 로컬 플래그에 걸린 one_required 제약은 서브커맨드 실행 시 오탐하면 안 된다.
    Command::new("app")
        .flag(Flag::new("mode", FlagValue::String(String::new()), "mode"))
        .one_required(&["mode"])
        .subcommand(Command::new("sub").on_run(|_| {}))
        .execute_with(args("sub"))
        .unwrap();
}

#[test]
fn persistent_constraint_applies_in_subcommand() {
    let err = Command::new("app")
        .persistent_flag(Flag::new("json", FlagValue::Bool(false), "json"))
        .persistent_flag(Flag::new("yaml", FlagValue::Bool(false), "yaml"))
        .mutually_exclusive(&["json", "yaml"])
        .subcommand(Command::new("sub").on_run(|_| {}))
        .execute_with(args("sub --json --yaml"))
        .unwrap_err();
    assert!(matches!(err, WrCliError::MutuallyExclusiveFlags { .. }));
}

// ── CSV 슬라이스 플래그 ─────────────────────────────────────────────────────

#[test]
fn comma_separated_string_vec_splits_values() {
    let out: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
    let out2 = out.clone();
    Command::new("app")
        .flag(Flag::new("tag", FlagValue::StringVec(vec![]), "tags").comma_separated())
        .on_run(move |ctx| {
            *out2.lock().unwrap() = ctx.flags.get_string_vec("tag").unwrap_or_default().to_vec();
        })
        .execute_with(args("--tag a,b,c"))
        .unwrap();
    assert_eq!(*out.lock().unwrap(), vec!["a", "b", "c"]);
}

#[test]
fn comma_separated_int_vec_splits_values() {
    let out: Arc<Mutex<Vec<i64>>> = Arc::new(Mutex::new(vec![]));
    let out2 = out.clone();
    Command::new("app")
        .flag(Flag::new("num", FlagValue::IntVec(vec![]), "numbers").comma_separated())
        .on_run(move |ctx| {
            *out2.lock().unwrap() = ctx.flags.get_int_vec("num").unwrap_or_default().to_vec();
        })
        .execute_with(args("--num 1,2,3"))
        .unwrap();
    assert_eq!(*out.lock().unwrap(), vec![1, 2, 3]);
}

#[test]
fn comma_separated_flag_trims_whitespace() {
    let out: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
    let out2 = out.clone();
    Command::new("app")
        .flag(Flag::new("tag", FlagValue::StringVec(vec![]), "tags").comma_separated())
        .on_run(move |ctx| {
            *out2.lock().unwrap() = ctx.flags.get_string_vec("tag").unwrap_or_default().to_vec();
        })
        .execute_with(vec!["--tag".to_owned(), "a, b ,c".to_owned()])
        .unwrap();
    assert_eq!(*out.lock().unwrap(), vec!["a", "b", "c"]);
}

#[test]
fn comma_separated_flag_supports_equals_and_repeats() {
    let out: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
    let out2 = out.clone();
    Command::new("app")
        .flag(Flag::new("tag", FlagValue::StringVec(vec![]), "tags").comma_separated())
        .on_run(move |ctx| {
            *out2.lock().unwrap() = ctx.flags.get_string_vec("tag").unwrap_or_default().to_vec();
        })
        .execute_with(args("--tag=a,b --tag c"))
        .unwrap();
    assert_eq!(*out.lock().unwrap(), vec!["a", "b", "c"]);
}

#[test]
fn vector_flag_without_comma_separated_keeps_value_intact() {
    let out: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
    let out2 = out.clone();
    Command::new("app")
        .flag(Flag::new("tag", FlagValue::StringVec(vec![]), "tags"))
        .on_run(move |ctx| {
            *out2.lock().unwrap() = ctx.flags.get_string_vec("tag").unwrap_or_default().to_vec();
        })
        .execute_with(args("--tag a,b"))
        .unwrap();
    assert_eq!(*out.lock().unwrap(), vec!["a,b"]);
}

#[test]
fn comma_separated_int_vec_rejects_invalid_element() {
    let err = Command::new("app")
        .flag(Flag::new("num", FlagValue::IntVec(vec![]), "numbers").comma_separated())
        .on_run(|_| {})
        .execute_with(args("--num 1,x,3"))
        .unwrap_err();
    assert!(matches!(err, WrCliError::InvalidFlagValue { .. }));
}

//! 서브커맨드 라우팅 및 persistent flag 통합 테스트.

mod common;
use common::args;

use std::sync::{Arc, Mutex};
use wrcli::args::exact_args;
use wrcli::{Command, Flag, FlagValue, WrCliError};

#[test]
#[should_panic(expected = "conflicts with existing subcommand")]
fn duplicate_subcommand_name_panics() {
    Command::new("app")
        .subcommand(Command::new("sub").on_run(|_| {}))
        .subcommand(Command::new("sub").on_run(|_| {}));
}

#[test]
#[should_panic(expected = "conflicts with existing subcommand")]
fn subcommand_alias_conflicting_with_existing_name_panics() {
    Command::new("app")
        .subcommand(Command::new("serve").on_run(|_| {}))
        .subcommand(Command::new("run").alias("serve").on_run(|_| {}));
}

#[test]
fn subcommand_basic_dispatch() {
    let ran = Arc::new(Mutex::new(false));
    let ran2 = ran.clone();
    Command::new("app")
        .subcommand(Command::new("sub").on_run(move |_| *ran2.lock().unwrap() = true))
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
            Command::new("config").subcommand(
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
    let flag_val = Arc::new(Mutex::new(false));
    let flag2 = flag_val.clone();
    Command::new("app")
        .persistent_flag(Flag::new("verbose", FlagValue::Bool(false), "verbose").short('v'))
        .subcommand(Command::new("sub").on_run(move |ctx| {
            *flag2.lock().unwrap() = ctx.flags.get_bool("verbose").unwrap_or(false)
        }))
        .execute_with(args("--verbose sub"))
        .unwrap();
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

#[test]
fn persistent_flag_visible_in_leaf() {
    let val = Arc::new(Mutex::new(false));
    let val2 = val.clone();
    Command::new("app")
        .persistent_flag(Flag::new("debug", FlagValue::Bool(false), "debug").short('d'))
        .subcommand(
            Command::new("sub").subcommand(Command::new("deep").on_run(move |ctx| {
                *val2.lock().unwrap() = ctx.flags.get_bool("debug").unwrap_or(false)
            })),
        )
        .execute_with(args("sub deep --debug"))
        .unwrap();
    assert!(*val.lock().unwrap());
}

#[test]
fn flag_value_matching_subcommand_name_is_not_misrouted() {
    // 회귀 테스트: --name의 값이 서브커맨드 이름과 같아도 서브커맨드로 오인되면 안 됨.
    let seen = Arc::new(Mutex::new(String::new()));
    let seen2 = seen.clone();
    Command::new("app")
        .flag(Flag::new("name", FlagValue::String(String::new()), "name"))
        .subcommand(Command::new("name").on_run(|_| {}))
        .on_run(move |ctx| {
            *seen2.lock().unwrap() = ctx.flags.get_string("name").unwrap_or("").to_owned();
        })
        .execute_with(args("--name name"))
        .unwrap();
    assert_eq!(*seen.lock().unwrap(), "name");
}

#[test]
fn double_dash_sentinel_prevents_subcommand_routing() {
    // 회귀 테스트: `--` 이후 토큰은 서브커맨드 이름과 같아도 리터럴 위치 인자로 취급.
    let positional = Arc::new(Mutex::new(Vec::<String>::new()));
    let p2 = positional.clone();
    Command::new("app")
        .subcommand(Command::new("sub").on_run(|_| {}))
        .on_run(move |ctx| *p2.lock().unwrap() = ctx.args.clone())
        .execute_with(args("-- sub"))
        .unwrap();
    assert_eq!(*positional.lock().unwrap(), vec!["sub".to_string()]);
}

#[test]
fn local_flag_not_visible_in_sibling_subcommand() {
    let err = Command::new("app")
        .subcommand(
            Command::new("a")
                .flag(Flag::new("local", FlagValue::Bool(false), "x"))
                .on_run(|_| {}),
        )
        .subcommand(Command::new("b").on_run(|_| {}))
        .execute_with(args("b --local"))
        .unwrap_err();
    assert!(matches!(err, WrCliError::UnknownFlag { .. }));
}

#[test]
fn hidden_subcommand_still_dispatches() {
    let ran = Arc::new(Mutex::new(false));
    let ran2 = ran.clone();
    Command::new("app")
        .subcommand(
            Command::new("secret")
                .hidden()
                .on_run(move |_| *ran2.lock().unwrap() = true),
        )
        .execute_with(args("secret"))
        .unwrap();
    assert!(*ran.lock().unwrap());
}

// ── 부모 로컬 플래그 (서브커맨드 앞) ─────────────────────────────────────────

#[test]
fn parent_local_flag_before_subcommand_is_parsed() {
    let out = Arc::new(Mutex::new(String::new()));
    let out2 = out.clone();
    Command::new("app")
        .flag(Flag::new("name", FlagValue::String(String::new()), "name"))
        .subcommand(Command::new("sub").on_run(move |ctx| {
            *out2.lock().unwrap() = ctx.flags.get_string("name").unwrap_or("").to_owned();
        }))
        .execute_with(args("--name Alice sub"))
        .unwrap();
    assert_eq!(*out.lock().unwrap(), "Alice");
}

#[test]
fn parent_local_flag_short_form_before_subcommand() {
    let out = Arc::new(Mutex::new(0i64));
    let out2 = out.clone();
    Command::new("app")
        .flag(Flag::new("count", FlagValue::Int(0), "count").short('c'))
        .subcommand(Command::new("sub").on_run(move |ctx| {
            *out2.lock().unwrap() = ctx.flags.get_int("count").unwrap_or(0);
        }))
        .execute_with(args("-c 7 sub"))
        .unwrap();
    assert_eq!(*out.lock().unwrap(), 7);
}

#[test]
fn parent_required_flag_not_enforced_for_subcommand() {
    Command::new("app")
        .flag(Flag::new("token", FlagValue::String(String::new()), "token").required())
        .subcommand(Command::new("sub").on_run(|_| {}))
        .execute_with(args("sub"))
        .unwrap();
}

#[test]
fn parent_local_flag_after_subcommand_is_unknown() {
    let err = Command::new("app")
        .flag(Flag::new("name", FlagValue::String(String::new()), "name"))
        .subcommand(Command::new("sub").on_run(|_| {}))
        .execute_with(args("sub --name Alice"))
        .unwrap_err();
    assert!(matches!(err, WrCliError::UnknownFlag { .. }));
}

#[test]
fn child_flag_overrides_parent_local_flag() {
    let out = Arc::new(Mutex::new(String::new()));
    let out2 = out.clone();
    Command::new("app")
        .flag(Flag::new(
            "name",
            FlagValue::String(String::new()),
            "parent name",
        ))
        .subcommand(
            Command::new("sub")
                .flag(Flag::new(
                    "name",
                    FlagValue::String(String::new()),
                    "child name",
                ))
                .on_run(move |ctx| {
                    *out2.lock().unwrap() = ctx.flags.get_string("name").unwrap_or("").to_owned();
                }),
        )
        .execute_with(args("--name parent sub --name child"))
        .unwrap();
    assert_eq!(*out.lock().unwrap(), "child");
}

#[test]
fn help_flag_after_parent_flag_still_works() {
    // 메타 플래그가 있으면 부모 플래그를 소비하지 않고 리프가 help를 처리한다.
    Command::new("app")
        .flag(Flag::new("name", FlagValue::String(String::new()), "name"))
        .subcommand(Command::new("sub").on_run(|_| {}))
        .execute_with(args("--name value --help sub"))
        .unwrap();
}

#[test]
fn parent_flag_value_does_not_leak_into_sibling_help() {
    // 부모 로컬 플래그는 하위 help의 Flags 목록에 나타나면 안 된다(정의는 상속되지 않음).
    let leaked = Arc::new(Mutex::new(false));
    let leaked2 = leaked.clone();
    Command::new("app")
        .flag(Flag::new("name", FlagValue::String(String::new()), "name"))
        .subcommand(Command::new("sub").on_run(move |ctx| {
            *leaked2.lock().unwrap() = ctx.flags.get_flag("name").is_some();
        }))
        .execute_with(args("--name value sub"))
        .unwrap();
    assert!(!*leaked.lock().unwrap());
}

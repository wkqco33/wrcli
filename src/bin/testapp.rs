//! Minimal binary used exclusively by `tests/binary.rs` (assert_cmd integration tests).
//!
//! Commands:
//! ```text
//! testapp greet <name> [--upper] [--count N]
//! testapp echo [args...]
//! testapp fail [--code N]
//! testapp tags [--tag <value>...] (StringVec flag demo)
//! testapp secret (hidden command)
//! testapp old (deprecated command)
//! ```

use wrcli::args::{arbitrary_args, minimum_n_args};
use wrcli::{Command, Flag, FlagValue, WrCliError};

fn main() {
    let result = Command::new("testapp")
        .version("0.1.0")
        .short("Test binary for wrcli assert_cmd tests")
        .persistent_flag(
            Flag::new("verbose", FlagValue::Bool(false), "enable verbose output").short('v'),
        )
        .flag(Flag::new("internal", FlagValue::Bool(false), "internal use only").hidden())
        .subcommand(
            Command::new("greet")
                .short("Print a greeting")
                .usage_args("<name>")
                .flag(Flag::new("upper", FlagValue::Bool(false), "uppercase output").short('u'))
                .flag(Flag::new("count", FlagValue::Int(1), "repeat count"))
                .flag(
                    Flag::new("legacy", FlagValue::Bool(false), "legacy mode")
                        .deprecated("use --upper"),
                )
                .args(minimum_n_args(1))
                .on_run(|ctx| {
                    let name = &ctx.args[0];
                    let upper = ctx.flags.get_bool("upper").unwrap_or(false);
                    let count = ctx.flags.get_int("count").unwrap_or(1);
                    for _ in 0..count {
                        let msg = format!("Hello, {}!", name);
                        println!("{}", if upper { msg.to_uppercase() } else { msg });
                    }
                    if ctx.flags.get_bool("verbose").unwrap_or(false) {
                        eprintln!("[verbose] greeted {}", name);
                    }
                }),
        )
        .subcommand(
            Command::new("echo")
                .short("Echo positional arguments")
                .args(arbitrary_args())
                .on_run(|ctx| {
                    println!("{}", ctx.args.join(" "));
                }),
        )
        .subcommand(
            Command::new("fail")
                .short("Exit with a non-zero code")
                .flag(Flag::new("code", FlagValue::Int(1), "exit code to use"))
                .on_run_e(|ctx| {
                    let code = ctx.flags.get_int("code").unwrap_or(1);
                    Err(WrCliError::ArgValidationFailed(format!(
                        "failing with code {}",
                        code
                    )))
                }),
        )
        .subcommand(
            Command::new("tags")
                .short("Collect repeated --tag flags (StringVec)")
                .flag(Flag::new(
                    "tag",
                    FlagValue::StringVec(vec![]),
                    "a tag (repeatable)",
                ))
                .on_run(|ctx| {
                    let tags = ctx.get_string_vec("tag").unwrap_or_default();
                    for tag in &tags {
                        println!("{}", tag);
                    }
                }),
        )
        .subcommand(
            Command::new("secret")
                .short("Hidden command")
                .hidden()
                .on_run(|_| println!("secret")),
        )
        .subcommand(
            Command::new("old")
                .short("Deprecated command")
                .deprecated("use greet instead")
                .on_run(|_| println!("old")),
        )
        .execute();

    if let Err(e) = result {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}

//! Minimal binary used exclusively by `tests/binary.rs` (assert_cmd integration tests).
//!
//! Commands:
//!   testapp greet <name> [--upper] [--count N]
//!   testapp echo [args...]
//!   testapp fail [--code N]
//!   testapp tags [--tag <value>...] (StringVec flag demo)

use wrcli::args::{arbitrary_args, minimum_n_args};
use wrcli::{Command, Flag, FlagValue, WrCliError};

fn main() {
    let result = Command::new("testapp")
        .version("0.1.0")
        .short("Test binary for wrcli assert_cmd tests")
        .persistent_flag(
            Flag::new("verbose", FlagValue::Bool(false), "enable verbose output").short('v'),
        )
        .subcommand(
            Command::new("greet")
                .short("Print a greeting")
                .flag(Flag::new("upper", FlagValue::Bool(false), "uppercase output").short('u'))
                .flag(Flag::new("count", FlagValue::Int(1), "repeat count"))
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
        .execute();

    if let Err(e) = result {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}

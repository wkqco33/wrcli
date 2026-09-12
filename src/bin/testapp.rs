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
    let cmd = Command::new("testapp")
        .version("0.1.0")
        .short("Test binary for wrcli assert_cmd tests")
        .standard_flags()
        .example("testapp greet Alice")
        .example("testapp greet Bob --upper --count 3")
        .support_url("https://github.com/wkqco33/wrcli/issues")
        .docs_url("https://docs.example.com/{command}")
        .persistent_flag(
            Flag::new("verbose", FlagValue::Bool(false), "enable verbose output").short('v'),
        )
        .flag(Flag::new("internal", FlagValue::Bool(false), "internal use only").hidden())
        .flag(
            Flag::new(
                "token",
                FlagValue::String("super-secret".to_owned()),
                "API token (sensitive)",
            )
            .sensitive(),
        )
        .subcommand(
            Command::new("greet")
                .short("Print a greeting")
                .usage_args("<name>")
                .example("testapp greet Alice")
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
            Command::new("list")
                .short("Print a table (honors --plain)")
                .on_run(|ctx| {
                    let table = wrcli::style::Table::new()
                        .headers(["Name", "Version"])
                        .row(["wrcli", "0.3.0"])
                        .row(["serde", "1.0"]);
                    if ctx.is_plain() {
                        print!("{}", table.render_plain());
                    } else {
                        table.print();
                    }
                }),
        )
        .subcommand(
            Command::new("messages")
                .short("Emit success/warning/info/error helpers")
                .on_run(|_| {
                    wrcli::style::print_success("done");
                    wrcli::style::print_warning("careful");
                    wrcli::style::print_info("note");
                    wrcli::style::print_error("bad");
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
        .subcommand(
            Command::new("sleep")
                .short("Sleep until interrupted (signal test)")
                .on_run(|_| std::thread::sleep(std::time::Duration::from_secs(30))),
        );

    let result = with_signal(cmd).execute();

    if let Err(e) = result {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}

/// Installs a Ctrl-C handler as well when the `signal` feature is enabled.
#[cfg(feature = "signal")]
fn with_signal(cmd: Command) -> Command {
    cmd.interrupt_message("interrupted by test\n")
}

#[cfg(not(feature = "signal"))]
fn with_signal(cmd: Command) -> Command {
    cmd
}

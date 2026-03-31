//! # wrcli — Cobra/Viper-inspired CLI framework for Rust
//!
//! Build CLI applications with a fluent, tree-based API inspired by Go's
//! [cobra](https://github.com/spf13/cobra) and [viper](https://github.com/spf13/viper) libraries.
//!
//! ## Quick start
//!
//! ```no_run
//! use wrcli::{Command, Flag, FlagValue, Config};
//! use wrcli::args::minimum_n_args;
//!
//! Command::new("myapp")
//!     .version("1.0.0")
//!     .short("My awesome CLI")
//!     .persistent_flag(
//!         Flag::new("verbose", FlagValue::Bool(false), "enable verbose output").short('v')
//!     )
//!     .subcommand(
//!         Command::new("greet")
//!             .short("Print a greeting")
//!             .args(minimum_n_args(1))
//!             .on_run(|ctx| {
//!                 for name in &ctx.args {
//!                     println!("Hello, {}!", name);
//!                 }
//!             })
//!     )
//!     .execute()
//!     .unwrap();
//! ```

pub mod command;
pub mod config;
pub mod error;
pub mod flag;
pub mod style;

// Flatten the most-used types to crate root for ergonomics.
pub use command::context::CommandContext;
pub use command::Command;
pub use config::{Config, ConfigValue};
pub use error::{Result, WrCliError};
pub use flag::{Flag, FlagSet, FlagValue};

/// Positional argument validators. Import with `use wrcli::args::*`.
pub mod args {
    pub use crate::command::args::{
        arbitrary_args, exact_args, maximum_n_args, minimum_n_args, no_args, range_args,
        valid_args, ArgValidator,
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    fn args(s: &str) -> Vec<String> {
        s.split_whitespace().map(str::to_owned).collect()
    }

    #[test]
    fn test_basic_run() {
        let called = Arc::new(Mutex::new(false));
        let called2 = called.clone();

        Command::new("app")
            .on_run(move |_ctx| {
                *called2.lock().unwrap() = true;
            })
            .execute_with(args(""))
            .unwrap();

        assert!(*called.lock().unwrap());
    }

    #[test]
    fn test_string_flag() {
        let result = Arc::new(Mutex::new(String::new()));
        let result2 = result.clone();

        Command::new("app")
            .flag(Flag::new("name", FlagValue::String(String::new()), "your name").short('n'))
            .on_run(move |ctx| {
                *result2.lock().unwrap() =
                    ctx.flags.get_string("name").unwrap_or("").to_owned();
            })
            .execute_with(args("--name Alice"))
            .unwrap();

        assert_eq!(*result.lock().unwrap(), "Alice");
    }

    #[test]
    fn test_int_flag() {
        let result = Arc::new(Mutex::new(0i64));
        let result2 = result.clone();

        Command::new("app")
            .flag(Flag::new("count", FlagValue::Int(0), "count"))
            .on_run(move |ctx| {
                *result2.lock().unwrap() = ctx.flags.get_int("count").unwrap_or(0);
            })
            .execute_with(args("--count 42"))
            .unwrap();

        assert_eq!(*result.lock().unwrap(), 42);
    }

    #[test]
    fn test_bool_flag_implicit_true() {
        let result = Arc::new(Mutex::new(false));
        let result2 = result.clone();

        Command::new("app")
            .flag(Flag::new("verbose", FlagValue::Bool(false), "verbose").short('v'))
            .on_run(move |ctx| {
                *result2.lock().unwrap() = ctx.flags.get_bool("verbose").unwrap_or(false);
            })
            .execute_with(args("--verbose"))
            .unwrap();

        assert!(*result.lock().unwrap());
    }

    #[test]
    fn test_bool_short_flag() {
        let result = Arc::new(Mutex::new(false));
        let result2 = result.clone();

        Command::new("app")
            .flag(Flag::new("verbose", FlagValue::Bool(false), "verbose").short('v'))
            .on_run(move |ctx| {
                *result2.lock().unwrap() = ctx.flags.get_bool("verbose").unwrap_or(false);
            })
            .execute_with(args("-v"))
            .unwrap();

        assert!(*result.lock().unwrap());
    }

    #[test]
    fn test_subcommand_dispatch() {
        let result = Arc::new(Mutex::new(String::new()));
        let result2 = result.clone();

        Command::new("app")
            .subcommand(
                Command::new("sub")
                    .on_run(move |ctx| {
                        *result2.lock().unwrap() = ctx.command_name().to_owned();
                    })
            )
            .execute_with(args("sub"))
            .unwrap();

        assert_eq!(*result.lock().unwrap(), "sub");
    }

    #[test]
    fn test_persistent_flag_propagation() {
        let result = Arc::new(Mutex::new(false));
        let result2 = result.clone();

        Command::new("app")
            .persistent_flag(Flag::new("debug", FlagValue::Bool(false), "debug").short('d'))
            .subcommand(
                Command::new("sub")
                    .on_run(move |ctx| {
                        *result2.lock().unwrap() = ctx.flags.get_bool("debug").unwrap_or(false);
                    })
            )
            .execute_with(args("sub --debug"))
            .unwrap();

        assert!(*result.lock().unwrap());
    }

    #[test]
    fn test_positional_args() {
        let result = Arc::new(Mutex::new(vec![]));
        let result2 = result.clone();

        Command::new("app")
            .on_run(move |ctx| {
                *result2.lock().unwrap() = ctx.args.clone();
            })
            .execute_with(args("foo bar baz"))
            .unwrap();

        assert_eq!(*result.lock().unwrap(), vec!["foo", "bar", "baz"]);
    }

    #[test]
    fn test_flag_equals_syntax() {
        let result = Arc::new(Mutex::new(String::new()));
        let result2 = result.clone();

        Command::new("app")
            .flag(Flag::new("output", FlagValue::String(String::new()), "output file"))
            .on_run(move |ctx| {
                *result2.lock().unwrap() =
                    ctx.flags.get_string("output").unwrap_or("").to_owned();
            })
            .execute_with(args("--output=result.txt"))
            .unwrap();

        assert_eq!(*result.lock().unwrap(), "result.txt");
    }

    #[test]
    fn test_config_defaults() {
        let config = Config::new().set_default("port", 8080i64);
        let result = Arc::new(Mutex::new(0i64));
        let result2 = result.clone();

        Command::new("app")
            .with_config(config)
            .on_run(move |ctx| {
                *result2.lock().unwrap() = ctx.config.get_int("port").unwrap_or(0);
            })
            .execute_with(args(""))
            .unwrap();

        assert_eq!(*result.lock().unwrap(), 8080);
    }

    #[test]
    fn test_arg_validation_exact() {
        let result = Command::new("app")
            .args(args::exact_args(2))
            .on_run(|_| {})
            .execute_with(args("a"));

        assert!(result.is_err());
    }

    #[test]
    fn test_run_e_error_propagation() {
        let result = Command::new("app")
            .on_run_e(|_ctx| {
                Err(WrCliError::ArgValidationFailed("test error".to_owned()))
            })
            .execute_with(args(""));

        assert!(result.is_err());
    }

    #[test]
    fn test_lifecycle_hooks_order() {
        let log = Arc::new(Mutex::new(vec![]));
        let l1 = log.clone();
        let l2 = log.clone();
        let l3 = log.clone();
        let l4 = log.clone();

        Command::new("app")
            .on_persistent_pre_run(move |_| l1.lock().unwrap().push("persistent_pre"))
            .on_pre_run(move |_| l2.lock().unwrap().push("pre"))
            .on_run(move |_| l3.lock().unwrap().push("run"))
            .on_post_run(move |_| l4.lock().unwrap().push("post"))
            .execute_with(args(""))
            .unwrap();

        assert_eq!(
            *log.lock().unwrap(),
            vec!["persistent_pre", "pre", "run", "post"]
        );
    }
}

//! Basic example demonstrating cobra/viper-style CLI with wrcli.
//!
//! Run:
//!   cargo run --example basic -- --help
//!   cargo run --example basic -- serve --port 9000
//!   cargo run --example basic -- config get server.host
//!   MYAPP_SERVER_PORT=3000 cargo run --example basic -- serve

use wrcli::args::{exact_args, no_args};
use wrcli::{Command, Config, Flag, FlagValue};

fn main() {
    env_logger::init();
    let config = Config::new()
        .set_config_name("myapp")
        .set_config_type("toml")
        .add_config_path(".")
        .set_default("server.port", 8080i64)
        .set_default("server.host", "127.0.0.1")
        .automatic_env()
        .set_env_prefix("MYAPP");

    let result = Command::new("myapp")
        .version("0.1.0")
        .short("Example CLI app")
        .long("A demonstration of wrcli — a cobra/viper-inspired CLI library for Rust.")
        // Persistent flag: propagates to all subcommands
        .persistent_flag(
            Flag::new("verbose", FlagValue::Bool(false), "enable verbose output").short('v'),
        )
        .with_config(config)
        .subcommand(
            Command::new("serve")
                .short("Start the HTTP server")
                .long("Start the HTTP server on the configured host and port.")
                .flag(Flag::new("port", FlagValue::Int(0), "port to listen on").short('p'))
                .flag(Flag::new(
                    "host",
                    FlagValue::String(String::new()),
                    "bind address",
                ))
                .args(no_args())
                .on_run(|ctx| {
                    let verbose = ctx.get_bool("verbose").unwrap_or(false);
                    // Flags take priority, then config, then defaults
                    let port = if ctx.flags.is_set("port") {
                        ctx.flags.get_int("port").unwrap()
                    } else {
                        ctx.config.get_int("server.port").unwrap_or(8080)
                    };
                    let host = if ctx.flags.is_set("host") {
                        ctx.flags
                            .get_string("host")
                            .unwrap_or("127.0.0.1")
                            .to_owned()
                    } else {
                        ctx.config
                            .get_string("server.host")
                            .unwrap_or_else(|| "127.0.0.1".to_owned())
                    };
                    if verbose {
                        println!("[verbose] Starting server...");
                    }
                    println!("Serving on {}:{}", host, port);
                }),
        )
        .subcommand(
            Command::new("config")
                .short("Inspect configuration")
                .subcommand(
                    Command::new("get")
                        .short("Get a config value by key")
                        .args(exact_args(1))
                        .on_run(|ctx| {
                            let key = &ctx.args[0];
                            match ctx.config.get_string(key) {
                                Some(v) => println!("{} = {}", key, v),
                                None => eprintln!("key not found: {}", key),
                            }
                        }),
                ),
        )
        .execute();

    if let Err(e) = result {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}

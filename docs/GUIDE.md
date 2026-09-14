# wrcli Guide

[English](GUIDE.md) | [한국어](GUIDE.ko.md)

---

## Table of Contents

- [Installation](#installation)
- [Commands](#commands)
- [Flags](#flags)
- [Help Conventions (clig.dev)](#help-conventions-cligdev)
- [Standard Flags and Output Formats](#standard-flags-and-output-formats)
- [Interactive Input and Confirmation Prompts](#interactive-input-and-confirmation-prompts)
- [Sensitive Flags](#sensitive-flags)
- [Optional-Value Flags](#optional-value-flags)
- [Standard I/O Substitution](#standard-io-substitution)
- [Color Policy and Pager](#color-policy-and-pager)
- [Ctrl-C (SIGINT) Handling](#ctrl-c-sigint-handling)
- [Hidden, Deprecated, and Flag Constraints](#hidden-deprecated-and-flag-constraints)
- [Positional Argument Validation](#positional-argument-validation)
- [Lifecycle Hooks](#lifecycle-hooks)
- [Configuration (Config)](#configuration-config)
  - [Default Values](#default-values)
  - [Config Files](#config-files)
  - [Config File Auto-Discovery](#config-file-auto-discovery)
  - [Automatic Config and Flag Binding](#automatic-config-and-flag-binding)
  - [Environment Variables](#environment-variables)
  - [Precedence Rules](#precedence-rules)
  - [Explicit Values and Aliases](#explicit-values-and-aliases)
  - [Key Delimiters and Env Settings](#key-delimiters-and-env-settings)
  - [Reading Config Values](#reading-config-values)
  - [Enumeration and Map Lookup](#enumeration-and-map-lookup)
  - [Runtime Reading and Merging](#runtime-reading-and-merging)
  - [Writing Config](#writing-config)
  - [Struct Deserialization](#struct-deserialization)
  - [Watching the Config File](#watching-the-config-file)
- [CommandContext](#commandcontext)
- [Generating Completion Scripts](#generating-completion-scripts)
  - [Dynamic Completion](#dynamic-completion)
- [Error Handling](#error-handling)
- [Writing Tests](#writing-tests)
- [Feature Flags](#feature-flags)

---

## Installation

```toml
[dependencies]
wrcli = "0.5"

# If you also need YAML
wrcli = { version = "0.5", features = ["yaml-config"] }

# Minimal build without config file support
wrcli = { version = "0.5", default-features = false }
```

### Referencing the Git Repository Directly

```toml
# SSH (recommended)
wrcli = { git = "git@github.com:wkqco33/wrcli.git" }

# Pin a branch / tag / commit
wrcli = { git = "git@github.com:wkqco33/wrcli.git", tag = "v0.5.0" }
wrcli = { git = "git@github.com:wkqco33/wrcli.git", rev = "a1b2c3d" }

# Local path (monorepo / during development)
wrcli = { path = "../wrcli" }
```

HTTPS authentication in a CI environment:

```yaml
- name: Configure git credentials
  run: |
    git config --global \
      url."https://x-access-token:${{ secrets.GITHUB_TOKEN }}@github.com/".insteadOf \
      "https://github.com/"
```

---

## Commands

Every command is assembled as a builder chain that starts with `Command::new("name")`.

```rust
Command::new("app")
    .short("One-line description (shown in the parent command's help listing)")
    .long("Long description (shown in this command's --help)")
    .on_run(|ctx| {
        println!("running!");
    })
    .execute()
    .unwrap();
```

### Subcommands and Aliases

```rust
Command::new("app")
    .subcommand(
        Command::new("deploy")
            .alias("d")        // also callable as `app d`
            .alias("ship")
            .short("Deploy the application")
            .on_run(|_| println!("Deploying...")),
    )
    .execute()
    .unwrap();
```

Nesting another `Command` inside a subcommand allows unbounded nesting.

### Usage Hint

`.usage_args("<name>")` displays a positional argument hint on the usage line of `--help`.

```rust
Command::new("greet")
    .usage_args("<name>")
    .on_run(|_| {})
    .execute()
    .unwrap();
```

```text
Usage:
  app greet <name> [flags]
```

### Version Flag

```rust
Command::new("app")
    .version("2.3.1")   // enables --version / -V automatically
    .on_run(|_| {})
    .execute()
    .unwrap();
```

```bash
$ app --version
app 2.3.1
```

---

## Flags

### Flag Types

| `FlagValue` variant | Rust type | Getter |
| ---------------- | --------- | ----------- |
| `Bool(bool)` | `bool` | `get_bool("name")` |
| `String(String)` | `String` | `get_string("name")` |
| `Int(i64)` | `i64` | `get_int("name")` |
| `Float(f64)` | `f64` | `get_float("name")` |
| `StringVec(Vec<String>)` | `Vec<String>` | `get_string_vec("name")` |
| `IntVec(Vec<i64>)` | `Vec<i64>` | use the raw `FlagValue` |

```rust
Command::new("app")
    .flag(Flag::new("output",  FlagValue::String(String::new()), "output file"))
    .flag(Flag::new("count",   FlagValue::Int(1),                "repeat count"))
    .flag(Flag::new("ratio",   FlagValue::Float(1.0),            "compression ratio"))
    .flag(Flag::new("verbose", FlagValue::Bool(false),           "verbose mode"))
    .on_run(|ctx| {
        let output  = ctx.flags.get_string("output").unwrap_or("out.txt");
        let count   = ctx.flags.get_int("count").unwrap_or(1);
        let ratio   = ctx.flags.get_float("ratio").unwrap_or(1.0);
        let verbose = ctx.flags.get_bool("verbose").unwrap_or(false);
    })
    .execute()
    .unwrap();
```

### Short Flags and Parsing Syntax

```rust
Flag::new("output", FlagValue::String(String::new()), "output file").short('o')
```

Supported parsing syntax:

```bash
--output result.txt   # long flag, space-separated
--output=result.txt   # long flag, = separated
-o result.txt         # short flag
-abc                  # bundled boolean flags (same as -a -b -c)
--verbose             # boolean flag (true when the value is omitted)
--verbose=false       # boolean flag, explicitly false
--                    # everything after is treated as positional arguments
```

### Required Flags

```rust
Flag::new("token", FlagValue::String(String::new()), "API token")
    .required()
```

If it is not provided, returns `WrCliError::MissingRequiredFlag`.

### Persistent Flags

Registering it on the root (or an intermediate) command makes it available in every subcommand automatically.

```rust
Command::new("app")
    .persistent_flag(
        Flag::new("config", FlagValue::String(String::new()), "config file path").short('c'),
    )
    .subcommand(
        Command::new("serve").on_run(|ctx| {
            let cfg_path = ctx.flags.get_string("config").unwrap_or("config.toml");
        }),
    )
    .execute()
    .unwrap();
```

### Repeatable Vector Flags

`StringVec` / `IntVec` flags accumulate values when the same name is given more than once.

```rust
Command::new("app")
    .flag(Flag::new("tag", FlagValue::StringVec(vec![]), "add a tag (repeatable)"))
    .on_run(|ctx| {
        let tags = ctx.get_string_vec("tag").unwrap_or_default();
        for tag in &tags { println!("tag: {}", tag); }
    })
    .execute()
    .unwrap();
```

```bash
$ app --tag frontend --tag prod --tag v2
tag: frontend
tag: prod
tag: v2
```

Adding `.comma_separated()` splits `--tag a,b,c` into multiple values (whitespace is trimmed and empty items are ignored). Nothing is split by default.

```rust
Flag::new("tag", FlagValue::StringVec(vec![]), "tags").comma_separated()
```

```bash
$ app --tag frontend,prod --tag=v2
tag: frontend
tag: prod
tag: v2
```

### Parent-Local Flags (Before the Subcommand)

A flag on a parent command that is not `persistent` is also consumed by the parent when it is given **before the subcommand name**, and its value can be read from the leaf context as well. A parent-local flag given after the subcommand name is treated as an unknown flag.

```rust
Command::new("app")
    .flag(Flag::new("profile", FlagValue::String(String::new()), "profile name"))
    .subcommand(Command::new("deploy").on_run(|ctx| {
        // `app --profile prod deploy` → "prod"
        let profile = ctx.flags.get_string("profile").unwrap_or("dev");
        println!("deploying with {profile}");
    }))
    .execute()
    .unwrap();
```

```bash
app --profile prod deploy   # OK
app deploy --profile prod   # error: unknown flag
```

Parent-local flags are not inherited into the help/completion listing, so they do not appear in the leaf's `Flags` section. When `--help`/`--version` is included, the parent does not consume the flags and the leaf handles them as-is.

---

## Help Conventions (clig.dev)

### Built-in `help` Subcommand

If you have not registered `help` as a subcommand yourself, the built-in `help` works automatically. If the user registers `help`, the built-in behavior is disabled.

```sh
myapp help                # root help
myapp help config         # subcommand help (aliases work too)
myapp help config get     # nested path
```

If you give a name that does not exist in the middle of the path, it produces an `UnknownSubcommand` error together with a suggestion (`Did you mean`). `help` is also included as a candidate in completion scripts.

### Examples, Support Links, and Documentation Links

clig.dev recommends “putting examples up front” and including a feedback path and web documentation links in your help. Examples are printed immediately after Usage.

```rust
Command::new("myapp")
    .short("My CLI")
    .example("myapp greet Alice")
    .example("myapp greet Bob --upper --count 3")
    .support_url("https://github.com/me/myapp/issues")
    .docs_url("https://docs.example.com/{command}") // {command} substitution
```

`docs_url`/`support_url` are inherited unless a subcommand defines them itself. So `myapp greet --help` shows a link to `https://docs.example.com/myapp greet`.

### Commands Without a Runner

Running a parent command that only has subcommands with no arguments prints help and exits **successfully (exit code 0)**. A leaf command with neither subcommands nor a runner is usually a mistake, so it produces a `CommandHasNoRunner` error. To show only help and still exit successfully at a leaf as well, use `help_on_missing_runner()`.

```rust
Command::new("myapp").short("My CLI").help_on_missing_runner()
```

### Bug Report URL

When `bug_report_url` is set, `execute_or_exit()` points users to that URL for unexpected errors that are not usage errors.

```rust
Command::new("myapp").bug_report_url("https://github.com/me/myapp/issues/new")
```

---

## Standard Flags and Output Formats

`standard_flags()` registers in one call the conventions that clig.dev mentions over and over. They are persistent flags, so they propagate to every subcommand, and both `app --plain list` and `app list --plain` work.

| Flag | Short | Accessor |
| ---- | ---- | ---- |
| `--quiet` | `-q` | `ctx.is_quiet()` |
| `--force` | `-f` | `ctx.is_force()` |
| `--no-input` | | `ctx.no_input()` |
| `--no-color` | | |
| `--plain` | | `ctx.is_plain()` |
| `--json` | | `ctx.is_json()` |
| `--color <when>` | | `style::color_choice()` |
| `--confirm <name>` | | `ctx.confirm_severe()` |

`--plain` and `--json` are validated with `mutually_exclusive`.

```rust
use wrcli::{Command, OutputFormat};

Command::new("myapp")
    .standard_flags()
    .on_run(|ctx| {
        match ctx.output_format() {
            OutputFormat::Human => { /* styled table */ }
            OutputFormat::Plain => { /* one record per line */ }
            OutputFormat::Json => { /* JSON */ }
        }
    });
```

A table can be rendered with `Table::render_plain()` as tab-separated, one line per record, with no borders.

```rust
use wrcli::style::Table;
let tsv = Table::new()
    .headers(["Name", "Version"])
    .row(["wrcli", "0.5.0"])
    .render_plain();
assert_eq!(tsv, "Name\tVersion\nwrcli\t0.5.0\n");
```

---

## Interactive Input and Confirmation Prompts

clig.dev: “Never require a prompt”, “Only use prompts if stdin is an interactive terminal”, “Confirm before doing anything dangerous”.

```rust
.on_run_e(|ctx| {
    if ctx.confirm("Delete 3 items?")? {
        // when --force is set or the user typed y
    }
    ctx.confirm_severe("myapp")?;   // --confirm="myapp" or typing the name directly
    let pw = ctx.prompt_password("Password: ")?; // unix: stty -echo
    Ok(())
})
```

Behavior rules:

- If `--force` (`-f`) is present, `confirm()` returns `true` without prompting.
- If `--confirm="<name>"` matches, `confirm_severe()` returns `true`. If the value differs, `ConfirmationFailed`.
- If `--no-input` is set or stdin is not a TTY, it raises an `InteractiveInputRequired` error instead of prompting and points to the flags you can use instead (`--force`, `--confirm="<name>"`). (It does not hang the way `cat` does.)
- You can check whether prompting is possible yourself with `ctx.is_interactive()`.
- On unix, `prompt_password()` hides input with `stty -echo`. On other platforms, echo cannot be turned off.

---

## Sensitive Flags

clig.dev recommends not accepting secrets directly as flags. When there is no way around it, at least add `sensitive()` so the value does not leak into help or error messages.

```rust
Flag::new("token", FlagValue::String(String::new()), "API token").sensitive()
```

- The default value is omitted from help.
- The value is masked as `***` in value-conversion error messages.

Secrets are best read from `--password-file` or stdin.

---

## Optional-Value Flags

A flag with an optional value interprets the special word `none` as “no value” (an empty string). clig.dev: “allow a special word like 'none'. Don't just use a blank value.”

```rust
Flag::new("config", FlagValue::String("/etc/app.toml".to_owned()), "config path").optional_value()
// --config /tmp/x.toml  -> "/tmp/x.toml"
// --config none        -> ""
```

---

## Standard I/O Substitution

clig.dev: “If input or output is a file, support `-` to read from stdin or write to stdout.”

```rust
use wrcli::io::{open_reader, open_writer, read_to_string};
use std::io::{Read, Write};

fn cat(path: &str) -> wrcli::Result<()> {
    let mut buf = String::new();
    open_reader(path)?.read_to_string(&mut buf)?;   // "-" -> stdin
    open_writer("-")?.write_all(buf.as_bytes())?;   // "-" -> stdout
    Ok(())
}
```

`read_to_string(path)` reads everything from stdin when the path is `-`.

---

## Color Policy and Pager

Whether color is used follows the clig.dev rules. The precedence is as follows.

1. Global override (`--no-color` = Never, `--color=always|never|auto`)
2. `FORCE_COLOR` (non-empty)
3. `NO_COLOR` (non-empty)
4. `TERM=dumb`
5. App-specific `*_NO_COLOR`
6. Whether stdout/stderr is a TTY

```rust
use wrcli::style::{ColorChoice, set_color_choice, set_no_color_env, stdout_is_styled};

set_no_color_env(Some("MYAPP_NO_COLOR"));
set_color_choice(ColorChoice::Never);
assert!(!stdout_is_styled());
```

`stdout_is_terminal()` / `stdin_is_terminal()` let you check TTY-ness independently of color.

Passing long output to `style::pager::page(&text)` sends it to `PAGER` (default `less -FIRX`) only when stdout is a TTY, and outputs it as-is in a pipe/CI.

```rust
wrcli::style::pager::page(&long_text)?;
```

`Progress::draw()` / `Progress::finish()` also do not use animation (`\r`) on non-TTY output and print only the final state on a single line.

---

## Ctrl-C (SIGINT) Handling

With the `signal` feature enabled, on Ctrl-C you can print a message immediately and exit with code `130`. The handler performs only async-signal-safe operations and does no clean-up work (crash-only).

```toml
wrcli = { version = "0.5", features = ["signal"] }
```

```rust
Command::new("myapp")
    .interrupt_message("interrupted\n")
    .on_run(|_| { /* long-running work */ });
```

To use it without the feature, you can also call `wrcli::signal::install(msg)` directly.

---

## Hidden, Deprecated, and Flag Constraints

### Hidden

Commands and flags marked with `.hidden()` are left out of `--help` and completion scripts, but parsing and execution still work as before. Use this for internal or experimental features.

```rust
Command::new("app")
    .flag(Flag::new("internal", FlagValue::Bool(false), "internal use only").hidden())
    .subcommand(Command::new("secret").hidden().on_run(|_| println!("secret")))
    .execute()
    .unwrap();
```

Hidden entries are also excluded from typo suggestions (`Did you mean`).

### Deprecated

When you set `.deprecated("message")`, a warning is printed to stderr when that command/flag is actually used. Execution itself continues.

```rust
Command::new("app")
    .flag(Flag::new("old", FlagValue::Bool(false), "legacy").deprecated("use --new"))
    .subcommand(Command::new("legacy").deprecated("use `app new`").on_run(|_| {}))
    .execute()
    .unwrap();
```

```bash
$ app legacy
Command "legacy" is deprecated: use `app new`
```

### Flag Constraint Groups

Declaring these three constraints validates them just before a leaf command runs. Values seeded from config are not counted as "specified by the user", so they do not trigger the constraints.

```rust
Command::new("app")
    .flag(Flag::new("json", FlagValue::Bool(false), "JSON output"))
    .flag(Flag::new("yaml", FlagValue::Bool(false), "YAML output"))
    .flag(Flag::new("user", FlagValue::String(String::new()), "user"))
    .flag(Flag::new("pass", FlagValue::String(String::new()), "password"))
    .mutually_exclusive(&["json", "yaml"])   // error if both are specified
    .required_together(&["user", "pass"])    // error if only some are specified
    .one_required(&["json", "yaml"])         // at least one required
    .on_run(|_| {})
    .execute()
    .unwrap();
```

| Violation | Error variant |
| ------- | --------- |
| More than one specified | `MutuallyExclusiveFlags` |
| Only some specified | `RequiredFlagsTogether` |
| None specified | `OneFlagRequired` |

### Typo Suggestions (Did you mean)

For an unregistered command/flag, edit-distance-based candidates are included in the error message. `.suggest_for("alias")` adds a **suggestion-only** name that never runs (Cobra's `SuggestFor`).

```rust
Command::new("app")
    .subcommand(
        Command::new("remove")
            .suggest_for("delete")   // `app delete` → "Did you mean: remove"
            .on_run(|_| {}),
    )
    .execute()
    .unwrap();
```

Hidden (`hidden`) entries are excluded from suggestions.

---

## Positional Argument Validation

The `wrcli::args` module ships built-in validators.

```rust
use wrcli::args::{no_args, arbitrary_args, exact_args,
                  minimum_n_args, maximum_n_args, range_args, valid_args};

Command::new("copy")
    .args(exact_args(2))
    .on_run(|ctx| {
        let src = &ctx.args[0];
        let dst = &ctx.args[1];
    })
```

| Function | Description |
| ---- | ---- |
| `no_args()` | No positional arguments |
| `arbitrary_args()` | No restriction |
| `exact_args(n)` | Exactly n arguments |
| `minimum_n_args(n)` | At least n arguments |
| `maximum_n_args(n)` | At most n arguments |
| `range_args(min, max)` | At least min and at most max |
| `valid_args(vec![...])` | Only values contained in the allow list |

Custom validator:

```rust
use wrcli::args::ArgValidator;
use wrcli::error::WrCliError;

fn only_existing_files() -> ArgValidator {
    Box::new(|args| {
        for arg in args {
            if !std::path::Path::new(arg).exists() {
                return Err(WrCliError::ArgValidationFailed(
                    format!("file not found: {}", arg)
                ));
            }
        }
        Ok(())
    })
}
```

---

## Lifecycle Hooks

```bash
persistent_pre_run  (chained in root → leaf order)
pre_run             (matched leaf command only)
run / run_e         (matched leaf command only)
post_run            (matched leaf command only)
persistent_post_run (chained in leaf → root order)
```

If `on_run_e` returns `Err`, `post_run` / `persistent_post_run` do not run.

```rust
Command::new("app")
    .on_persistent_pre_run(|_| println!("always runs: init"))
    .subcommand(
        Command::new("deploy")
            .on_pre_run(|_| println!("pre-deploy validation"))
            .on_run_e(|_| {
                do_deploy()?;
                Ok(())
            })
            .on_post_run(|_| println!("deploy finished notification")),
    )
    .on_persistent_post_run(|_| println!("always runs: cleanup"))
    .execute()
    .unwrap();
```

---

## Configuration (Config)

The configuration store that corresponds to Go's Viper. Attaching it to the root command with `.with_config(config)` makes it reachable from every subcommand through `ctx.config`.

### Default Values

```rust
let config = Config::new()
    .set_default("server.host", "127.0.0.1")
    .set_default("server.port", 8080i64)
    .set_default("debug",       false);
```

### Config Files

```rust
let mut config = Config::new()
    .set_config_name("myapp")          // file name (without extension)
    .set_config_type("toml")           // toml | json | yaml | ini | env(dotenv) | properties
    .add_config_path(".")              // search directory (multiple allowed)
    .add_config_path("~/.config/myapp");

config.read_in_config().ok();          // ignored if the file is missing
```

TOML example:

```toml
[server]
host = "0.0.0.0"
port = 9000

[database]
url = "postgres://localhost/mydb"
```

Nested keys are accessed with dot notation: `config.get_string("server.host")`.

### Config File Auto-Discovery

If `config_type` is not specified, **every enabled supported format** (TOML, JSON, YAML, INI, dotenv, properties) is tried in order. It also detects standard locations automatically even when you do not specify a search path yourself.

**Search order** (Viper style):

```text
1. Paths added with add_config_path (if specified)
2. $XDG_CONFIG_HOME/<name>  or  ~/.config/<name>
3. ~/.<name>
4. Current directory (.)
```

```rust
let mut config = Config::new()
    .set_config_name("myapp");   // type unset → auto-detect, path unset → auto-discover

config.read_in_config().ok();    // searched in ~/.config/myapp/{toml,json,yaml,ini,...} and so on
```

**Specifying a single file directly** — `set_config_file`:

```rust
let mut config = Config::new()
    .set_config_file("~/.config/myapp/custom.toml");   // format auto-detected from the extension

config.read_in_config()?;
```

`set_config_file` loads straight from that path, regardless of the config name/type/search path.

### Automatic Config and Flag Binding

Flags that were not explicitly set are **automatically seeded with the value from the config store**. That is, the value is looked up in config files/environment variables/defaults and injected into the flag, so `ctx.flags.get_*()` and `ctx.get_*()` return consistent values.

```rust
// when config has server.port = 9000
Command::new("app")
    .flag(Flag::new("port", FlagValue::Int(0), "port"))
    .with_config(config)              // config already loaded
    .on_run(|ctx| {
        // 9000 is used if the flag was not given explicitly
        let port = ctx.flags.get_int("port").unwrap();
    })
    .execute_with(vec![])             // explicit input like "--port 5000" takes precedence
    .unwrap();
```

**Precedence**: explicit CLI flag > config (file/environment variable/default). If a flag conflicts with a config value of a mismatched type, the config value is ignored.

### Environment Variables

```rust
let config = Config::new()
    .automatic_env()
    .set_env_prefix("MYAPP");
```

With `automatic_env()` + `set_env_prefix("MYAPP")` applied:

| Config key | Environment variable |
| ------- | --------- |
| `server.port` | `MYAPP_SERVER_PORT` |
| `database.url` | `MYAPP_DATABASE_URL` |
| `debug` | `MYAPP_DEBUG` |

Binding a specific key to an environment variable explicitly:

```rust
config.bind_env("token", "API_TOKEN");
```

### Precedence Rules

```text
1. Defaults       (set_default)               ← lowest
2. Config file    (read_in_config)
3. Environment    (automatic_env / bind_env)
4. CLI flags      (values the user actually entered)
5. Explicit value (set)                       ← highest
```

CLI flag defaults are not injected. Only values the user actually specified override the config. `set` takes precedence over flags, and `is_set(key)` tells you whether a value exists in any layer.

### Explicit Values and Aliases

```rust
let config = Config::new()
    .set_default("server.port", 8080i64)
    .set("server.port", 9000i64)          // highest-priority layer
    .register_alias("port", "server.port"); // also looked up as "port"

assert!(config.is_set("port"));
assert_eq!(config.get_int("port"), Some(9000));
```

Aliases can be chained (`a` → `b` → the canonical key).

### Key Delimiters and Env Settings

```rust
let config = Config::new()
    .set_key_delimiter('/')                  // nested access as "server/port"
    .set_env_key_replacer(&[(".", "__")])   // applied in order to uppercase keys
    .allow_empty_env(false);                 // treat empty env vars as unset (Viper default)
```

Defaults: delimiter `'.'`, replacer `.`/`-` → `_`, `allow_empty_env = true` (empty values are used as well).

### Reading Config Values

```rust
ctx.config.get_string("server.host")      // Option<String>
ctx.config.get_int("server.port")         // Option<i64> (same as get_int64)
ctx.config.get_uint("workers")            // Option<u64>
ctx.config.get_bool("debug")              // Option<bool>
ctx.config.get_float("ratio")             // Option<f64>
ctx.config.get_string_vec("allowed.ips")  // Option<Vec<String>> (same as get_string_slice)
ctx.config.get_duration("timeout")        // Option<Duration> — "1h30m", "250ms", number = seconds
ctx.config.get_time("started_at")         // Option<SystemTime> — RFC3339 or Unix seconds
ctx.config.get_size_in_bytes("max_body")  // Option<u64> — "1.5MB", "2GiB" (base 1024)
```

### Enumeration and Map Lookup

```rust
let keys = ctx.config.all_keys();          // Vec<String>, sorted (Viper AllKeys)
let tree = ctx.config.all_settings();      // SettingsMap — rebuilds dot keys as nesting

// a key's sub-values as a map
let server = ctx.config.get_string_map_string("server");      // BTreeMap<String, String>
let ips = ctx.config.get_string_map_string_slice("allowed");  // BTreeMap<String, Vec<String>>

// a Config holding only the subtree
let sub = ctx.config.sub("server");
let host = sub.get_string("host");
```

### Runtime Reading and Merging

```rust
let mut cfg = Config::new().set_config_type("toml");
cfg.read_config("[a]\nb = 1\n".as_bytes())?;   // replace the file layer
cfg.merge_in_config("extra.toml")?;            // merge while keeping existing keys
cfg.merge_config_map([("c".to_owned(), ConfigValue::Int(3))]);
```

`read_config` requires the format to be specified first with [`set_config_type`](Config::set_config_type), and returns `WrCliError::ConfigTypeNotSet` when it is not specified.

### Writing Config

```rust
// save the current config (all layers merged) to a file (format from the extension)
ctx.config.write_config_as("out.toml")?;

// fails if the target file already exists
ctx.config.safe_write_config_as("out.json")?; // Err(ConfigFileExists)
```

Supported write formats: TOML, JSON, INI, dotenv, properties (when the corresponding feature is enabled).

### Struct Deserialization

With the `serde` feature enabled, config can be mapped straight onto structs.

```rust
#[derive(serde::Deserialize)]
struct Server {
    host: String,
    port: i64,
}

let server: Server = ctx.config.unmarshal_key("server")?; // subtree
let app: App = ctx.config.unmarshal()?;                    // whole tree
```

Supported: struct/map, `Vec`, `Option`, primitive types, unit enum variants.

### Watching the Config File

```rust
let mut cfg = Config::new()
    .set_config_file("app.toml")
    .set_watch_interval(Duration::from_millis(500))
    .on_config_change(|reloaded| {
        println!("port = {:?}", reloaded.get_int("port"));
    });
cfg.read_in_config()?;

let _watcher = cfg.watch_config()?; // watching stops on drop
```

It is polling-based (1 second by default), and `on_config_change` and `read_in_config` must come first. If they are missing, it returns `WrCliError::ConfigWatchNotReady`.

---

## CommandContext

The `&CommandContext` passed to `on_run` / `on_run_e` callbacks bundles flags, config, and positional arguments together.

```rust
.on_run(|ctx| {
    // positional arguments
    let first = &ctx.args[0];

    // look up flags only
    let verbose = ctx.flags.get_bool("verbose").unwrap_or(false);

    // automatic lookup in flag → config order
    let host = ctx.get_string("server.host").unwrap_or_default();
    let port = ctx.get_int("server.port").unwrap_or(8080);

    // conversion getters (flag → config)
    let _limit = ctx.get_uint("limit");
    let _timeout = ctx.get_duration("timeout");
    let _max_size = ctx.get_size_in_bytes("max_size");
    let _nums = ctx.get_int_vec("nums");
    let _server = ctx.get_string_map("server");

    // whether explicitly set on a flag or present in config
    let explicit = ctx.is_set("port");

    // current command path (e.g. ["myapp", "config", "get"])
    println!("{}", ctx.command_name());
    println!("{:?}", ctx.command_path);
    let _ = explicit;
})
```

`ctx.get_*(key)` is handy when the flag name and the config key are the same. `get_duration` / `get_time` / `get_size_in_bytes` / `get_string_map` look the value up in config when the flag does not have that type. `is_set` returns `true` for a flag specified on argv or for a key that exists in config.

---

## Generating Completion Scripts

`Command::gen_completion(shell)` generates a completion script for bash / zsh / fish. It collects subcommands and flags recursively.

```rust
let bash_script = Command::new("myapp")
    .subcommand(Command::new("serve"))
    .gen_completion("bash")
    .unwrap();

std::fs::write("myapp.bash", bash_script)?;
```

Supported shells: `"bash"`, `"zsh"`, `"fish"`. Any other shell returns `WrCliError::UnsupportedCompletionShell`.

There is no built-in install subcommand (such as `gen-completion`). To generate at runtime, build the command tree and register your own subcommand that calls `gen_completion`.

```bash
# when a custom subcommand prints the script to stdout
myapp gen-completion bash > /etc/bash_completion.d/myapp
```

### Dynamic Completion

`gen_completion` bakes the command tree in statically. When the candidates need to vary with config, files, or server state, use the dynamic API.

- `Command::complete(&[String])` — returns candidates for the last token (the one being completed). It automatically collects subcommand, flag, and `arg_candidates` candidates and filters them by prefix.
- `Command::completion_request(Vec<String>)` — a `main` entry-point helper that returns the candidates as `Some` when the first token is `__complete`.
- `Command::arg_candidates(f)` — registers a function that builds positional argument candidates.

```rust
fn main() {
    let cmd = Command::new("myapp")
        .subcommand(
            Command::new("run")
                .arg_candidates(|prior| {
                    if prior.is_empty() {
                        vec!["build".into(), "test".into(), "deploy".into()]
                    } else {
                        Vec::new()
                    }
                })
                .on_run(|_| {}),
        );

    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Some(candidates) = cmd.completion_request(args) {
        for c in candidates {
            println!("{}", c);
        }
        return;
    }

    cmd.execute().unwrap();
}
```

```bash
$ myapp __complete run ""
build
test
deploy
```

To make the generated script call this protocol, fill `COMPREPLY` with the result of `"$1" __complete "${COMP_WORDS[@]:1}"`.

---

## Error Handling

`.execute()` returns `Result<(), WrCliError>`.

| Variant | When it occurs |
| ---- | --------- |
| `UnknownFlag` | Use of an unregistered flag |
| `UnknownSubcommand` | Use of an unregistered subcommand |
| `MissingRequiredFlag` | A `.required()` flag was not provided |
| `MissingFlagValue` | No value was given to a flag that requires one (e.g. `--name`) |
| `InvalidFlagValue` | Type mismatch (e.g. `--count abc`) |
| `ArgValidationFailed` | Positional argument validation failed |
| `CommandHasNoRunner` | Execution of a command with no `on_run` registered |
| `ConfigFileNotFound` | The config file could not be found |
| `ConfigParseError` | Failed to parse the config file |
| `ConfigTypeNotSet` | No format was specified for `read_config` |
| `ConfigFileExists` | The target file for `safe_write_config_as` already exists |
| `ConfigDeserializeError` | `unmarshal` deserialization failed |
| `ConfigWatchNotReady` | The prerequisites for `watch_config` are not met |
| `MutuallyExclusiveFlags` | A `mutually_exclusive` group was violated |
| `RequiredFlagsTogether` | A `required_together` group was violated |
| `OneFlagRequired` | A `one_required` group was violated |
| `UnsupportedConfigFormat` | Use of a config format that is not enabled |
| `UserError` | An error returned from `on_run_e` |
| `Io` | An I/O failure such as reading a config file |
| `UnsupportedCompletionShell` | Completion generation for an unsupported shell |

`UnknownFlag` / `UnknownSubcommand` include edit-distance-based typo suggestions in the message.

```text
unknown command 'gret' for 'app'  Run with --help for available commands.

Did you mean this?
 greet
```

### Exit Codes

You can classify errors with `WrCliError::is_usage_error()` and `WrCliError::exit_code()`. Usage errors (unregistered flag/command, missing required flag, type error, constraint violation, and so on) are **2**, and all other execution errors are **1**.

The simplest pattern is `Command::execute_or_exit()`. It prints the error to stderr as `Error: ...` and exits with the appropriate code.

```rust
fn main() {
    build_cli().execute_or_exit(); // 0 on success, 1 or 2 on error
}
```

To control it manually:

```rust
fn main() {
    if let Err(e) = build_cli().execute() {
        eprintln!("error: {}", e);
        std::process::exit(e.exit_code());
    }
}
```

To return an arbitrary error type from `on_run_e`, use `WrCliError::user(e)`:

```rust
.on_run_e(|_| {
    let data = std::fs::read_to_string("data.txt")
        .map_err(WrCliError::user)?;
    Ok(())
})
```

---

## Writing Tests

### Unit Tests — `execute_with()`

```rust
#[test]
fn test_greet_command() {
    use std::sync::{Arc, Mutex};

    let output = Arc::new(Mutex::new(String::new()));
    let out2   = output.clone();

    Command::new("app")
        .flag(Flag::new("name", FlagValue::String(String::new()), "name").short('n'))
        .on_run(move |ctx| {
            *out2.lock().unwrap() =
                ctx.flags.get_string("name").unwrap_or("").to_owned();
        })
        .execute_with(vec!["--name".into(), "Alice".into()])
        .unwrap();

    assert_eq!(*output.lock().unwrap(), "Alice");
}
```

### Binary Tests — `assert_cmd`

Run a real process and verify stdout / stderr / exit code.

```toml
[dev-dependencies]
assert_cmd = "2"
predicates = "3"
```

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn greet_basic() {
    Command::cargo_bin("myapp").unwrap()
        .args(["greet", "Alice"])
        .assert()
        .success()
        .stdout("Hello, Alice!\n");
}

#[test]
fn unknown_flag_fails() {
    Command::cargo_bin("myapp").unwrap()
        .args(["--no-such-flag"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown flag"));
}
```

---

## Feature Flags

| Feature | Enabled by default | Description |
| ---- | :---------: | ---- |
| `toml-config` | ✅ | TOML config file support |
| `json-config` | ✅ | JSON config file support |
| `yaml-config` | ❌ | YAML config file support (`noyalib`) |
| `ini-config` | ❌ | INI config file support |
| `dotenv-config` | ❌ | `.env` / dotenv config file support |
| `properties-config` | ❌ | Java properties config file support |
| `serde` | ❌ | `unmarshal`/`unmarshal_key` struct deserialization |
| `signal` | ❌ | Ctrl-C(SIGINT) handler (`interrupt_message`) |

```toml
# enable all formats
wrcli = { version = "0.5", features = ["yaml-config", "ini-config", "dotenv-config", "properties-config"] }

# minimal build (no config file support)
wrcli = { version = "0.5", default-features = false }
```

Styling (`Style`, `Table`, `Panel`, `Rule`, `Tree`, `Text`, `Progress`, and so on) is provided by default and needs no separate feature. For usage, see [STYLE.md](/docs/STYLE.md).

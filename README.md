# wrcli

[English](README.md) | [한국어](README.ko.md)

[![CI](https://github.com/wkqco33/wrcli/actions/workflows/ci.yml/badge.svg)](https://github.com/wkqco33/wrcli/actions/workflows/ci.yml)

Releases are published to crates.io automatically when a matching version tag
is pushed (for example, `v0.4.0`). Configure the repository secret
`CARGO_REGISTRY_TOKEN` with a crates.io API token before creating a release tag.
[![Crates.io](https://img.shields.io/crates/v/wrcli.svg)](https://crates.io/crates/wrcli)
[![Documentation](https://docs.rs/wrcli/badge.svg)](https://docs.rs/wrcli)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A Rust CLI framework inspired by Go's [cobra](https://github.com/spf13/cobra) +
[viper](https://github.com/spf13/viper).
Nested subcommands, type-safe flags and multi-source configuration, composed
through a fluent builder API.

---

## Features

- Infinitely nested subcommands with aliases
- Type-safe flags (`bool`, `string`, `int`, `float`, `string[]`, `int[]`)
- Persistent flags — register once on the root and they propagate to every subcommand
- **Hidden** commands and flags — excluded from help/completion, still executable
- **Deprecated** commands and flags — warn on stderr when used
- **Flag constraint groups**: `mutually_exclusive`, `required_together`, `one_required`
- **Typo suggestions** (`Did you mean`) based on edit distance, plus `Command::suggest_for`
- **Dynamic completion** — `Command::complete`, `completion_request`, `arg_candidates`
- **CSV slice flags** — `Flag::comma_separated()` splits `--tag a,b,c`
- **Parent-local flags** — parsed even before the subcommand (`app --profile prod deploy`)
- **Usage hints** — `Command::usage_args("<name>")`
- **Exit-code classification** — `WrCliError::exit_code()`, `is_usage_error()`, `Command::execute_or_exit()`
- 5-layer configuration precedence: defaults → file (TOML/JSON/YAML/INI/dotenv/properties) → env vars → CLI flags → explicit `set`
- **Automatic config discovery** (`set_config_file`, format/path inference)
- **Key aliases** (`register_alias`), custom key delimiter, env key replacer, empty-env handling
- **Config ↔ flag binding** — unset flags are seeded from configuration values
- Typed getters: `get_string`/`get_int`/`get_uint`/`get_bool`/`get_float`/`get_string_vec`/`get_duration`/`get_time`/`get_size_in_bytes`
- **Enumeration & subtrees**: `all_keys`, `all_settings`, `get_string_map*`, `sub`
- **Runtime read/merge**: `read_config`, `merge_in_config`, `merge_config_map`
- **Config writing**: `write_config_as`, `safe_write_config_as`
- **Struct deserialization** (`serde` feature): `unmarshal`, `unmarshal_key`
- **Config file watching**: `on_config_change`, `watch_config`, `ConfigWatcher`
- **Formats**: TOML and JSON (default), YAML/INI/dotenv/Java properties (features)
- Lifecycle hooks: `persistent_pre_run` → `pre_run` → `run` → `post_run` → `persistent_post_run`
- `CommandContext` getters: `get_string`/`get_int`/`get_uint`/`get_bool`/`get_float`/`get_string_vec`/`get_int_vec`/`get_duration`/`get_time`/`get_size_in_bytes`/`get_string_map`/`is_set`
- **Built-in `help` subcommand** — `app help`, `app help sub [subsub]`
- **Help examples, support and docs links** — `example`, `support_url`, `docs_url` (with `{command}` substitution and inheritance)
- **Missing-runner policy** — a parent with subcommands prints help and exits 0; opt in with `help_on_missing_runner()`
- **Standard flags** — `standard_flags()`: `-q/--quiet`, `-f/--force`, `--no-input`, `--no-color`, `--plain`, `--json`, `--color`, `--confirm`
- **Output formats** — `OutputFormat` (Human/Plain/Json), `Table::render_plain()`
- **Interactive prompts** — `confirm`, `confirm_severe`, `prompt_password` (TTY and `--no-input` safe)
- **Sensitive flags** — `Flag::sensitive()` masks values
- **Optional values** — `Flag::optional_value()` (`none` means "no value")
- **`-` stdin/stdout** — `wrcli::io::{open_reader, open_writer, read_to_string}`
- **Color policy** — `FORCE_COLOR`/`NO_COLOR`/`TERM=dumb`/`*_NO_COLOR`/`--no-color`, `ColorChoice`
- **Pager** — `style::pager::page` (`PAGER`, defaults to `less -FIRX`)
- **SIGINT** — `signal` feature, `interrupt_message` (exit code 130)
- **Bug report URL** — `bug_report_url`
- **Completion script generation** (bash / zsh / fish)
- Rich terminal styling: `Style`, `Color`, `Table`, `Panel`, `Rule`, `Tree`, `Text`, `Progress`, `KeyVal`, `Badge`, `List`, `Spinner`
- `execute_with()` — inject arguments directly for unit tests without real argv

- Automatic `--help` / `--version`

---

## Installation

```toml
[dependencies]
wrcli = "0.4"

# If you also need YAML config files
wrcli = { version = "0.4", features = ["yaml-config"] }
```

---

## Quick start

```rust
use wrcli::{Command, Flag, FlagValue, Config};
use wrcli::args::minimum_n_args;

fn main() {
    let config = Config::new()
        .set_default("server.port", 8080i64)
        .automatic_env()
        .set_env_prefix("MYAPP");

    Command::new("myapp")
        .version("1.0.0")
        .short("My awesome CLI")
        .with_config(config)
        .persistent_flag(
            Flag::new("verbose", FlagValue::Bool(false), "enable verbose output").short('v'),
        )
        .subcommand(
            Command::new("greet")
                .short("Print a greeting")
                .args(minimum_n_args(1))
                .on_run(|ctx| {
                    for name in &ctx.args {
                        println!("Hello, {}!", name);
                    }
                }),
        )
        .execute()
        .unwrap();
}
```

```bash
$ myapp greet Alice Bob
Hello, Alice!
Hello, Bob!

$ myapp --help
Usage:
  myapp [command]
  myapp [flags]
...
```

Run the full examples:

```sh
cargo run --example basic -- --help
cargo run --example styled
```

---

## Documentation

See [docs/GUIDE.md](docs/GUIDE.md) for the full reference and
[docs/STYLE.md](docs/STYLE.md) for terminal styling.

| Topic | Link |
| ----- | ---- |
| Commands & subcommands | [docs/GUIDE.md#commands](docs/GUIDE.md#commands) |
| Flag types & parsing syntax | [docs/GUIDE.md#flags](docs/GUIDE.md#flags) |
| Help conventions (clig.dev) | [docs/GUIDE.md#help-conventions-cligdev](docs/GUIDE.md#help-conventions-cligdev) |
| Standard flags & output formats | [docs/GUIDE.md#standard-flags-and-output-formats](docs/GUIDE.md#standard-flags-and-output-formats) |
| Interactive input & confirmations | [docs/GUIDE.md#interactive-input-and-confirmation-prompts](docs/GUIDE.md#interactive-input-and-confirmation-prompts) |
| Sensitive flags | [docs/GUIDE.md#sensitive-flags](docs/GUIDE.md#sensitive-flags) |
| Optional-value flags | [docs/GUIDE.md#optional-value-flags](docs/GUIDE.md#optional-value-flags) |
| `-` stdin/stdout | [docs/GUIDE.md#standard-io-substitution](docs/GUIDE.md#standard-io-substitution) |
| Color policy & pager | [docs/GUIDE.md#color-policy-and-pager](docs/GUIDE.md#color-policy-and-pager) |
| Ctrl-C (SIGINT) handling | [docs/GUIDE.md#ctrl-c-sigint-handling](docs/GUIDE.md#ctrl-c-sigint-handling) |
| Hidden / deprecated / constraints | [docs/GUIDE.md#hidden-deprecated-and-flag-constraints](docs/GUIDE.md#hidden-deprecated-and-flag-constraints) |
| Configuration & precedence | [docs/GUIDE.md#configuration-config](docs/GUIDE.md#configuration-config) |
| Config ↔ flag binding | [docs/GUIDE.md#automatic-config-and-flag-binding](docs/GUIDE.md#automatic-config-and-flag-binding) |
| Completion generation | [docs/GUIDE.md#generating-completion-scripts](docs/GUIDE.md#generating-completion-scripts) |
| Dynamic completion | [docs/GUIDE.md#dynamic-completion](docs/GUIDE.md#dynamic-completion) |
| Lifecycle hooks | [docs/GUIDE.md#lifecycle-hooks](docs/GUIDE.md#lifecycle-hooks) |
| `CommandContext` | [docs/GUIDE.md#commandcontext](docs/GUIDE.md#commandcontext) |
| Error handling | [docs/GUIDE.md#error-handling](docs/GUIDE.md#error-handling) |
| Writing tests | [docs/GUIDE.md#writing-tests](docs/GUIDE.md#writing-tests) |
| Feature flags | [docs/GUIDE.md#feature-flags](docs/GUIDE.md#feature-flags) |
| Terminal styling | [docs/STYLE.md](docs/STYLE.md) |
| Changelog | [CHANGELOG.md](CHANGELOG.md) |

---

## License

[MIT](LICENSE)

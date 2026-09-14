# Changelog

[English](CHANGELOG.md) | [한국어](CHANGELOG.ko.md)

All notable changes to this project are documented here.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Added

- **New style components**:
  - `BoxStyle` enum (`Square`, `Rounded`, `Double`, `Heavy`, `Ascii`, `Markdown`) for custom border characters.
  - `KeyVal`: Aligned key-value pairs with customizable separator and styles.
  - `Badge`: Status tags/badges with convenient presets (`success`, `error`, `warn`, `info`) and bracket customisation.
  - `List`: Bullet (`•`, `-`, `→`) and `Numbered` list component with sublist nesting.
  - `Spinner`: Non-blocking terminal activity indicator respecting clig.dev rules.
  - `Text::from_markup`: Rich-inspired inline tag parser (e.g. `[bold green]...[/]`).
- **Enhanced component options**:
  - `Table`: Added `.box_style(BoxStyle)`, `.row_separator(bool)`, and `.border_style(Style)`.
  - `Panel`: Added `.box_style(BoxStyle)`, `.content_align(Align)`, `.subtitle(&str)`, `.subtitle_style(Style)`, and `.subtitle_align(Align)`.
  - `Rule`: Added `.align(Align)` for left/center/right title alignment.
  - `Tree`: Added `.guide_style(Style)` and multi-line label support.

### Fixed

- `display_width`: Now strips ANSI escape sequences and properly accounts for CJK and emoji character widths (2 columns) when computing visible terminal width, preventing misaligned borders in `Table`, `Panel`, and `Rule` when formatted or mixed-script text is used.
- `Table::border(false)`: Fixed issue where vertical line separators (`│`) were erroneously printed between columns and misaligned with plain separator dashes.
- `Tree`: Fixed branch and guide lines incorrectly inheriting the root node's label style.

### Changed

- Documentation is now English-first. `README.md`, `docs/GUIDE.md`, `docs/STYLE.md` and
  `CHANGELOG.md` are the canonical English versions, with Korean translations kept
  alongside as `README.ko.md`, `docs/GUIDE.ko.md`, `docs/STYLE.ko.md` and
  `CHANGELOG.ko.md`.
- All source comments (rustdoc and inline) and log messages are now in English.
  Korean string literals kept in tests are deliberate CJK display-width fixtures.


## [0.4.0] - 2026-09-12

Adds support for the clig.dev (Command Line Interface Guidelines).

### Added

- **Built-in `help` subcommand**: `app help`, `app help sub`, `app help sub subsub`.
  Disabled when the user registers `help` themselves. `help`, `--help` and `--version`
  are offered as completion candidates.
- **Help examples, support and docs links**: `Command::example`, `Command::support_url`,
  `Command::docs_url` (with `{command}` substitution and inheritance). The examples
  section is printed right after Usage.
- **Missing-runner policy**: a parent command that only has subcommands now prints help
  and exits with code 0 when run without arguments. `Command::help_on_missing_runner()`
  opts a leaf command into the same behavior.
- **`Command::bug_report_url`**: `execute_or_exit()` now points at the report URL for
  unexpected errors.
- **Standard flags** `Command::standard_flags()`: `-q/--quiet`, `-f/--force`,
  `--no-input`, `--no-color`, `--plain`, `--json`, `--color <when>`, `--confirm <name>`.
  Registered as persistent flags so they propagate to subcommands; `--plain` and `--json`
  are mutually exclusive.
- **Output formats**: `OutputFormat` (Human/Plain/Json) with
  `CommandContext::output_format`, `is_quiet`, `is_force`, `no_input`, `is_plain`,
  `is_json`, and `Table::render_plain()`.
- **Interactive input**: `CommandContext::confirm`, `confirm_severe`, `prompt_password`
  and `is_interactive`. Returns `InteractiveInputRequired` when stdin is not a TTY or
  `--no-input` was passed; a mismatched `confirm_severe` returns `ConfirmationFailed`.
- **Sensitive flags** `Flag::sensitive()`: masks the default in help and the value in
  error messages with `***`.
- **Optional-value flags** `Flag::optional_value()`: the special word `none` means
  "no value" (empty string).
- **`wrcli::io`**: `open_reader`, `open_writer`, `read_to_string` — treat `-` as
  stdin/stdout.
- **Color policy**: `ColorChoice`, `set_color_choice`/`color_choice`/`reset_color_choice`,
  `set_no_color_env`, `ColorEnv`/`should_use_color`, `stdout_is_terminal`,
  `stdin_is_terminal`. Supports `FORCE_COLOR`, `TERM=dumb`, app-specific `*_NO_COLOR`
  and `--no-color`/`--color=<when>`.
- **Pager** `style::pager::page`: pipes to `PAGER` (default `less -FIRX`) only when
  stdout is a TTY.
- **`Progress::draw()` / `Progress::finish()`**: without a TTY they print a single line
  instead of animating.
- **`signal` feature**: `Command::interrupt_message` and `wrcli::signal` — on Ctrl-C
  they print a message and exit immediately with code 130.

### Changed

- `style::print_warning`/`print_info` now write to **stderr** instead of stdout
  (clig.dev: messaging to stderr).
- `WrCliError::InvalidFlagValue` messages now include a `Run with --help for usage.` hint.
- `NO_COLOR` disables color only when **non-empty** (spec compliance); `TERM=dumb` added.
- A runner-less leaf command no longer prints help to stdout, only the error (parents with
  subcommands print help and succeed).

### Fixed

- `myapp help <unknown>` now returns `UnknownSubcommand` with `Did you mean` suggestions.

## [0.3.0] - 2026-09-12

### Added

- **Typo suggestions**: edit-distance based `Did you mean` candidates for unknown
  subcommands/flags in error messages (`src/suggest.rs`).
- **Hidden**: `Command::hidden()`, `Flag::hidden()` — excluded from help/completion/
  suggestions, still parsed and executed.
- **Deprecated**: `Command::deprecated(msg)`, `Flag::deprecated(msg)` — warn on stderr
  when used.
- **Flag constraint groups**: `Command::mutually_exclusive`, `required_together`,
  `one_required` with three dedicated error variants.
- **Exit codes**: `WrCliError::is_usage_error()`, `WrCliError::exit_code()`
  (usage errors 2, everything else 1), `Command::execute_or_exit()`.
- **Extended `CommandContext` getters**: `get_uint`, `get_int_vec`, `get_duration`,
  `get_time`, `get_size_in_bytes`, `get_string_map`, `is_set`.
- `FlagSet::get_uint`.
- **Dynamic completion**: `Command::complete`, `Command::completion_request`
  (`__complete` protocol), `Command::arg_candidates`.
- **CSV slice flags**: `Flag::comma_separated()` — splits `--tag a,b,c` into several
  values.
- **Parent-local flags**: the parent consumes its local flags placed before the
  subcommand name (`app --profile prod deploy`) and passes the values to the leaf
  context.
- **Usage hints**: `Command::usage_args("<name>")` — positional hints on the `--help`
  usage line.
- **`Command::suggest_for`**: suggestion-only aliases that are never executed
  (Cobra `SuggestFor`).
- `FlagSet::parse_partial` (consume parent flags without required validation),
  `FlagSet::inherit_values`.

### Changed

- Help output now splits inherited persistent flags into a `Global Flags:` section
  (Cobra style).
- Deprecated flags are marked `(deprecated)` in the help flag rows.
- Subcommand routing parses parent flags into the parent FlagSet and forwards the values
  to the child (`inherit_values`) so they are readable from `ctx.flags`. When
  `--help`/`--version` is present, the leaf still handles it as before.
- `FlagSet::is_set` returns `true` only for flags named on argv. Values seeded from
  configuration are now `false`, and Config-layer binding applies only to explicit input.
- Added a `suggestions` field to `WrCliError::UnknownFlag` / `UnknownSubcommand`.

### Fixed

- Removed unused import warnings in `config/writer.rs` for `no-default-features` builds.

## [0.2.0] - 2026-09-12

### Added — Config (Viper parity)

- 5-layer precedence: defaults → config file → env vars → CLI flags → explicit `set`.
- `set`, `is_set`, `register_alias`, `set_key_delimiter`, `set_env_key_replacer`,
  `allow_empty_env`.
- Typed getters: `get_int64`, `get_uint`, `get_duration`, `get_time`,
  `get_size_in_bytes`, `get_string_slice`.
- Enumeration and subtrees: `all_keys`, `all_settings`, `SettingsMap`/`SettingsEntry`,
  `get_string_map`/`get_string_map_string`/`get_string_map_string_slice`, `sub`.
- Runtime read/merge: `read_config`, `merge_in_config`, `merge_config_map`.
- Writing: `write_config_as`, `safe_write_config_as`.
- New formats (features): `ini-config`, `dotenv-config`, `properties-config`
  (read and write).
- Struct deserialization (`serde` feature): `unmarshal`, `unmarshal_key`.
- Config file watching: `on_config_change`, `set_watch_interval`, `watch_config`,
  `ConfigWatcher` (polling based, 1 second default).
- New errors: `ConfigTypeNotSet`, `ConfigFileExists`, `ConfigDeserializeError`,
  `ConfigWatchNotReady`.

### Fixed

- bash/zsh completion script generation bugs (stray quote, duplicate `#compdef`,
  missing `_arguments` continuation, missing fish short flags).
- `Panel` title border width no longer misaligns with the body when `padding != 1`.
- `--help` column alignment/wrapping broke with CJK and ANSI output
  (now uses `display_width`).

### Changed

- Consolidated the duplicated 4-layer lookup logic of six Config getters into `resolve()`.
- Reduced per-run allocations in config search-path computation, flag seeding and
  `Style::apply`.
- Docs: moved the guide into `docs/`, refreshed `README.md`/`docs/GUIDE.md`/`AGENTS.md`
  and added a changelog.

## [0.1.0]

- Initial release: command tree, typed flags, 4-layer configuration, completion,
  terminal styling.

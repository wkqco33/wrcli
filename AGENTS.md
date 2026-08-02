# AGENTS.md

Rust CLI framework library (cobra/viper-inspired). This file guides AI agents
(and humans) working on this repository. It is mandatory reading before writing code.

## Non-negotiable workflow: TDD (Test-Driven Development)

Every code change **must** follow the Red → Green → Refactor cycle. Do not write
implementation without a failing test first.

### Cycle

1. **RED** — Write a test that captures the desired behavior. Run it and confirm it **fails**
   (`cargo test`).
2. **GREEN** — Write the *minimum* implementation to make that test pass.
3. **REFACTOR** — Clean up: remove duplication, improve naming/structure, keep tests green.

### Where to put tests

| Level | Location | Purpose |
| ----- | -------- | ------- |
| Unit | `src/<mod>/...` `#[cfg(test)]` | Internal logic of a single module |
| Integration | `tests/*.rs` | Library-level behavior via public API (`Command`, `Config`, `Flag`) |
| Binary | `tests/binary.rs` | Real process behavior with `assert_cmd` + `predicates` |

Prefer **integration tests** under `tests/` for user-facing features. Group by domain:
`flags.rs`, `args.rs`, `config.rs`, `errors.rs`, `lifecycle.rs`, `subcommand.rs`.

### Test helpers (`tests/common/mod.rs`)

Reuse these instead of hand-rolling:

- `args("--name Alice")` — build `Vec<String>` from a whitespace-separated string.
- `EnvGuard::set("KEY", "val")` — set an env var, auto-restored on drop. **Always use for env tests**
  (parallel-safe via a global mutex).
- `tempdir()` / `TempDir` — auto-deleting temp dir for config-file tests.

### Capturing callback values in tests

Callbacks receive `&CommandContext`. To observe values, capture into `Arc<Mutex<_>>`:

```rust
let out = Arc::new(Mutex::new(String::new()));
let out2 = out.clone();
Command::new("app")
    .flag(Flag::new("name", FlagValue::String(String::new()), "name"))
    .on_run(move |ctx| *out2.lock().unwrap() = ctx.flags.get_string("name").unwrap().to_owned())
    .execute_with(args("--name Alice"))
    .unwrap();
assert_eq!(*out.lock().unwrap(), "Alice");
```

### Testing errors

Assert on the exact `WrCliError` variant with `matches!`:

```rust
let err = Command::new("app")
    .flag(Flag::new("name", FlagValue::String(String::new()), "name").required())
    .on_run(|_| {})
    .execute_with(args(""))
    .unwrap_err();
assert!(matches!(err, WrCliError::MissingRequiredFlag(n) if n == "name"));
```

For builder-time misuse (duplicate registration), use `#[should_panic(expected = "...")]`.

### Running tests

```sh
cargo test                      # default features
cargo test --all-features       # includes yaml-config
cargo test --test flags         # single integration file
cargo test -- --test-threads=1  # if you suspect env-var races
```

## Verification before finishing (MANDATORY)

After any change, all three must pass. Do not skip them.

```sh
cargo test --all-features
cargo clippy --all-features --all-targets -- -D warnings
cargo fmt -- --check
```

## Conventions

- **Builder pattern**: methods consume `self` and return `Self` (`Command::new("x").flag(...).on_run(...)`).
- **Errors**: use the `WrCliError` enum directly. New variants go at the end (the enum is
  `#[non_exhaustive]`). User errors wrap via `WrCliError::user(e)` or `on_run_e`.
- **Config priority** (low→high): defaults → config file → env vars → CLI flags (only explicitly set values).
- **Persistent flags**: register with `Command::persistent_flag()`; they propagate to subcommands.
- **Dotted keys**: config supports `"server.port"`; env vars map `.`/`-` to `_` and uppercase.
- **Feature-gated code**: config format backends (`toml-config`, `json-config`, `yaml-config`) must
  be `#[cfg(feature = ...)]`-guarded and tested with `--all-features`.
- **Do not add comments** to code unless asked. Docs (`///`) on public API are welcome.

## Commit style

Concise imperative messages matching the repo history, e.g.
`feat: add config auto-discovery` / `fix: correct help width for non-ASCII`.

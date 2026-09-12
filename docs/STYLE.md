# wrcli Styling Guide

[English](STYLE.md) | [한국어](STYLE.ko.md)

The `wrcli::style` module for decorating terminal output.
Inspired by the rich library, it provides colors, text attributes, tables, panels,
rules, trees, progress bars, and more.

## Table of Contents

- [Getting Started](#getting-started)
- [Color](#color)
- [Style](#style)
- [Text](#text)
- [Table](#table)
- [Panel](#panel)
- [Rule](#rule)
- [Tree](#tree)
- [Progress](#progress)
- [PAGER and Color Policy](#pager-and-color-policy)
- [Convenience Output Helpers](#convenience-output-helpers)

---

## Getting Started

```rust
use wrcli::style::{Color, Style, Table, Panel, Rule, Tree, Text, Progress, Align};
```

Styling is **automatically disabled** in the following cases:

- When `NO_COLOR` is set to a non-empty value
- When `TERM=dumb`
- When an app-specific `*_NO_COLOR` (for example, `MYAPP_NO_COLOR`) is set
- When the output stream is not a terminal (a pipe, etc.)

`FORCE_COLOR` (non-empty) overrides detection and turns colors on, and a global
override (`--no-color` / `--color=<when>`) takes the highest priority. See
“PAGER and Color Policy” for the detailed rules.

Every rendering type produces a string via `render(styled: bool)` and prints to
stdout via `print()`. `styled` is detected with `stdout_is_styled()`.

```rust
use wrcli::style::stdout_is_styled;

let table = Table::new().headers(["A", "B"]).row(["1", "2"]);
println!("{}", table.render(stdout_is_styled()));
```

---

## Color

Supports the 16 standard ANSI colors, 8-bit (256 colors), and 24-bit RGB truecolor.

| Name | Description |
| ---- | ---- |
| `Black` … `White` | Standard 8 colors |
| `BrightBlack` … `BrightWhite` | Bright 8 colors |
| `Fixed(u8)` | 8-bit (256-color) index |
| `Rgb(u8, u8, u8)` | 24-bit truecolor |

### Parsing from a String — `Color::from_name`

```rust
use wrcli::style::Color;

let c1 = Color::from_name("red");            // Some(Color::Red)
let c2 = Color::from_name("bright_cyan");    // Some(Color::BrightCyan)
let c3 = Color::from_name("bright cyan");    // a space is allowed instead of an underscore
let c4 = Color::from_name("42");             // Some(Color::Fixed(42))
let c5 = Color::from_name("#ff0000");        // Some(Color::Rgb(255, 0, 0))
let c6 = Color::from_name("rgb(0,128,255)"); // Some(Color::Rgb(0, 128, 255))
let c7 = Color::from_name("nope");           // None
```

---

## Style

A set of colors and text decorations (attributes). Build it with the builder, then
produce an ANSI escape string with `apply(text, styled)`.

### Attributes

| Method | ANSI | Description |
| ------ | :--: | ---- |
| `.fg(Color)` | 30–97 | Foreground color |
| `.bg(Color)` | 40–107 | Background color |
| `.bold()` | 1 | Bold |
| `.dim()` | 2 | Dim |
| `.italic()` | 3 | Italic |
| `.underline()` | 4 | Underline |
| `.blink()` | 5 | Blink |
| `.reverse()` | 7 | Reverse |
| `.hide()` | 8 | Hidden |
| `.strikethrough()` | 9 | Strikethrough |
| `.overline()` | 53 | Overline |

### Usage Example

```rust
use wrcli::style::{Style, Color};

let style = Style::new()
    .fg(Color::Green)
    .bg(Color::Black)
    .bold()
    .underline();

let out = style.apply("Success", true);   // "\x1b[1;4;32;40mSuccess\x1b[0m"
let plain = style.apply("Success", false); // "Success"  (unchanged)
```

If `styled = false` or the style is empty, the original text is returned unchanged.

---

## Text

Concatenates spans of different styles and renders them as a single text.

```rust
use wrcli::style::{Text, Style, Color};

let text = Text::new()
    .plain("Error: ")
    .span("boom", Style::new().fg(Color::Red).bold());

println!("{}", text.render(false)); // "Error: boom"
```

| Method | Description |
| ------ | ---- |
| `.plain("...")` | Add an unstyled span |
| `.span("...", style)` | Add a span with the given style |
| `.plain_styled("...", style)` | Alias for `span` |

---

## Table

A table that draws its border with Unicode box characters.

```rust
use wrcli::style::{Table, Align};

let out = Table::new()
    .headers(["Name", "Version", "Description"])
    .row(["wrcli", "0.1.0", "CLI framework"])
    .row(["serde", "1.0",  "serialization"])
    .align(vec![Align::Left, Align::Center, Align::Right])
    .render(false);
```

```text
┌───────┬─────────┬───────────────┐
│ Name  │ Version │   Description │
├───────┼─────────┼───────────────┤
│ wrcli │  0.1.0  │ CLI framework │
├───────┼─────────┼───────────────┤
│ serde │   1.0   │ serialization │
└───────┴─────────┴───────────────┘
```

| Method | Description |
| ------ | ---- |
| `.headers([...])` | Header row |
| `.row([...])` | Data row (call multiple times) |
| `.align(Vec<Align>)` | Per-column alignment (`Left`/`Center`/`Right`) |
| `.border(bool)` | Whether to draw the border (default `true`) |
| `.header_style(Style)` | Header style |

CJK characters (such as Hangul) are counted as 2 columns based on `display_width`,
so alignment is preserved.

---

## Panel

A box with a border and an optional title.

```rust
use wrcli::style::{Panel, Style, Color};

let out = Panel::new("Deploy complete.\nAll services healthy.")
    .title("Status")
    .border_style(Style::new().fg(Color::Green))
    .padding(1)
    .width(40)          // fixed width (fits the content when unset)
    .render(false);
```

```text
╭── Status ────────────────────────────────╮
│ Deploy complete.                         │
│ All services healthy.                    │
╰──────────────────────────────────────────╯
```

| Method | Description |
| ------ | ---- |
| `.title("...")` | Title (optional) |
| `.border_style(Style)` | Border style |
| `.title_style(Style)` | Title style |
| `.padding(usize)` | Left/right padding |
| `.width(usize)` | Fixed inner width |

---

## Rule

A horizontal rule with an optional centered title.

```rust
use wrcli::style::{Rule, Style, Color};

let out = Rule::new()
    .title("Configuration")
    .style(Style::new().fg(Color::Yellow))
    .width(60)
    .line_char('─')    // default
    .render(false);
```

| Method | Description |
| ------ | ---- |
| `.title("...")` | Centered title (optional) |
| `.style(Style)` | Line style |
| `.title_style(Style)` | Title style |
| `.width(usize)` | Line width (default 80) |
| `.line_char(char)` | Line character (default `─`) |

---

## Tree

Renders a hierarchical tree with Unicode box characters.

```rust
use wrcli::style::Tree;

let tree = Tree::new("root")
    .child(Tree::new("child1"))
    .child(
        Tree::new("child2")
            .child(Tree::new("grandchild1"))
            .child(Tree::new("grandchild2")),
    );

println!("{}", tree.render(false));
```

```text
root
├── child1
└── child2
    ├── grandchild1
    └── grandchild2
```

| Method | Description |
| ------ | ---- |
| `Tree::new("...")` | Create a node |
| `.child(Tree)` | Add a child node (call multiple times) |
| `.style(Style)` | Style of this node's label (default: cyan) |

---

## Progress

A terminal progress bar.

```rust
use wrcli::style::{Progress, Style, Color};

let bar = Progress::new(100)
    .progress(42)
    .width(20)
    .label("Downloading")
    .bar_style(Style::new().fg(Color::Green))
    .render(false);
```

```text
Downloading [########------------]  42%
```

| Method | Description |
| ------ | ---- |
| `Progress::new(total)` | Create with a total |
| `.progress(n)` | Current progress (0..total) |
| `.width(usize)` | Bar width (default 30) |
| `.label("...")` | Label prepended to the bar |
| `.bar_style(Style)` | Style of the filled portion (default: green) |
| `.filled_char(char)` | Filled character (default `#`) |
| `.empty_char(char)` | Empty character (default `-`) |

### Animation-Safe Output

`draw()` overwrites the current line only when stdout is a TTY, and `finish()`
prints the final state as a single line when stdout is not a TTY. No animation
artifacts are left behind in pipes or CI logs.

```rust
use wrcli::style::Progress;

let bar = Progress::new(100).progress(42).width(20);
for n in 0..=100 {
    bar.progress(n).draw();   // nothing is printed on a non-TTY
    std::thread::sleep(std::time::Duration::from_millis(10));
}
bar.finish();
```

---

## PAGER and Color Policy

Pass long output to `pager::page()` and it is sent to `PAGER`
(default `less -FIRX`) only when stdout is a TTY; in pipes and CI it is printed as-is.

```rust
wrcli::style::pager::page(&long_text)?;
```

Color priority (highest → lowest):

1. Global override — `--no-color` (Never) / `--color=always|never|auto`
2. `FORCE_COLOR` (non-empty)
3. `NO_COLOR` (non-empty)
4. `TERM=dumb`
5. App-specific `*_NO_COLOR`
6. Whether stdout is a TTY

```rust
use wrcli::style::{ColorChoice, set_color_choice, set_no_color_env};

set_no_color_env(Some("MYAPP_NO_COLOR"));  // register an app-specific variable
set_color_choice(ColorChoice::Always);     // global override
```

For TTY checks unrelated to color, use `stdout_is_terminal()` / `stdin_is_terminal()`.

---

## Convenience Output Helpers

`wrcli::style` provides helpers for quickly printing status messages.

```rust
use wrcli::style::{print_success, print_error, print_warning, print_info};

print_success("Build complete");   // ✓ (green)
print_error("Task failed");        // ✗ (red, stderr)
print_warning("Missing config");   // ⚠ (yellow)
print_info("Processing...");       // ℹ (cyan)
```

It also provides the `stdout_is_styled()`, `stderr_is_styled()`, and
`display_width(s)` (CJK 2-column calculation) utilities.

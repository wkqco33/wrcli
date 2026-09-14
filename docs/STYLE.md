# wrcli Styling Guide

[English](STYLE.md) | [한국어](STYLE.ko.md)

The `wrcli::style` module for decorating terminal output.
Inspired by the rich library, it provides colors, text attributes, tables, panels,
rules, trees, progress bars, and more.

## Table of Contents

- [Getting Started](#getting-started)
- [Color](#color)
- [Style](#style)
- [Text and Markup](#text-and-markup)
- [Table and BoxStyle](#table-and-boxstyle)
- [Panel](#panel)
- [Rule](#rule)
- [Tree](#tree)
- [KeyVal](#keyval)
- [Badge](#badge)
- [List](#list)
- [Spinner and Progress](#spinner-and-progress)
- [PAGER and Color Policy](#pager-and-color-policy)
- [Convenience Output Helpers](#convenience-output-helpers)

---

## Getting Started

```rust
use wrcli::style::{
    Align, Badge, BoxStyle, Color, KeyVal, List, ListMarker, Panel, Progress, Rule, Spinner,
    Style, Table, Text, Tree,
};
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

## Text and Markup

Concatenates spans of different styles and renders them as a single text. It also supports rich-style inline markup.

```rust
use wrcli::style::{Text, Style, Color};

let text = Text::new()
    .plain("Error: ")
    .span("boom", Style::new().fg(Color::Red).bold());

println!("{}", text.render(false)); // "Error: boom"

// Inline markup
let markup_text = Text::from_markup("[bold green]Success:[/] file [cyan]test.rs[/] created");
markup_text.print();
```

| Method | Description |
| ------ | ---- |
| `.plain("...")` | Add an unstyled span |
| `.span("...", style)` | Add a span with the given style |
| `.plain_styled("...", style)` | Alias for `span` |
| `Text::from_markup("...")` | Parses inline tags like `[bold green]...[/]` or `[on_red white]...[/]` |

---

## Table and BoxStyle

A table that draws its border with configurable box-drawing characters (`BoxStyle`).

```rust
use wrcli::style::{Table, Align, BoxStyle};

let out = Table::new()
    .box_style(BoxStyle::Rounded)
    .headers(["Name", "Version", "Description"])
    .row(["wrcli", "0.4.0", "CLI framework"])
    .row(["serde", "1.0",  "serialization"])
    .align(vec![Align::Left, Align::Center, Align::Right])
    .row_separator(false)
    .render(false);
```

```text
╭───────┬─────────┬───────────────╮
│ Name  │ Version │   Description │
├───────┼─────────┼───────────────┤
│ wrcli │  0.4.0  │ CLI framework │
│ serde │   1.0   │ serialization │
╰───────┴─────────┴───────────────╯
```

| Method | Description |
| ------ | ---- |
| `.headers([...])` | Header row |
| `.row([...])` | Data row (call multiple times) |
| `.align(Vec<Align>)` | Per-column alignment (`Left`/`Center`/`Right`) |
| `.border(bool)` | Whether to draw borders (default `true`) |
| `.box_style(BoxStyle)` | Border character style (`Square`, `Rounded`, `Double`, `Heavy`, `Ascii`, `Markdown`) |
| `.row_separator(bool)` | Whether to draw separators between data rows (default `true`) |
| `.border_style(Style)` | Style for border lines |
| `.header_style(Style)` | Header row text style |

CJK characters and ANSI escape sequences are properly measured with `display_width` so alignment remains exact.

---

## Panel

A box with a border, an optional title, and an optional subtitle (footer).

```rust
use wrcli::style::{Panel, Style, Color, BoxStyle, Align};

let out = Panel::new("Deploy complete.\nAll services healthy.")
    .title("Status")
    .subtitle("region: us-east-1")
    .box_style(BoxStyle::Rounded)
    .content_align(Align::Left)
    .border_style(Style::new().fg(Color::Green))
    .padding(1)
    .width(40)          // fixed width (fits the content when unset)
    .render(false);
```

```text
╭── Status ────────────────────────────────╮
│ Deploy complete.                         │
│ All services healthy.                    │
╰───────────────────── region: us-east-1 ──╯
```

| Method | Description |
| ------ | ---- |
| `.title("...")` | Top title (optional) |
| `.subtitle("...")` | Bottom footer / subtitle (optional) |
| `.title_style(Style)` | Title style |
| `.subtitle_style(Style)` | Subtitle style |
| `.subtitle_align(Align)` | Subtitle alignment (`Left`/`Center`/`Right`, default `Right`) |
| `.box_style(BoxStyle)` | Border style (`Square`, `Rounded`, `Double`, etc.) |
| `.content_align(Align)` | Content alignment (`Left`/`Center`/`Right`) |
| `.border_style(Style)` | Border line style |
| `.padding(usize)` | Left/right padding |
| `.width(usize)` | Fixed inner width |

---

## Rule

A horizontal rule with an optional title and configurable alignment.

```rust
use wrcli::style::{Rule, Style, Color, Align};

let out = Rule::new()
    .title("Configuration")
    .align(Align::Left)
    .style(Style::new().fg(Color::Yellow))
    .width(60)
    .line_char('─')    // default
    .render(false);
```

| Method | Description |
| ------ | ---- |
| `.title("...")` | Title text (optional) |
| `.align(Align)` | Title alignment (`Left`/`Center`/`Right`, default `Center`) |
| `.style(Style)` | Line style |
| `.title_style(Style)` | Title style |
| `.width(usize)` | Line width (default 80) |
| `.line_char(char)` | Line character (default `─`) |

---

## Tree

Renders a hierarchical tree with Unicode box characters, customizable guide styles, and multi-line support.

```rust
use wrcli::style::{Tree, Style, Color};

let tree = Tree::new("root")
    .guide_style(Style::new().fg(Color::BrightBlack))
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
| `.guide_style(Style)` | Style for guide lines and branches |

---

## KeyVal

Aligns key-value pairs cleanly according to the widest key.

```rust
use wrcli::style::KeyVal;

let out = KeyVal::new()
    .entry("Host", "127.0.0.1")
    .entry("Port", "8080")
    .separator(" : ")
    .render(false);
```

```text
Host : 127.0.0.1
Port : 8080
```

| Method | Description |
| ------ | ---- |
| `KeyVal::new()` | Create a new KeyVal viewer |
| `.entry(key, val)` | Add a key-value entry |
| `.separator(" : ")` | Separator between key and value |
| `.key_style(Style)` | Style for keys |
| `.val_style(Style)` | Style for values |
| `.sep_style(Style)` | Style for the separator |

---

## Badge

Compact status badges and tags with presets for common states.

```rust
use wrcli::style::Badge;

let b1 = Badge::success("PASS");
let b2 = Badge::error("FAIL");
let b3 = Badge::warn("WARN");
let b4 = Badge::info("INFO");
let pill = Badge::new("ACTIVE").no_brackets();
```

| Method | Description |
| ------ | ---- |
| `Badge::new("...")` | Custom badge |
| `Badge::success("...")` | Green success badge (`[PASS]`) |
| `Badge::error("...")` | Red error badge (`[FAIL]`) |
| `Badge::warn("...")` | Yellow warning badge (`[WARN]`) |
| `Badge::info("...")` | Cyan info badge (`[INFO]`) |
| `.brackets('(', ')')` | Custom brackets (default `[`, `]`) |
| `.no_brackets()` | Space-padded pill badge without brackets |
| `.style(Style)` | Text style |
| `.bracket_style(Style)` | Bracket style |

---

## List

Bulleted and numbered lists with support for nesting.

```rust
use wrcli::style::{List, ListMarker};

let list = List::new()
    .marker(ListMarker::Bullet)
    .item("Compile assets")
    .item("Link binaries")
    .sublist(
        List::new()
            .marker(ListMarker::Arrow)
            .item("target/release/app"),
    );

list.print();
```

```text
• Compile assets
• Link binaries
  → target/release/app
```

| Method | Description |
| ------ | ---- |
| `List::new()` | Create a new list |
| `.item("...")` | Add an item |
| `.sublist(List)` | Add a nested sublist |
| `.marker(ListMarker)` | `Bullet` (`•`), `Dash` (`-`), `Arrow` (`→`), `Numbered` (`1.`) |
| `.marker_style(Style)` | Style for bullet/number markers |
| `.item_style(Style)` | Style for item text |

---

## Spinner and Progress

`Spinner` provides a non-blocking indicator for indeterminate tasks, while `Progress` tracks determinate completion percentages. Both strictly adhere to clig.dev rules (animated only on interactive TTYs, static on non-TTY pipes/CI).

```rust
use wrcli::style::Spinner;

let mut sp = Spinner::new("Downloading assets...");
sp.tick(); // advances frame and draws via `\r` on TTY
sp.finish_with_message("Done!");
```


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

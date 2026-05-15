//! `examples/styled.rs` — Demonstration of wrcli's rich terminal styling API.
//!
//! Run:
//!   cargo run --example styled

use wrcli::style::{
    Align, Color, Panel, Rule, Style, Table,
    print_error, print_info, print_success, print_warning,
};

fn main() {
    env_logger::init();
    // ── Section separator ──────────────────────────────────────────────────────
    Rule::new()
        .title("wrcli Rich Styling Demo")
        .style(Style::new().fg(Color::Cyan).bold())
        .title_style(Style::new().bold().fg(Color::BrightWhite))
        .width(60)
        .print();

    println!();

    // ── Status messages ────────────────────────────────────────────────────────
    Rule::new()
        .title("Status messages")
        .style(Style::new().fg(Color::BrightBlack))
        .width(60)
        .print();

    print_success("Build passed — all 114 tests green");
    print_error("Connection refused: 127.0.0.1:5432");
    print_warning("Flag `--timeout` is deprecated; use `--max-wait`");
    print_info("Listening on http://0.0.0.0:8080");

    println!();

    // ── Inline styled text ─────────────────────────────────────────────────────
    Rule::new()
        .title("Inline styled text")
        .style(Style::new().fg(Color::BrightBlack))
        .width(60)
        .print();

    let bold      = Style::new().bold();
    let italic    = Style::new().italic();
    let underline = Style::new().underline();
    let strike    = Style::new().strikethrough();
    let dim       = Style::new().dim();
    let highlight = Style::new().fg(Color::Black).bg(Color::BrightYellow);

    // Detect TTY once so we can pass the flag consistently.
    let tty = wrcli::style::stdout_is_styled();

    println!(
        "  {} {} {} {} {} {}",
        bold.apply("bold", tty),
        italic.apply("italic", tty),
        underline.apply("underline", tty),
        strike.apply("strikethrough", tty),
        dim.apply("dim", tty),
        highlight.apply(" highlight ", tty),
    );

    println!();

    // ── 16-color palette ───────────────────────────────────────────────────────
    Rule::new()
        .title("16-color palette")
        .style(Style::new().fg(Color::BrightBlack))
        .width(60)
        .print();

    let colors = [
        Color::Black,   Color::Red,     Color::Green,   Color::Yellow,
        Color::Blue,    Color::Magenta, Color::Cyan,    Color::White,
        Color::BrightBlack, Color::BrightRed, Color::BrightGreen, Color::BrightYellow,
        Color::BrightBlue,  Color::BrightMagenta, Color::BrightCyan, Color::BrightWhite,
    ];
    let names = [
        "Black", "Red", "Green", "Yellow",
        "Blue", "Magenta", "Cyan", "White",
        "BrightBlack", "BrightRed", "BrightGreen", "BrightYellow",
        "BrightBlue", "BrightMagenta", "BrightCyan", "BrightWhite",
    ];
    let mut row = String::new();
    for (i, (c, name)) in colors.iter().zip(names.iter()).enumerate() {
        row.push_str(&Style::new().fg(*c).apply(&format!("{:14}", name), tty));
        if (i + 1) % 4 == 0 {
            println!("  {}", row.trim_end());
            row.clear();
        }
    }

    println!();

    // ── Table ──────────────────────────────────────────────────────────────────
    Rule::new()
        .title("Table")
        .style(Style::new().fg(Color::BrightBlack))
        .width(60)
        .print();

    Table::new()
        .header_style(Style::new().bold().fg(Color::BrightCyan))
        .headers(["Crate", "Version", "Description"])
        .row(["wrcli",      "0.1.0", "Cobra/Viper-inspired CLI framework"])
        .row(["serde",      "1.0",   "De/serialization framework"])
        .row(["tokio",      "1.38",  "Async runtime"])
        .row(["clap",       "4.5",   "Command Line Argument Parser"])
        .align(vec![Align::Left, Align::Center, Align::Left])
        .print();

    println!();

    // ── Panel ──────────────────────────────────────────────────────────────────
    Rule::new()
        .title("Panels")
        .style(Style::new().fg(Color::BrightBlack))
        .width(60)
        .print();

    Panel::new("Deploy complete.\nAll services are healthy.\nRollback window: 30 minutes.")
        .title("Deployment")
        .border_style(Style::new().fg(Color::Green))
        .title_style(Style::new().bold().fg(Color::BrightGreen))
        .print();

    println!();

    Panel::new("Connection to database timed out after 30s.\nCheck that postgres is running on port 5432.")
        .title("Error")
        .border_style(Style::new().fg(Color::Red))
        .title_style(Style::new().bold().fg(Color::BrightRed))
        .print();

    println!();

    Panel::new("This panel has no title and uses the default cyan border style.\nIt wraps multiple lines of text neatly inside a Unicode box.")
        .print();

    println!();

    // ── Closing rule ───────────────────────────────────────────────────────────
    Rule::new()
        .style(Style::new().fg(Color::Cyan))
        .width(60)
        .print();
}

use crate::command::Command;
use crate::flag::{Flag, FlagSet, FlagValue};
use crate::style::{Color, Style, stdout_is_styled};

/// Collects the set of styles used when rendering help text.
struct HelpStyles {
    styled: bool,
    section: Style,
    cmd_name: Style,
    flag_name: Style,
    meta: Style,
    required: Style,
}

impl HelpStyles {
    fn new() -> Self {
        let styled = stdout_is_styled();
        HelpStyles {
            styled,
            section:   Style::new().bold().fg(Color::Yellow),
            cmd_name:  Style::new().bold().fg(Color::Green),
            flag_name: Style::new().fg(Color::Cyan),
            meta:      Style::new().dim(),
            required:  Style::new().fg(Color::Red),
        }
    }
}

pub fn print_help(
    name: &str,
    short: &str,
    long: &str,
    version: &Option<String>,
    flags: &FlagSet,
    subcommands: &[Command],
    command_path: &[String],
) {
    let path = command_path.join(" ");
    let s = HelpStyles::new();

    // ── Usage ─────────────────────────────────────────────────────────────────
    println!("{}", s.section.apply("Usage:", s.styled));
    if !subcommands.is_empty() {
        println!("  {} [command]", path);
    }
    println!("  {} [flags]", path);

    // ── Description ───────────────────────────────────────────────────────────
    let desc = if !long.is_empty() { long } else { short };
    if !desc.is_empty() {
        println!();
        for line in word_wrap(desc, 80) {
            println!("{}", line);
        }
    }

    // ── Version ───────────────────────────────────────────────────────────────
    if let Some(v) = version {
        println!();
        println!("Version: {}", s.meta.apply(v, s.styled));
    }

    // ── Subcommands ───────────────────────────────────────────────────────────
    if !subcommands.is_empty() {
        println!();
        println!("{}", s.section.apply("Available Commands:", s.styled));
        let col = subcommands.iter().map(|c| c.name.len()).max().unwrap_or(0);
        for cmd in subcommands {
            let aliases = if cmd.aliases.is_empty() {
                String::new()
            } else {
                format!(" ({})", cmd.aliases.join(", "))
            };
            println!(
                "  {:<width$}   {}{}",
                s.cmd_name.apply(&cmd.name, s.styled),
                cmd.short,
                s.meta.apply(&aliases, s.styled),
                width = col
            );
        }
    }

    // ── Flags ─────────────────────────────────────────────────────────────────
    let implicit_has_version = version.is_some();
    let has_any_short = flags.flags_iter().any(|f| f.short.is_some()) || implicit_has_version;

    let flag_col_width = {
        let user_max = flags.flags_iter()
            .map(|f| flag_lhs_len(f, has_any_short))
            .max()
            .unwrap_or(0);
        let help_lhs = short_prefix_len(true, has_any_short) + "--help".len();
        let ver_lhs  = short_prefix_len(implicit_has_version, has_any_short) + "--version".len();
        user_max.max(help_lhs).max(ver_lhs)
    };

    let has_user_flags = flags.flags_iter().next().is_some();
    if has_user_flags {
        println!();
        println!("{}", s.section.apply("Flags:", s.styled));
        for flag in flags.flags_iter() {
            print_flag_row(flag, has_any_short, flag_col_width, &s);
        }
    }

    // Implicit flags (-h / -V)
    println!();
    if !has_user_flags {
        println!("{}", s.section.apply("Flags:", s.styled));
    }
    print_implicit_flag('h', "help",    "", "Show this help message", has_any_short, flag_col_width, &s);
    if implicit_has_version {
        print_implicit_flag('V', "version", "", "Show version", has_any_short, flag_col_width, &s);
    }

    // ── Footer ────────────────────────────────────────────────────────────────
    if !name.is_empty() && !subcommands.is_empty() {
        println!();
        println!(
            "Use \"{} [command] --help\" for more information about a command.",
            path
        );
    }
}

// ── Rendering helpers ─────────────────────────────────────────────────────────

fn flag_lhs_len(flag: &Flag, has_any_short: bool) -> usize {
    short_prefix_len(flag.short.is_some(), has_any_short)
        + "--".len()
        + flag.name.len()
        + type_hint_len(&flag.default)
}

fn short_prefix_len(has_short: bool, has_any_short: bool) -> usize {
    if !has_any_short {
        0
    } else if has_short {
        4 // "-x, "
    } else {
        4 // "    " (padding to align)
    }
}

fn type_hint_len(val: &FlagValue) -> usize {
    match val {
        FlagValue::Bool(_) => 0,
        other => 2 + other.type_name().len() + 1, // " <typename>"
    }
}

fn print_flag_row(flag: &Flag, has_any_short: bool, col_width: usize, s: &HelpStyles) {
    let mut lhs = String::with_capacity(col_width + 4);
    if has_any_short {
        match flag.short {
            Some(c) => { lhs.push('-'); lhs.push(c); lhs.push_str(", "); }
            None    => lhs.push_str("    "),
        }
    }
    lhs.push_str("--");
    lhs.push_str(&flag.name);
    if !matches!(&flag.default, FlagValue::Bool(_)) {
        lhs.push_str(" <");
        lhs.push_str(flag.default.type_name());
        lhs.push('>');
    }

    let rhs = build_flag_rhs(flag, s);
    println!(
        "  {:<width$}   {}",
        s.flag_name.apply(&lhs, s.styled),
        rhs,
        width = col_width,
    );
}

fn print_implicit_flag(
    short: char,
    name: &str,
    type_hint: &str,
    usage: &str,
    has_any_short: bool,
    col_width: usize,
    s: &HelpStyles,
) {
    let mut lhs = String::with_capacity(col_width + 4);
    if has_any_short {
        lhs.push('-'); lhs.push(short); lhs.push_str(", ");
    }
    lhs.push_str("--");
    lhs.push_str(name);
    lhs.push_str(type_hint);
    println!(
        "  {:<width$}   {}",
        s.flag_name.apply(&lhs, s.styled),
        usage,
        width = col_width,
    );
}

fn build_flag_rhs(flag: &Flag, s: &HelpStyles) -> String {
    let mut buf = String::with_capacity(flag.usage.len() + 32);
    buf.push_str(&flag.usage);

    let has_default = match &flag.default {
        FlagValue::Bool(false) | FlagValue::Int(0) => false,
        FlagValue::Float(f) if *f == 0.0 => false,
        FlagValue::String(s) if s.is_empty() => false,
        FlagValue::StringVec(v) if v.is_empty() => false,
        FlagValue::IntVec(v) if v.is_empty() => false,
        _ => true,
    };
    if has_default {
        buf.push_str(" (default: ");
        match &flag.default {
            FlagValue::Bool(true)   => buf.push_str("true"),
            FlagValue::String(v)    => { buf.push('"'); buf.push_str(v); buf.push('"'); }
            FlagValue::Int(i)       => buf.push_str(&i.to_string()),
            FlagValue::Float(f)     => buf.push_str(&f.to_string()),
            _ => {}
        }
        buf.push(')');
    }
    if flag.required {
        buf.push_str(&s.required.apply(" [required]", s.styled));
    }
    buf
}

fn word_wrap(text: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    for paragraph in text.split('\n') {
        if paragraph.is_empty() {
            lines.push(String::new());
            continue;
        }
        let mut current = String::new();
        for word in paragraph.split_whitespace() {
            if current.is_empty() {
                current.push_str(word);
            } else if current.len() + 1 + word.len() <= max_width {
                current.push(' ');
                current.push_str(word);
            } else {
                lines.push(current);
                current = word.to_owned();
            }
        }
        if !current.is_empty() {
            lines.push(current);
        }
    }
    lines
}

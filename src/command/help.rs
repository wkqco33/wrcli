use crate::command::Command;
use crate::flag::{Flag, FlagSet, FlagValue};
use crate::style::{Color, Style, display_width, stdout_is_styled};

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
            section: Style::new().bold().fg(Color::Yellow),
            cmd_name: Style::new().bold().fg(Color::Green),
            flag_name: Style::new().fg(Color::Cyan),
            meta: Style::new().dim(),
            required: Style::new().fg(Color::Red),
        }
    }
}

/// 커맨드 하나의 도움말을 stdout에 출력한다.
///
/// `flags`는 상속된 persistent 플래그가 병합된 FlagSet일 수 있고,
/// `support_url`/`docs_url`은 상위 커맨드에서 상속된 유효 값이다.
pub fn print_help(
    cmd: &Command,
    flags: &FlagSet,
    command_path: &[String],
    support_url: Option<&str>,
    docs_url: Option<&str>,
) {
    let name = &cmd.name;
    let short = &cmd.short;
    let long = &cmd.long;
    let version = &cmd.version;
    let usage_args = cmd.usage_args.as_deref();
    let subcommands = &cmd.subcommands;
    let path = command_path.join(" ");
    let s = HelpStyles::new();
    let visible_subcommands: Vec<&Command> = subcommands.iter().filter(|c| !c.hidden).collect();

    // ── Usage ─────────────────────────────────────────────────────────────────
    println!("{}", s.section.apply("Usage:", s.styled));
    if !visible_subcommands.is_empty() {
        println!("  {} [command]", path);
    }
    match usage_args {
        Some(hint) if !hint.is_empty() => println!("  {} {} [flags]", path, hint),
        _ => println!("  {} [flags]", path),
    }

    // ── Examples ───────────────────────────────────────────────────────────
    // clig.dev: 사용자는 다른 문서보다 예제를 먼저 본다.
    if !cmd.examples.is_empty() {
        println!();
        println!("{}", s.section.apply("Examples:", s.styled));
        for example in &cmd.examples {
            println!("  {}", example);
        }
    }

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
    if !visible_subcommands.is_empty() {
        println!();
        println!("{}", s.section.apply("Available Commands:", s.styled));
        let col = subcommand_col_width(&visible_subcommands);
        for cmd in &visible_subcommands {
            let aliases = if cmd.aliases.is_empty() {
                String::new()
            } else {
                format!(" ({})", cmd.aliases.join(", "))
            };
            println!(
                "  {}   {}{}",
                s.cmd_name.apply(&pad_display(&cmd.name, col), s.styled),
                cmd.short,
                s.meta.apply(&aliases, s.styled),
            );
        }
    }

    // ── Flags ─────────────────────────────────────────────────────────────────
    let implicit_has_version = version.is_some();
    let local: Vec<&Flag> = flags
        .flags_iter()
        .filter(|f| !f.hidden && !f.inherited)
        .collect();
    let global: Vec<&Flag> = flags
        .flags_iter()
        .filter(|f| !f.hidden && f.inherited)
        .collect();

    let has_any_short = local.iter().any(|f| f.short.is_some()) || implicit_has_version;
    let flag_col_width = {
        let user_max = local
            .iter()
            .map(|f| flag_lhs_len(f, has_any_short))
            .max()
            .unwrap_or(0);
        let help_lhs = short_prefix_len(has_any_short) + "--help".len();
        let ver_lhs = short_prefix_len(has_any_short) + "--version".len();
        user_max.max(help_lhs).max(ver_lhs)
    };

    // Local flags section always includes the implicit `-h` / `-V` rows.
    println!();
    println!("{}", s.section.apply("Flags:", s.styled));
    for flag in &local {
        print_flag_row(flag, has_any_short, flag_col_width, &s);
    }
    print_implicit_flag(
        'h',
        "help",
        "",
        "Show this help message",
        has_any_short,
        flag_col_width,
        &s,
    );
    if implicit_has_version {
        print_implicit_flag(
            'V',
            "version",
            "",
            "Show version",
            has_any_short,
            flag_col_width,
            &s,
        );
    }

    // Inherited persistent flags get their own section (Cobra-style).
    if !global.is_empty() {
        let global_has_short = global.iter().any(|f| f.short.is_some());
        let global_width = global
            .iter()
            .map(|f| flag_lhs_len(f, global_has_short))
            .max()
            .unwrap_or(0);
        println!();
        println!("{}", s.section.apply("Global Flags:", s.styled));
        for flag in &global {
            print_flag_row(flag, global_has_short, global_width, &s);
        }
    }

    // ── Footer ────────────────────────────────────────────────────────────────
    if !name.is_empty() && !visible_subcommands.is_empty() {
        println!();
        println!(
            "Use \"{} [command] --help\" for more information about a command.",
            path
        );
    }

    // ── Support / Documentation ──────────────────────────────────────────
    // clig.dev: 피드백 경로와 웹 문서 링크를 도움말에 포함한다.
    if support_url.is_some() || docs_url.is_some() {
        println!();
        if let Some(url) = support_url {
            println!("{}", s.section.apply("Support:", s.styled));
            println!("  {}", url);
        }
        if let Some(url) = docs_url {
            println!("{}", s.section.apply("Documentation:", s.styled));
            println!("  {}", url.replace("{command}", &path));
        }
    }
}

// ── Rendering helpers ─────────────────────────────────────────────────────────

fn subcommand_col_width(subcommands: &[&Command]) -> usize {
    subcommands
        .iter()
        .map(|c| display_width(&c.name))
        .max()
        .unwrap_or(0)
}

/// Right-pad `s` with spaces to `width` terminal columns.
fn pad_display(s: &str, width: usize) -> String {
    let mut out = String::with_capacity(s.len() + width);
    out.push_str(s);
    out.push_str(&" ".repeat(width.saturating_sub(display_width(s))));
    out
}

fn flag_lhs_len(flag: &Flag, has_any_short: bool) -> usize {
    short_prefix_len(has_any_short)
        + "--".len()
        + display_width(&flag.name)
        + type_hint_len(&flag.default)
}

fn short_prefix_len(has_any_short: bool) -> usize {
    if has_any_short { 4 } else { 0 }
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
            Some(c) => {
                lhs.push('-');
                lhs.push(c);
                lhs.push_str(", ");
            }
            None => lhs.push_str("    "),
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
        "  {}   {}",
        s.flag_name.apply(&pad_display(&lhs, col_width), s.styled),
        rhs,
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
        lhs.push('-');
        lhs.push(short);
        lhs.push_str(", ");
    }
    lhs.push_str("--");
    lhs.push_str(name);
    lhs.push_str(type_hint);
    println!(
        "  {}   {}",
        s.flag_name.apply(&pad_display(&lhs, col_width), s.styled),
        usage,
    );
}

fn build_flag_rhs(flag: &Flag, s: &HelpStyles) -> String {
    let mut buf = String::with_capacity(flag.usage.len() + 32);
    buf.push_str(&flag.usage);

    let has_default = !flag.sensitive
        && match &flag.default {
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
            FlagValue::Bool(true) => buf.push_str("true"),
            FlagValue::String(v) => {
                buf.push('"');
                buf.push_str(v);
                buf.push('"');
            }
            FlagValue::Int(i) => buf.push_str(&i.to_string()),
            FlagValue::Float(f) => buf.push_str(&f.to_string()),
            _ => {}
        }
        buf.push(')');
    }
    if flag.required {
        buf.push_str(&s.required.apply(" [required]", s.styled));
    }
    if flag.deprecated.is_some() {
        buf.push_str(&s.required.apply(" (deprecated)", s.styled));
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
        let mut current_width = 0usize;
        for word in paragraph.split_whitespace() {
            let word_width = display_width(word);
            if current.is_empty() {
                current.push_str(word);
                current_width = word_width;
            } else if current_width + 1 + word_width <= max_width {
                current.push(' ');
                current.push_str(word);
                current_width += 1 + word_width;
            } else {
                lines.push(std::mem::take(&mut current));
                current.push_str(word);
                current_width = word_width;
            }
        }
        if !current.is_empty() {
            lines.push(current);
        }
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn word_wrap_uses_display_width_for_cjk() {
        assert_eq!(word_wrap("가나 다라 마바", 10), vec!["가나 다라", "마바"]);
    }

    #[test]
    fn subcommand_column_width_uses_display_width() {
        let cmds = [Command::new("가나"), Command::new("abcd")];
        let refs: Vec<&Command> = cmds.iter().collect();
        assert_eq!(subcommand_col_width(&refs), 4);
    }

    #[test]
    fn pad_display_pads_by_display_width() {
        assert_eq!(pad_display("가나", 6), "가나  ");
        assert_eq!(pad_display("abcd", 4), "abcd");
    }
}

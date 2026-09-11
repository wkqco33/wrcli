//! Completion script generation for bash/zsh/fish.

use super::Command;
use crate::error::Result;
use crate::flag::Flag;

/// Recursively collect subcommand names reachable from `cmd`.
fn collect_commands(cmd: &Command, out: &mut Vec<String>) {
    for sub in &cmd.subcommands {
        out.push(sub.name.clone());
        collect_commands(sub, out);
    }
}

/// Collect flags for a command and all its subcommands, deduplicated by name.
fn collect_flags<'a>(cmd: &'a Command, out: &mut Vec<&'a Flag>) {
    for flag in cmd.flags.flags_iter() {
        if !out.iter().any(|f| f.name == flag.name) {
            out.push(flag);
        }
    }
    for sub in &cmd.subcommands {
        collect_flags(sub, out);
    }
}

/// Escape a description for single-quoted shell literals.
fn quote(s: &str) -> String {
    s.replace('\'', "\\'")
}

fn shell_bash(cmd: &Command, out: &mut String) {
    use std::fmt::Write as _;

    let _ = write!(
        out,
        "#!/usr/bin/env bash\n# bash completion for {name}\n\n_{name}() {{\n",
        name = cmd.name
    );
    out.push_str("  local cur prev words cword\n");
    out.push_str("  COMPREPLY=()\n");
    out.push_str("  cur=\"${COMP_WORDS[COMP_CWORD]}\"\n\n");

    let mut commands = Vec::new();
    collect_commands(cmd, &mut commands);
    if !commands.is_empty() {
        out.push_str("  if [[ ${COMP_CWORD} -eq 1 ]]; then\n");
        out.push_str("    COMPREPLY=( $(compgen -W \"");
        out.push_str(&commands.join(" "));
        out.push_str("\" -- \"$cur\") )\n");
        out.push_str("    return 0\n");
        out.push_str("  fi\n");
    }

    let mut flags: Vec<&Flag> = Vec::new();
    collect_flags(cmd, &mut flags);
    let words: Vec<String> = flags
        .iter()
        .flat_map(|f| {
            f.short
                .map(|c| vec![format!("-{}", c), format!("--{}", f.name)])
                .unwrap_or_else(|| vec![format!("--{}", f.name)])
        })
        .collect();
    out.push_str("  COMPREPLY=( $(compgen -W \"");
    out.push_str(&words.join(" "));
    out.push_str("\" -- \"$cur\") )\n");
    out.push_str("  return 0\n");
    out.push_str("}\n");
    let _ = writeln!(out, "complete -F _{name} {name}", name = cmd.name);
}

fn shell_zsh(cmd: &Command, out: &mut String) {
    use std::fmt::Write as _;

    let _ = write!(
        out,
        "#compdef {name}\n# zsh completion for {name}\n\n_arguments \\\n",
        name = cmd.name
    );

    let mut commands = Vec::new();
    collect_commands(cmd, &mut commands);
    let mut flags: Vec<&Flag> = Vec::new();
    collect_flags(cmd, &mut flags);

    let mut specs: Vec<String> = commands
        .iter()
        .map(|name| format!("'{}: :{}'", name, name))
        .collect();
    for flag in flags {
        let usage = quote(&flag.usage);
        match flag.short {
            Some(short) => specs.push(format!("'-{}[{}]'", short, usage)),
            None => specs.push(format!("'--{}[{}]'", flag.name, usage)),
        }
        if flag.short.is_some() {
            specs.push(format!("'--{}[{}]'", flag.name, usage));
        }
    }
    specs.push("'--help[Show help]'".to_owned());

    let last = specs.len() - 1;
    for (i, spec) in specs.iter().enumerate() {
        if i == last {
            let _ = writeln!(out, "  {}", spec);
        } else {
            let _ = writeln!(out, "  {} \\", spec);
        }
    }
}

fn shell_fish(cmd: &Command, out: &mut String) {
    use std::fmt::Write as _;

    let _ = write!(out, "# fish completion for {}\n\n", cmd.name);

    let mut commands = Vec::new();
    collect_commands(cmd, &mut commands);
    for name in &commands {
        let _ = writeln!(
            out,
            "complete -c {} -n '__fish_use_subcommand' -a {} -d 'subcommand'",
            cmd.name, name
        );
    }

    let mut flags: Vec<&Flag> = Vec::new();
    collect_flags(cmd, &mut flags);
    for flag in flags {
        let short = flag.short.map(|c| format!("-s {} ", c)).unwrap_or_default();
        let _ = writeln!(
            out,
            "complete -c {name} -f {short}-l {flag} -d '{usage}'",
            name = cmd.name,
            flag = flag.name,
            usage = quote(&flag.usage),
        );
    }
    let _ = writeln!(out, "complete -c {} -l help -d 'Show help'", cmd.name);
}

impl Command {
    /// Generate a completion script for the given shell: `"bash"`, `"zsh"`, or `"fish"`.
    pub fn gen_completion(&self, shell: &str) -> Result<String> {
        let mut out = String::new();
        match shell {
            "bash" => shell_bash(self, &mut out),
            "zsh" => shell_zsh(self, &mut out),
            "fish" => shell_fish(self, &mut out),
            other => {
                return Err(crate::WrCliError::UnsupportedCompletionShell(
                    other.to_owned(),
                ));
            }
        }
        Ok(out)
    }
}

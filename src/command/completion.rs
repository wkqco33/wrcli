//! Completion script generation for bash/zsh/fish.

use super::Command;
use crate::error::Result;

/// Recursively collect subcommand names reachable from `cmd`.
fn collect_commands(cmd: &Command, out: &mut Vec<String>) {
    for sub in &cmd.subcommands {
        out.push(sub.name.clone());
        collect_commands(sub, out);
    }
}

/// Collect flag names (with optional short forms) for a command and all its subcommands.
fn collect_flags(cmd: &Command, out: &mut Vec<String>) {
    for flag in cmd.flags.flags_iter() {
        if let Some(short) = flag.short {
            out.push(format!("-{}", short));
        }
        out.push(format!("--{}", flag.name));
    }
    for sub in &cmd.subcommands {
        collect_flags(sub, out);
    }
}

fn shell_bash(cmd: &Command, out: &mut String) {
    out.push_str(&format!(
        "#!/usr/bin/env bash\n# bash completion for {}\n\n",
        cmd.name
    ));
    out.push_str(&format!("_{}() {{\n", cmd.name));
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

    let mut flags = Vec::new();
    collect_flags(cmd, &mut flags);
    out.push_str("  COMPREPLY=( $(compgen -W \"");
    out.push_str(&flags.join(" "));
    out.push_str("\" -- \"$cur\") )\n");
    out.push_str("  return 0\n");
    out.push_str("}\n");
    out.push_str(&format!("complete -F _{} {} \"\n", cmd.name, cmd.name));
}

fn shell_zsh(cmd: &Command, out: &mut String) {
    out.push_str(&format!(
        "#compdef {}\n# zsh completion for {}\n\n",
        cmd.name, cmd.name
    ));
    out.push_str(&format!("#compdef {}\n", cmd.name));
    out.push_str("_arguments \\\n");

    let mut commands = Vec::new();
    collect_commands(cmd, &mut commands);
    for name in &commands {
        out.push_str(&format!("  '{}: :{}'\n", name, name));
    }
    for flag in cmd.flags.flags_iter() {
        if let Some(short) = flag.short {
            out.push_str(&format!(
                "  '-{}[{}]' '--{}[{}]'\n",
                short, flag.usage, flag.name, flag.usage
            ));
        } else {
            out.push_str(&format!("  '--{}[{}]'\n", flag.name, flag.usage));
        }
    }
    out.push_str("  '--help[Show help]'\n");
}

fn shell_fish(cmd: &Command, out: &mut String) {
    out.push_str(&format!("# fish completion for {}\n\n", cmd.name));

    let mut commands = Vec::new();
    collect_commands(cmd, &mut commands);
    for name in &commands {
        out.push_str(&format!(
            "complete -c {} -n '__fish_use_subcommand' -a {} -d 'subcommand'\n",
            cmd.name, name
        ));
    }

    for flag in cmd.flags.flags_iter() {
        let mut names = format!("--{}", flag.name);
        if let Some(short) = flag.short {
            names = format!("-{} {}", short, names);
        }
        let _ = &names;
        out.push_str(&format!(
            "complete -c {} -f -l {} -d '{}'\n",
            cmd.name, flag.name, flag.usage
        ));
    }
    out.push_str(&format!(
        "complete -c {} -l help -d 'Show help'\n",
        cmd.name
    ));
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

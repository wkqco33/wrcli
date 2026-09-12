//! Completion script generation for bash/zsh/fish.

use super::Command;
use super::dispatch::takes_value;
use crate::error::Result;
use crate::flag::Flag;

/// 내장 `help` 서브커맨드를 목록에 추가할지 여부.
///
/// 사용자가 `help`를 직접 등록했거나 보여줄 서브커맨드가 없으면 추가하지 않는다.
fn builtin_help_available(cmd: &Command) -> bool {
    cmd.find_subcommand("help").is_none() && cmd.subcommands.iter().any(|c| !c.hidden)
}

/// Recursively collect subcommand names reachable from `cmd`.
fn collect_commands(cmd: &Command, out: &mut Vec<String>) {
    for sub in &cmd.subcommands {
        if sub.hidden {
            continue;
        }
        out.push(sub.name.clone());
        collect_commands(sub, out);
    }
}

/// Collect flags for a command and all its subcommands, deduplicated by name.
fn collect_flags<'a>(cmd: &'a Command, out: &mut Vec<&'a Flag>) {
    for flag in cmd.flags.flags_iter() {
        if flag.hidden {
            continue;
        }
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
    if builtin_help_available(cmd) {
        commands.push("help".to_owned());
    }
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
    let mut words: Vec<String> = flags
        .iter()
        .flat_map(|f| {
            f.short
                .map(|c| vec![format!("-{}", c), format!("--{}", f.name)])
                .unwrap_or_else(|| vec![format!("--{}", f.name)])
        })
        .collect();
    words.push("--help".to_owned());
    if cmd.version.is_some() {
        words.push("--version".to_owned());
    }
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
    if builtin_help_available(cmd) {
        commands.push("help".to_owned());
    }
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
    if builtin_help_available(cmd) {
        commands.push("help".to_owned());
    }
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

    /// 현재 입력 중인 토큰에 대한 동적 completion 후보를 계산.
    ///
    /// `args`는 프로그램 이름 이후의 인자들이며, 마지막 원소가 완성 중인 토큰이다.
    /// 하위 커맨드로 이동하며 플래그/서브커맨드/`arg_candidates` 후보를 반환한다.
    pub fn complete(&self, args: &[String]) -> Vec<String> {
        let current = args.last().cloned().unwrap_or_default();
        let prior = &args[..args.len().saturating_sub(1)];

        let mut cmd = self;
        let mut positional: Vec<String> = Vec::new();
        let mut i = 0;
        while i < prior.len() {
            let tok = &prior[i];
            if tok == "--" {
                positional.extend(prior[i + 1..].iter().cloned());
                break;
            }
            if let Some(rest) = tok.strip_prefix("--") {
                let (name, has_eq) = match rest.split_once('=') {
                    Some((n, _)) => (n, true),
                    None => (rest, false),
                };
                let consumes_next = !has_eq
                    && cmd
                        .flags
                        .get_flag(name)
                        .map(|f| takes_value(&f.default))
                        .unwrap_or(false);
                i += if consumes_next { 2 } else { 1 };
                continue;
            }
            if tok.starts_with('-') && tok.len() > 1 {
                i += 1;
                continue;
            }
            if let Some(sub) = cmd
                .subcommands
                .iter()
                .find(|c| !c.hidden && (c.name == *tok || c.aliases.iter().any(|a| a == tok)))
            {
                cmd = sub;
                positional.clear();
                i += 1;
                continue;
            }
            positional.push(tok.clone());
            i += 1;
        }

        if let Some(prefix) = current.strip_prefix("--") {
            let mut out: Vec<String> = cmd
                .flags
                .flags_iter()
                .filter(|f| !f.hidden && f.name.starts_with(prefix))
                .map(|f| format!("--{}", f.name))
                .collect();
            if "help".starts_with(prefix) {
                out.push("--help".to_owned());
            }
            if cmd.version.is_some() && "version".starts_with(prefix) {
                out.push("--version".to_owned());
            }
            return out;
        }

        if let Some(prefix) = current.strip_prefix('-') {
            let first = prefix.chars().next();
            let mut out: Vec<String> = cmd
                .flags
                .flags_iter()
                .filter(|f| !f.hidden)
                .filter_map(|f| f.short.map(|c| (c, format!("-{c}"))))
                .filter(|(c, _)| first.map(|p| *c == p).unwrap_or(true))
                .map(|(_, s)| s)
                .collect();
            if first.map(|p| p == 'h').unwrap_or(true) {
                out.push("-h".to_owned());
            }
            if cmd.version.is_some() && first.map(|p| p == 'V').unwrap_or(true) {
                out.push("-V".to_owned());
            }
            return out;
        }

        let mut out: Vec<String> = cmd
            .subcommands
            .iter()
            .filter(|c| !c.hidden)
            .map(|c| c.name.clone())
            .collect();
        if std::ptr::eq(cmd, self) && builtin_help_available(cmd) {
            out.push("help".to_owned());
        }
        if let Some(f) = &cmd.arg_candidates {
            out.extend(f(&positional));
        }
        out.retain(|c| c.starts_with(&current));
        out
    }

    /// `__complete` 프로토콜 요청이면 후보를 반환.
    ///
    /// `args`는 프로그램 이름 이후의 인자들이다. 첫 토큰이 `__complete`가 아니면 `None`.
    ///
    /// ```no_run
    /// # use wrcli::Command;
    /// let cmd = Command::new("myapp").subcommand(Command::new("serve"));
    /// let args: Vec<String> = std::env::args().skip(1).collect();
    /// if let Some(candidates) = cmd.completion_request(args) {
    ///     for c in candidates { println!("{}", c); }
    ///     return;
    /// }
    /// cmd.execute().unwrap();
    /// ```
    pub fn completion_request(&self, args: Vec<String>) -> Option<Vec<String>> {
        if args.first().map(String::as_str) != Some("__complete") {
            return None;
        }
        let rest = &args[1..];
        let words = if rest.is_empty() {
            vec![String::new()]
        } else {
            rest.to_vec()
        };
        Some(self.complete(&words))
    }
}

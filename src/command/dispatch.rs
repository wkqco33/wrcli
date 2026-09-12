use super::command::{Command, FlagGroup, RunFn};
use super::context::CommandContext;
use super::help;
use crate::config::Config;
use crate::error::{Result, WrCliError};
use crate::flag::{Flag, FlagSet, FlagValue};
use crate::style::{ColorChoice, reset_color_choice, set_color_choice};

/// Whether the flag takes a value (i.e. is not a bool).
pub(crate) fn takes_value(default: &FlagValue) -> bool {
    !matches!(default, FlagValue::Bool(_))
}

/// Scan the raw argv for `--no-color` / `--color=<when>`.
///
/// Help itself also decides on color, so apply this before flag parsing and subcommand routing.
fn scan_color_flags(args: &[String]) -> Option<ColorChoice> {
    let mut choice = None;
    for (i, a) in args.iter().enumerate() {
        if a == "--no-color" {
            choice = Some(ColorChoice::Never);
        } else if let Some(v) = a.strip_prefix("--color=") {
            choice = Some(parse_color_choice(v));
        } else if a == "--color"
            && let Some(v) = args.get(i + 1)
        {
            choice = Some(parse_color_choice(v));
        }
    }
    choice
}

fn parse_color_choice(value: &str) -> ColorChoice {
    match value.to_ascii_lowercase().as_str() {
        "always" => ColorChoice::Always,
        "never" => ColorChoice::Never,
        _ => ColorChoice::Auto,
    }
}

/// Validate the collected flag constraint groups against the leaf command's actual input.
///
/// Skip validation when none of the group's flags are registered in the leaf FlagSet
/// (e.g. when a parent local flag constraint was collected while running a subcommand).
fn validate_flag_groups(groups: &[FlagGroup], flags: &FlagSet) -> Result<()> {
    let registered = |names: &[String]| names.iter().any(|n| flags.get_flag(n).is_some());
    for group in groups {
        match group {
            FlagGroup::MutuallyExclusive(names) => {
                if !registered(names) {
                    continue;
                }
                let provided: Vec<String> =
                    names.iter().filter(|n| flags.is_set(n)).cloned().collect();
                if provided.len() > 1 {
                    return Err(WrCliError::MutuallyExclusiveFlags {
                        group: names.clone(),
                        provided,
                    });
                }
            }
            FlagGroup::RequiredTogether(names) => {
                if !registered(names) {
                    continue;
                }
                let provided = names.iter().filter(|n| flags.is_set(n)).count();
                if provided > 0 && provided < names.len() {
                    let missing: Vec<String> =
                        names.iter().filter(|n| !flags.is_set(n)).cloned().collect();
                    return Err(WrCliError::RequiredFlagsTogether {
                        group: names.clone(),
                        missing,
                    });
                }
            }
            FlagGroup::OneRequired(names) => {
                if !registered(names) {
                    continue;
                }
                if !names.iter().any(|n| flags.is_set(n)) {
                    return Err(WrCliError::OneFlagRequired {
                        group: names.clone(),
                    });
                }
            }
        }
    }
    Ok(())
}

/// Find the index of the first token that can serve as a subcommand candidate or an
/// unrecognized positional argument.
///
/// Skip value tokens of value-consuming flags (`--name value`, `-c value`) without
/// actually calling `flags.parse()`, so a value that happens to equal a subcommand
/// name is not mistaken for a subcommand. On encountering the `--` sentinel, stop
/// searching for a candidate because everything after it is a literal positional arg.
fn find_positional_candidate(args: &[String], flags: &FlagSet) -> Option<usize> {
    let mut i = 0;
    while i < args.len() {
        let a = args[i].as_str();
        if a == "--" {
            return None;
        }
        if let Some(rest) = a.strip_prefix("--") {
            let name = match rest.find('=') {
                Some(eq) => &rest[..eq],
                None => rest,
            };
            let consumes_next = !rest.contains('=')
                && flags
                    .get_flag(name)
                    .map(|f| takes_value(&f.default))
                    .unwrap_or(false);
            i += if consumes_next { 2 } else { 1 };
            continue;
        }
        if a.starts_with('-') && a.len() > 1 {
            let last = a[1..].chars().last().unwrap();
            let consumes_next = flags
                .short_flag(last)
                .map(|f| takes_value(&f.default))
                .unwrap_or(false);
            i += if consumes_next { 2 } else { 1 };
            continue;
        }
        return Some(i);
    }
    None
}

impl Command {
    /// Entry point: parse `std::env::args()` (excluding `argv[0]`) and run.
    pub fn execute(self) -> Result<()> {
        let args: Vec<String> = std::env::args().skip(1).collect();
        self.execute_with(args)
    }

    /// Test variant: parse the given argument list and run.
    pub fn execute_with(mut self, args: Vec<String>) -> Result<()> {
        #[cfg(feature = "signal")]
        if let Some(msg) = self.interrupt_message {
            crate::signal::install(msg);
        }
        // `--no-color`/`--color` must take effect before help rendering, so scan the raw argv first.
        let color = scan_color_flags(&args);
        if let Some(choice) = color {
            set_color_choice(choice);
        }
        let mut config = self.config.take().unwrap_or_default();
        let mut pre_chain: Vec<RunFn> = Vec::new();
        let mut post_chain: Vec<RunFn> = Vec::new();
        let mut command_path: Vec<String> = Vec::new();
        let mut groups: Vec<FlagGroup> = Vec::new();
        let result = self.dispatch(
            args,
            &mut config,
            &mut pre_chain,
            &mut post_chain,
            &mut command_path,
            &mut groups,
        );
        if color.is_some() {
            reset_color_choice();
        }
        result
    }

    /// If an error occurs during execution, print it to stderr and exit the process.
    ///
    /// On success exits with code 0, on usage errors with 2, and with 1 otherwise.
    /// Calling this in a test terminates the test process, so use `execute()` instead.
    pub fn execute_or_exit(self) -> ! {
        let bug_report = self.bug_report_url.clone();
        match self.execute() {
            Ok(()) => std::process::exit(0),
            Err(e) => {
                eprintln!("Error: {}", e);
                if !e.is_usage_error()
                    && let Some(url) = bug_report
                {
                    eprintln!("\nThis looks like a bug. Please report it: {}", url);
                }
                std::process::exit(e.exit_code());
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn dispatch(
        mut self,
        mut args: Vec<String>,
        config: &mut Config,
        pre_chain: &mut Vec<RunFn>,
        post_chain: &mut Vec<RunFn>,
        command_path: &mut Vec<String>,
        groups: &mut Vec<FlagGroup>,
    ) -> Result<()> {
        command_path.push(self.name.clone());

        // Built-in `help` subcommand: active only when the user has not registered `help` themselves.
        if !self.has_subcommand_named("help")
            && let Some(idx) = find_positional_candidate(&args, &self.flags)
            && args[idx] == "help"
        {
            let target: Vec<String> = args[idx + 1..].to_vec();
            return self.dispatch_help(&target, command_path.as_slice());
        }

        groups.extend(
            self.mutually_exclusive
                .iter()
                .cloned()
                .map(FlagGroup::MutuallyExclusive),
        );
        groups.extend(
            self.required_together
                .iter()
                .cloned()
                .map(FlagGroup::RequiredTogether),
        );
        groups.extend(
            self.one_required
                .iter()
                .cloned()
                .map(FlagGroup::OneRequired),
        );

        if let Some(f) = self.persistent_pre_run.take() {
            pre_chain.push(f);
        }
        if let Some(f) = self.persistent_post_run.take() {
            // Reverse iteration after push preserves leaf-to-root order (O(1) versus insert(0))
            post_chain.push(f);
        }

        // Try subcommand routing first so that `app serve --help` prints serve's help.
        // Skip the value tokens of value-consuming flags and take the first true positional token as the candidate.
        let candidate = find_positional_candidate(&args, &self.flags);
        let subcommand_pos = candidate.and_then(|idx| {
            let name = args[idx].as_str();
            self.subcommands
                .iter()
                .position(|c| c.name == name || c.aliases.iter().any(|a| a == name))
                .map(|pos| (idx, pos))
        });

        // Meta flags must be scanned across the whole argv regardless of position so help
        // wins even for `app unknown-sub --help`. Parent local flags before the subcommand are
        // not consumed here either; they are passed through so the leaf handles help/version.
        let found_help = args.iter().any(|a| a == "--help" || a == "-h");
        let found_version = args.iter().any(|a| a == "--version" || a == "-V");
        let meta = found_help || found_version;

        if let Some((arg_idx, cmd_pos)) = subcommand_pos {
            args.remove(arg_idx);
            let mut child = self.subcommands.remove(cmd_pos);
            log::debug!(
                "subcommand routing: {} -> {}",
                command_path.join(" "),
                child.name
            );
            // Propagate persistent flags to the child — if already present, skip without cloning
            for flag in self.flags.persistent_flags() {
                child.flags.add_inherited(flag);
            }
            // Docs/support links are inherited unless the subcommand defines its own.
            if child.support_url.is_none() {
                child.support_url = self.support_url.clone();
            }
            if child.docs_url.is_none() {
                child.docs_url = self.docs_url.clone();
            }

            if meta {
                return child.dispatch(args, config, pre_chain, post_chain, command_path, groups);
            }

            // Consume parent flags (including local ones) that precede the subcommand with the
            // parent FlagSet, and pass their values down so the leaf context can read them too.
            let child_args = args.split_off(arg_idx);
            let _ = self.flags.parse_partial(args)?;
            child.flags.inherit_values(&self.flags);
            return child.dispatch(
                child_args,
                config,
                pre_chain,
                post_chain,
                command_path,
                groups,
            );
        }

        if found_help {
            help::print_help(
                &self,
                &self.flags,
                command_path,
                self.support_url.as_deref(),
                self.docs_url.as_deref(),
            );
            return Ok(());
        }

        if found_version && self.version.is_some() {
            println!("{} {}", self.name, self.version.as_deref().unwrap_or(""));
            return Ok(());
        }

        // With registered subcommands present, return a clear error for an unrecognized token
        if !self.subcommands.is_empty()
            && let Some(idx) = candidate
        {
            let name = args[idx].clone();
            log::warn!("unknown subcommand '{}' for '{}'", name, self.name);
            return Err(self.unknown_subcommand_error(&name, &self.name));
        }

        // ── Leaf command ──────────────────────────────────────────────────────

        let positional = self.flags.parse(args)?;
        validate_flag_groups(groups, &self.flags)?;

        if let Some(ref validator) = self.arg_validator {
            validator(&positional)?;
        }

        // Seed flags not set explicitly with values from the config store (config → flag).
        self.flags.seed_from_config(config);

        // Bind only flags explicitly entered by the user to Config layer 4
        for (name, fv) in self.flags.values_iter() {
            config.bind_flag_value(name, crate::config::ConfigValue::from(fv));
        }

        let config: &Config = config;
        let ctx = CommandContext {
            command_path: command_path.clone(),
            args: positional,
            flags: &self.flags,
            config,
        };

        log::trace!("lifecycle hooks start: {}", command_path.join(" "));

        if let Some(msg) = &self.deprecated {
            eprintln!("Command \"{}\" is deprecated: {}", self.name, msg);
        }

        for f in pre_chain.iter() {
            f(&ctx);
        }
        if let Some(ref f) = self.pre_run {
            f(&ctx);
        }

        if let Some(ref f) = self.run_e {
            f(&ctx)?;
        } else if let Some(ref f) = self.run {
            f(&ctx);
        } else {
            // A parent command with only subcommands shows help and finishes successfully (Cobra behavior).
            let has_visible_subcommands = self.subcommands.iter().any(|c| !c.hidden);
            if has_visible_subcommands || self.help_on_missing_runner {
                help::print_help(
                    &self,
                    &self.flags,
                    command_path,
                    self.support_url.as_deref(),
                    self.docs_url.as_deref(),
                );
                return Ok(());
            }
            log::warn!("command '{}' has no run handler", self.name);
            return Err(WrCliError::CommandHasNoRunner(self.name.clone()));
        }

        if let Some(ref f) = self.post_run {
            f(&ctx);
        }
        for f in post_chain.iter().rev() {
            f(&ctx);
        }

        log::trace!("lifecycle hooks complete: {}", command_path.join(" "));

        Ok(())
    }

    /// Handle the built-in `help` subcommand: print the help for the command at path `target`.
    ///
    /// If a persistent flag of an ancestor command appears along the path, mark it as inherited
    /// and expose it in the `Global Flags:` section.
    fn dispatch_help(&self, target: &[String], command_path: &[String]) -> Result<()> {
        let mut cmd = self;
        let mut full_path = command_path.to_vec();
        let mut inherited: Vec<Flag> = Vec::new();
        let mut support_url = self.support_url.clone();
        let mut docs_url = self.docs_url.clone();
        for seg in target {
            for f in cmd.flags.persistent_flags() {
                inherited.push(f.clone());
            }
            if cmd.support_url.is_some() {
                support_url = cmd.support_url.clone();
            }
            if cmd.docs_url.is_some() {
                docs_url = cmd.docs_url.clone();
            }
            let Some(next) = cmd.find_subcommand(seg) else {
                return Err(cmd.unknown_subcommand_error(seg, &full_path.join(" ")));
            };
            cmd = next;
            full_path.push(cmd.name.clone());
        }
        if cmd.support_url.is_some() {
            support_url = cmd.support_url.clone();
        }
        if cmd.docs_url.is_some() {
            docs_url = cmd.docs_url.clone();
        }
        let mut flags = cmd.flags.clone();
        for f in &inherited {
            flags.add_inherited(f);
        }
        help::print_help(
            cmd,
            &flags,
            &full_path,
            support_url.as_deref(),
            docs_url.as_deref(),
        );
        Ok(())
    }
}

pub mod args;
pub mod context;
pub mod help;

use crate::command::context::CommandContext;
use crate::config::Config;
use crate::error::{Result, WrCliError};
use crate::flag::{Flag, FlagSet};

/// Infallible run callback type.
pub type RunFn = Box<dyn for<'ctx> Fn(&CommandContext<'ctx>) + Send + Sync>;
/// Fallible run callback type — returning an error aborts the lifecycle chain.
pub type RunEFn =
    Box<dyn for<'ctx> Fn(&CommandContext<'ctx>) -> Result<()> + Send + Sync>;

/// A command in the CLI tree.
///
/// Build via the fluent builder API, then call [`Command::execute`] on the root command.
///
/// # Example
/// ```no_run
/// use wrcli::{Command, Flag, FlagValue};
///
/// Command::new("myapp")
///     .short("My application")
///     .version("1.0.0")
///     .flag(Flag::new("verbose", FlagValue::Bool(false), "enable verbose output").short('v'))
///     .on_run(|ctx| {
///         println!("verbose={}", ctx.get_bool("verbose").unwrap_or(false));
///     })
///     .execute()
///     .unwrap();
/// ```
pub struct Command {
    pub(crate) name: String,
    pub(crate) short: String,
    pub(crate) long: String,
    pub(crate) version: Option<String>,
    pub(crate) aliases: Vec<String>,

    pub(crate) flags: FlagSet,
    pub(crate) subcommands: Vec<Command>,

    pub(crate) arg_validator: Option<args::ArgValidator>,

    pub(crate) persistent_pre_run: Option<RunFn>,
    pub(crate) pre_run: Option<RunFn>,
    pub(crate) run: Option<RunFn>,
    pub(crate) run_e: Option<RunEFn>,
    pub(crate) post_run: Option<RunFn>,
    pub(crate) persistent_post_run: Option<RunFn>,

    /// Only the root command holds a `Config`; it is passed by reference during execution.
    pub(crate) config: Option<Config>,
}

// ── Builder ──────────────────────────────────────────────────────────────────

impl Command {
    pub fn new(name: &str) -> Self {
        Command {
            name: name.to_owned(),
            short: String::new(),
            long: String::new(),
            version: None,
            aliases: Vec::new(),
            flags: FlagSet::new(),
            subcommands: Vec::new(),
            arg_validator: None,
            persistent_pre_run: None,
            pre_run: None,
            run: None,
            run_e: None,
            post_run: None,
            persistent_post_run: None,
            config: None,
        }
    }

    /// Short one-line description (shown in parent's subcommand list).
    pub fn short(mut self, s: &str) -> Self {
        self.short = s.to_owned();
        self
    }

    /// Long description (shown in this command's own `--help` output).
    pub fn long(mut self, s: &str) -> Self {
        self.long = s.to_owned();
        self
    }

    /// Version string. Enables `--version` / `-V` flags on this command.
    pub fn version(mut self, v: &str) -> Self {
        self.version = Some(v.to_owned());
        self
    }

    /// Add an alternative name for this command.
    pub fn alias(mut self, a: &str) -> Self {
        self.aliases.push(a.to_owned());
        self
    }

    /// Add a local flag (not propagated to subcommands).
    pub fn flag(mut self, flag: Flag) -> Self {
        self.flags.add(flag);
        self
    }

    /// Add a persistent flag (propagated to all subcommands).
    pub fn persistent_flag(mut self, mut flag: Flag) -> Self {
        flag.persistent = true;
        self.flags.add(flag);
        self
    }

    /// Add a subcommand.
    pub fn subcommand(mut self, cmd: Command) -> Self {
        self.subcommands.push(cmd);
        self
    }

    /// Set a positional argument validator. See the [`args`] module for built-ins.
    pub fn args(mut self, validator: args::ArgValidator) -> Self {
        self.arg_validator = Some(validator);
        self
    }

    /// Attach the config store (Viper equivalent) to this command tree.
    pub fn with_config(mut self, config: Config) -> Self {
        self.config = Some(config);
        self
    }

    // ── Lifecycle callbacks ───────────────────────────────────────────────

    /// Called before all commands in the chain, from root to leaf.
    pub fn on_persistent_pre_run<F>(mut self, f: F) -> Self
    where
        F: for<'ctx> Fn(&CommandContext<'ctx>) + Send + Sync + 'static,
    {
        self.persistent_pre_run = Some(Box::new(f));
        self
    }

    /// Called before `on_run` / `on_run_e` on the matched leaf command only.
    pub fn on_pre_run<F>(mut self, f: F) -> Self
    where
        F: for<'ctx> Fn(&CommandContext<'ctx>) + Send + Sync + 'static,
    {
        self.pre_run = Some(Box::new(f));
        self
    }

    /// Infallible run handler for this command.
    pub fn on_run<F>(mut self, f: F) -> Self
    where
        F: for<'ctx> Fn(&CommandContext<'ctx>) + Send + Sync + 'static,
    {
        self.run = Some(Box::new(f));
        self
    }

    /// Fallible run handler. Returning `Err` aborts post-run hooks and surfaces the error.
    pub fn on_run_e<F>(mut self, f: F) -> Self
    where
        F: for<'ctx> Fn(&CommandContext<'ctx>) -> Result<()> + Send + Sync + 'static,
    {
        self.run_e = Some(Box::new(f));
        self
    }

    /// Called after `on_run` / `on_run_e` on the matched leaf command only.
    pub fn on_post_run<F>(mut self, f: F) -> Self
    where
        F: for<'ctx> Fn(&CommandContext<'ctx>) + Send + Sync + 'static,
    {
        self.post_run = Some(Box::new(f));
        self
    }

    /// Called after all commands in the chain, from leaf to root.
    pub fn on_persistent_post_run<F>(mut self, f: F) -> Self
    where
        F: for<'ctx> Fn(&CommandContext<'ctx>) + Send + Sync + 'static,
    {
        self.persistent_post_run = Some(Box::new(f));
        self
    }
}

// ── Execution ─────────────────────────────────────────────────────────────────

impl Command {
    /// Entry point: parse `std::env::args()` (skipping argv[0]) and execute.
    pub fn execute(self) -> Result<()> {
        let args: Vec<String> = std::env::args().skip(1).collect();
        self.execute_with(args)
    }

    /// Testable variant: parse the given argument list and execute.
    pub fn execute_with(mut self, args: Vec<String>) -> Result<()> {
        let mut config = self.config.take().unwrap_or_default();
        let mut pre_chain: Vec<RunFn> = Vec::new();
        let mut post_chain: Vec<RunFn> = Vec::new();
        let mut command_path: Vec<String> = Vec::new();
        self.dispatch(args, &mut config, &mut pre_chain, &mut post_chain, &mut command_path)
    }

    fn dispatch(
        mut self,
        mut args: Vec<String>,
        config: &mut Config,
        pre_chain: &mut Vec<RunFn>,
        post_chain: &mut Vec<RunFn>,
        command_path: &mut Vec<String>,
    ) -> Result<()> {
        command_path.push(self.name.clone());

        // Collect this command's persistent hooks into the chain.
        if let Some(f) = self.persistent_pre_run.take() {
            pre_chain.push(f);
        }
        if let Some(f) = self.persistent_post_run.take() {
            // Post runs fire leaf→root, so we insert at the front.
            post_chain.insert(0, f);
        }

        // Try subcommand routing FIRST so that `app serve --help` prints serve's
        // help, not root's help.  The first non-flag, non-`--help` token is the
        // candidate subcommand name.
        let candidate = args.iter().enumerate().find(|(_, a)| {
            !a.starts_with('-')
        });
        let subcommand_pos = candidate.and_then(|(idx, name)| {
            self.subcommands
                .iter()
                .position(|c| &c.name == name || c.aliases.iter().any(|a| a == name))
                .map(|pos| (idx, pos))
        });

        if let Some((arg_idx, cmd_pos)) = subcommand_pos {
            args.remove(arg_idx);
            let mut child = self.subcommands.remove(cmd_pos);
            // Propagate persistent flags top-down — clone only when actually absent.
            for flag in self.flags.persistent_flags() {
                child.flags.add_if_absent(flag);
            }
            return child.dispatch(args, config, pre_chain, post_chain, command_path);

        }

        // No subcommand matched — now handle special meta-flags for this command.
        if args.iter().any(|a| a == "--help" || a == "-h") {
            help::print_help(
                &self.name,
                &self.short,
                &self.long,
                &self.version,
                &self.flags,
                &self.subcommands,
                command_path,
            );
            return Ok(());
        }

        if self.version.is_some() && args.iter().any(|a| a == "--version" || a == "-V") {
            println!("{} {}", self.name, self.version.as_deref().unwrap_or(""));
            return Ok(());
        }

        // If the command has registered subcommands and the user supplied an unrecognized
        // non-flag token, emit a helpful error rather than treating it as a positional arg.
        if !self.subcommands.is_empty() {
            if let Some(unknown) = args.iter().find(|a| !a.starts_with('-')) {
                return Err(WrCliError::UnknownSubcommand {
                    name: unknown.clone(),
                    parent: self.name.clone(),
                });
            }
        }

        // ── Leaf command ─────────────────────────────────────────────────────

        // Parse flags; remaining are positional args.
        let positional = self.flags.parse(args)?;

        // Validate positional args.
        if let Some(ref validator) = self.arg_validator {
            validator(&positional)?;
        }

        // Bind explicitly-set flag values into the config as layer 4 (highest priority).
        // Only flags the user actually provided on the command line are injected —
        // default values are not, so they don't shadow config-file or env values.
        // Iterate values directly — avoids two HashMap lookups per flag.
        for (name, fv) in self.flags.values_iter() {
            config.bind_flag_value(name, crate::config::ConfigValue::from(fv));
        }

        // Build context — config is now fully populated.
        // Use a shared reference from here on so both flags and config can be borrowed.
        let config: &Config = config;
        let ctx = CommandContext {
            command_path: command_path.clone(),
            args: positional,
            flags: &self.flags,
            config,
        };

        // Fire lifecycle chain.
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
            // No runner registered — print help and signal it.
            help::print_help(
                &self.name,
                &self.short,
                &self.long,
                &self.version,
                &self.flags,
                &self.subcommands,
                command_path,
            );
            return Err(WrCliError::CommandHasNoRunner(self.name.clone()));
        }

        if let Some(ref f) = self.post_run {
            f(&ctx);
        }
        for f in post_chain.iter() {
            f(&ctx);
        }

        Ok(())
    }
}

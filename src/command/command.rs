use super::args;
use crate::command::context::CommandContext;
use crate::config::Config;
use crate::error::{Result, WrCliError};
use crate::flag::{Flag, FlagSet, FlagValue};

/// Callback type invoked without arguments.
pub type RunFn = Box<dyn for<'ctx> Fn(&CommandContext<'ctx>) + Send + Sync>;
/// Callback type that can return an error. Returning `Err` aborts the lifecycle chain.
pub type RunEFn = Box<dyn for<'ctx> Fn(&CommandContext<'ctx>) -> Result<()> + Send + Sync>;

/// A command node in the CLI tree.
///
/// Configure it with the fluent builder API, then call [`crate::Command::execute`] on the root command.
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
    pub(crate) hidden: bool,
    pub(crate) deprecated: Option<String>,
    pub(crate) suggest_for: Vec<String>,
    /// Positional hint shown on the `--help` usage line (e.g. `"<name>"`).
    pub(crate) usage_args: Option<String>,
    /// List of examples shown after Usage in help (clig.dev: keep examples up front).
    pub(crate) examples: Vec<String>,
    /// Issue/feedback URL shown at the bottom of help.
    pub(crate) support_url: Option<String>,
    /// Documentation URL shown at the bottom of help. `{command}` is replaced with the full command path.
    pub(crate) docs_url: Option<String>,
    /// Bug report URL that `execute_or_exit` shows on an unexpected error.
    pub(crate) bug_report_url: Option<String>,
    /// Print help and exit successfully when there is no runner.
    pub(crate) help_on_missing_runner: bool,
    /// Message to print on Ctrl-C (`signal` feature).
    #[cfg(feature = "signal")]
    pub(crate) interrupt_message: Option<&'static str>,

    pub(crate) flags: FlagSet,
    pub(crate) subcommands: Vec<Command>,

    pub(crate) arg_validator: Option<args::ArgValidator>,
    /// Generator for dynamic completion candidates for positional arguments.
    pub(crate) arg_candidates: Option<ArgCandidatesFn>,
    pub(crate) mutually_exclusive: Vec<Vec<String>>,
    pub(crate) required_together: Vec<Vec<String>>,
    pub(crate) one_required: Vec<Vec<String>>,

    pub(crate) persistent_pre_run: Option<RunFn>,
    pub(crate) pre_run: Option<RunFn>,
    pub(crate) run: Option<RunFn>,
    pub(crate) run_e: Option<RunEFn>,
    pub(crate) post_run: Option<RunFn>,
    pub(crate) persistent_post_run: Option<RunFn>,

    /// Only the root command holds the Config. It is passed by reference during execution.
    pub(crate) config: Option<Config>,
}

// ── Builder ──────────────────────────────────────────────────────────────────

impl Command {
    pub fn new(name: &str) -> Self {
        let mut flags = FlagSet::new();
        flags.set_command_name(name);
        Command {
            name: name.to_owned(),
            short: String::new(),
            long: String::new(),
            version: None,
            aliases: Vec::new(),
            hidden: false,
            deprecated: None,
            suggest_for: Vec::new(),
            usage_args: None,
            examples: Vec::new(),
            support_url: None,
            docs_url: None,
            bug_report_url: None,
            help_on_missing_runner: false,
            #[cfg(feature = "signal")]
            interrupt_message: None,
            flags,
            subcommands: Vec::new(),
            arg_validator: None,
            arg_candidates: None,
            mutually_exclusive: Vec::new(),
            required_together: Vec::new(),
            one_required: Vec::new(),
            persistent_pre_run: None,
            pre_run: None,
            run: None,
            run_e: None,
            post_run: None,
            persistent_post_run: None,
            config: None,
        }
    }

    /// One-line description shown in the parent command's list.
    pub fn short(mut self, s: &str) -> Self {
        self.short = s.to_owned();
        self
    }

    /// Long description shown in this command's own `--help`.
    pub fn long(mut self, s: &str) -> Self {
        self.long = s.to_owned();
        self
    }

    /// Version string. Enables the `--version` / `-V` flags.
    pub fn version(mut self, v: &str) -> Self {
        self.version = Some(v.to_owned());
        self
    }

    /// Add a command alias.
    pub fn alias(mut self, a: &str) -> Self {
        self.aliases.push(a.to_owned());
        self
    }

    /// Hide this command from help and completion listings (it can still run).
    pub fn hidden(mut self) -> Self {
        self.hidden = true;
        self
    }

    /// Mark the command as deprecated. Prints a warning to stderr when it runs.
    pub fn deprecated(mut self, msg: &str) -> Self {
        self.deprecated = Some(msg.to_owned());
        self
    }

    /// Add an alias used to suggest this command on typos (Cobra's `SuggestFor`).
    ///
    /// Unlike a real alias, it cannot be executed and is only used as a `Did you mean` candidate.
    pub fn suggest_for(mut self, name: &str) -> Self {
        self.suggest_for.push(name.to_owned());
        self
    }

    /// Positional hint shown on the `--help` usage line (e.g. `"<name>"`, `"SRC DST"`).
    pub fn usage_args(mut self, hint: &str) -> Self {
        self.usage_args = Some(hint.to_owned());
        self
    }

    /// Add an example shown right after Usage in help (can be called multiple times).
    pub fn example(mut self, e: &str) -> Self {
        self.examples.push(e.to_owned());
        self
    }

    /// Issue/feedback URL shown at the bottom of help.
    pub fn support_url(mut self, url: &str) -> Self {
        self.support_url = Some(url.to_owned());
        self
    }

    /// Documentation URL shown at the bottom of help. `{command}` is replaced with the full command path.
    pub fn docs_url(mut self, url: &str) -> Self {
        self.docs_url = Some(url.to_owned());
        self
    }

    /// Bug report URL that [`Command::execute_or_exit`] shows on an unexpected error.
    pub fn bug_report_url(mut self, url: &str) -> Self {
        self.bug_report_url = Some(url.to_owned());
        self
    }

    /// Print help and finish successfully (exit code 0) even without a runner.
    ///
    /// Parent commands with subcommands do this automatically without this setting.
    pub fn help_on_missing_runner(mut self) -> Self {
        self.help_on_missing_runner = true;
        self
    }

    /// On Ctrl-C (SIGINT), print the message immediately and exit with code 130.
    ///
    /// Requires the `signal` feature.
    #[cfg(feature = "signal")]
    pub fn interrupt_message(mut self, message: &'static str) -> Self {
        self.interrupt_message = Some(message);
        self
    }

    /// The list of registered examples.
    pub fn examples(&self) -> &[String] {
        &self.examples
    }

    /// Look up a subcommand by name or alias.
    pub(crate) fn find_subcommand(&self, name: &str) -> Option<&Command> {
        self.subcommands
            .iter()
            .find(|c| c.name == name || c.aliases.iter().any(|a| a == name))
    }

    /// Whether a subcommand with name/alias `name` is registered.
    pub(crate) fn has_subcommand_named(&self, name: &str) -> bool {
        self.find_subcommand(name).is_some()
    }

    /// Edit-distance based subcommand suggestions for `name` (excluding hidden ones, including `suggest_for`).
    pub(crate) fn subcommand_suggestions(&self, name: &str) -> Vec<String> {
        let mut suggestions = crate::suggest::closest(
            name,
            self.subcommands.iter().filter(|c| !c.hidden).flat_map(|c| {
                std::iter::once(c.name.as_str()).chain(c.aliases.iter().map(String::as_str))
            }),
        );
        for cmd in self.subcommands.iter().filter(|c| !c.hidden) {
            let explicit = cmd.suggest_for.iter().any(|s| s.eq_ignore_ascii_case(name));
            if explicit && !suggestions.contains(&cmd.name) {
                suggestions.push(cmd.name.clone());
            }
        }
        suggestions
    }

    /// Build an unknown-subcommand error (including suggestions).
    pub(crate) fn unknown_subcommand_error(&self, name: &str, parent: &str) -> WrCliError {
        WrCliError::UnknownSubcommand {
            name: name.to_owned(),
            parent: parent.to_owned(),
            suggestions: self.subcommand_suggestions(name),
        }
    }

    /// Error if two or more flags in this group are specified together.
    pub fn mutually_exclusive(mut self, flags: &[&str]) -> Self {
        self.mutually_exclusive.push(names(flags));
        self
    }

    /// Error if only some flags in this group are specified (all or nothing).
    pub fn required_together(mut self, flags: &[&str]) -> Self {
        self.required_together.push(names(flags));
        self
    }

    /// At least one flag in this group must be specified.
    pub fn one_required(mut self, flags: &[&str]) -> Self {
        self.one_required.push(names(flags));
        self
    }

    /// Add a local flag (not propagated to subcommands).
    pub fn flag(mut self, flag: Flag) -> Self {
        self.flags.add(flag);
        self
    }

    /// Add a persistent flag (automatically propagated to all subcommands).
    pub fn persistent_flag(mut self, mut flag: Flag) -> Self {
        flag.persistent = true;
        self.flags.add(flag);
        self
    }

    /// Register the clig.dev standard flag bundle.
    ///
    /// `-q/--quiet`, `-f/--force`, `--no-input`, `--no-color`,
    /// `--plain`, `--json`, `--color <when>`. `--plain` and `--json` are validated as mutually exclusive.
    ///
    /// Because these are app-wide conventions, they are registered as persistent flags and
    /// propagated to all subcommands, so both `list --plain` and `--plain list` work.
    pub fn standard_flags(mut self) -> Self {
        self = self.persistent_flag(
            Flag::new(
                "quiet",
                FlagValue::Bool(false),
                "suppress non-essential output",
            )
            .short('q'),
        );
        self = self.persistent_flag(
            Flag::new("force", FlagValue::Bool(false), "skip confirmation prompts").short('f'),
        );
        self = self.persistent_flag(Flag::new(
            "no-input",
            FlagValue::Bool(false),
            "never prompt; fail if input is required",
        ));
        self = self.persistent_flag(Flag::new(
            "no-color",
            FlagValue::Bool(false),
            "disable colored output",
        ));
        self = self.persistent_flag(Flag::new(
            "plain",
            FlagValue::Bool(false),
            "output plain machine-readable records (one per line)",
        ));
        self = self.persistent_flag(Flag::new("json", FlagValue::Bool(false), "output JSON"));
        self = self.persistent_flag(Flag::new(
            "color",
            FlagValue::String("auto".to_owned()),
            "when to use color: auto, always, never",
        ));
        self = self.persistent_flag(Flag::new(
            "confirm",
            FlagValue::String(String::new()),
            "confirm a dangerous action by name (for scripts)",
        ));
        self = self.mutually_exclusive(&["plain", "json"]);
        self
    }

    /// Add a subcommand.
    ///
    /// # Panics
    /// Panics if the name or an alias conflicts with an already registered subcommand.
    pub fn subcommand(mut self, cmd: Command) -> Self {
        for existing in &self.subcommands {
            let names = std::iter::once(&cmd.name).chain(cmd.aliases.iter());
            for n in names {
                assert!(
                    existing.name != *n && !existing.aliases.contains(n),
                    "wrcli: subcommand name/alias \"{n}\" conflicts with existing subcommand \"{}\"",
                    existing.name
                );
            }
        }
        self.subcommands.push(cmd);
        self
    }

    /// Set the positional argument validator. See the built-in functions in the [`args`] module.
    pub fn args(mut self, validator: args::ArgValidator) -> Self {
        self.arg_validator = Some(validator);
        self
    }

    /// Register a function that provides dynamic completion candidates for positional arguments.
    ///
    /// Receives the list of positional arguments entered so far and returns candidates.
    pub fn arg_candidates<F>(mut self, f: F) -> Self
    where
        F: Fn(&[String]) -> Vec<String> + Send + Sync + 'static,
    {
        self.arg_candidates = Some(Box::new(f));
        self
    }

    /// Attach a configuration store (Viper equivalent) to the command tree.
    pub fn with_config(mut self, config: Config) -> Self {
        self.config = Some(config);
        self
    }

    // ── Lifecycle callbacks ─────────────────────────────────────────────────────

    /// Called before running every command, in root-to-leaf order.
    pub fn on_persistent_pre_run<F>(mut self, f: F) -> Self
    where
        F: for<'ctx> Fn(&CommandContext<'ctx>) + Send + Sync + 'static,
    {
        self.persistent_pre_run = Some(Box::new(f));
        self
    }

    /// Called only immediately before the matched leaf command's `on_run`.
    pub fn on_pre_run<F>(mut self, f: F) -> Self
    where
        F: for<'ctx> Fn(&CommandContext<'ctx>) + Send + Sync + 'static,
    {
        self.pre_run = Some(Box::new(f));
        self
    }

    /// The run handler for this command.
    pub fn on_run<F>(mut self, f: F) -> Self
    where
        F: for<'ctx> Fn(&CommandContext<'ctx>) + Send + Sync + 'static,
    {
        self.run = Some(Box::new(f));
        self
    }

    /// The run handler for this command. Returning `Err` aborts the post-run hooks.
    pub fn on_run_e<F>(mut self, f: F) -> Self
    where
        F: for<'ctx> Fn(&CommandContext<'ctx>) -> Result<()> + Send + Sync + 'static,
    {
        self.run_e = Some(Box::new(f));
        self
    }

    /// Called only immediately after the matched leaf command's `on_run`.
    pub fn on_post_run<F>(mut self, f: F) -> Self
    where
        F: for<'ctx> Fn(&CommandContext<'ctx>) + Send + Sync + 'static,
    {
        self.post_run = Some(Box::new(f));
        self
    }

    /// Called after running every command, in leaf-to-root order.
    pub fn on_persistent_post_run<F>(mut self, f: F) -> Self
    where
        F: for<'ctx> Fn(&CommandContext<'ctx>) + Send + Sync + 'static,
    {
        self.persistent_post_run = Some(Box::new(f));
        self
    }
}

/// A group of flag constraints validated before running the leaf command.
pub(crate) enum FlagGroup {
    MutuallyExclusive(Vec<String>),
    RequiredTogether(Vec<String>),
    OneRequired(Vec<String>),
}

/// Convert `&[&str]` into an owned `Vec<String>`.
fn names(flags: &[&str]) -> Vec<String> {
    flags.iter().map(|f| (*f).to_owned()).collect()
}

/// Type of the dynamic completion candidate generator for positional arguments.
pub(crate) type ArgCandidatesFn = Box<dyn Fn(&[String]) -> Vec<String> + Send + Sync>;

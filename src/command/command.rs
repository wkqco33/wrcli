use super::args;
use crate::command::context::CommandContext;
use crate::config::Config;
use crate::error::{Result, WrCliError};
use crate::flag::{Flag, FlagSet, FlagValue};

/// 인자 없이 실행되는 콜백 타입.
pub type RunFn = Box<dyn for<'ctx> Fn(&CommandContext<'ctx>) + Send + Sync>;
/// 에러를 반환할 수 있는 콜백 타입. Err 반환 시 라이프사이클 체인 중단.
pub type RunEFn = Box<dyn for<'ctx> Fn(&CommandContext<'ctx>) -> Result<()> + Send + Sync>;

/// CLI 트리의 커맨드 노드.
///
/// 플루언트 빌더 API로 구성 후 루트 커맨드에서 [`crate::Command::execute`] 호출.
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
    /// `--help`의 usage 줄에 표시할 포지셔널 힌트 (예: `"<name>"`).
    pub(crate) usage_args: Option<String>,
    /// help에서 Usage 다음에 표시되는 예제 목록 (clig.dev: 예제를 앞에 둠).
    pub(crate) examples: Vec<String>,
    /// help 하단에 표시할 이슈/피드백 URL.
    pub(crate) support_url: Option<String>,
    /// help 하단에 표시할 문서 URL. `{command}`가 전체 커맨드 경로로 치환됨.
    pub(crate) docs_url: Option<String>,
    /// 예기치 못한 오류 시 `execute_or_exit`이 안내할 버그 리포트 URL.
    pub(crate) bug_report_url: Option<String>,
    /// 러너가 없을 때 도움말을 출력하고 성공으로 종료.
    pub(crate) help_on_missing_runner: bool,
    /// Ctrl-C 수신 시 출력할 메시지 (`signal` 피처).
    #[cfg(feature = "signal")]
    pub(crate) interrupt_message: Option<&'static str>,

    pub(crate) flags: FlagSet,
    pub(crate) subcommands: Vec<Command>,

    pub(crate) arg_validator: Option<args::ArgValidator>,
    /// 포지셔널 인수 동적 completion 후보 생성기.
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

    /// 루트 커맨드만 Config를 보유. 실행 중 참조로 전달됨.
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

    /// 부모 커맨드 목록에 표시되는 한 줄 설명.
    pub fn short(mut self, s: &str) -> Self {
        self.short = s.to_owned();
        self
    }

    /// 이 커맨드 자체 `--help`에 표시되는 긴 설명.
    pub fn long(mut self, s: &str) -> Self {
        self.long = s.to_owned();
        self
    }

    /// 버전 문자열. `--version` / `-V` 플래그를 활성화.
    pub fn version(mut self, v: &str) -> Self {
        self.version = Some(v.to_owned());
        self
    }

    /// 커맨드 별칭 추가.
    pub fn alias(mut self, a: &str) -> Self {
        self.aliases.push(a.to_owned());
        self
    }

    /// help와 completion 목록에서 이 커맨드를 숨김 (실행은 계속 가능).
    pub fn hidden(mut self) -> Self {
        self.hidden = true;
        self
    }

    /// deprecated 커맨드로 표시. 실행 시 stderr에 경고를 출력.
    pub fn deprecated(mut self, msg: &str) -> Self {
        self.deprecated = Some(msg.to_owned());
        self
    }

    /// 오타 입력 시 이 커맨드를 제안할 별칭 추가 (Cobra의 `SuggestFor`).
    ///
    /// 실제 별칭과 달리 실행되지 않고, `Did you mean` 후보로만 사용된다.
    pub fn suggest_for(mut self, name: &str) -> Self {
        self.suggest_for.push(name.to_owned());
        self
    }

    /// `--help`의 usage 줄에 표시할 포지셔널 힌트 (예: `"<name>"`, `"SRC DST"`).
    pub fn usage_args(mut self, hint: &str) -> Self {
        self.usage_args = Some(hint.to_owned());
        self
    }

    /// help의 Usage 바로 다음에 표시할 예제를 추가한다 (여러 번 호출 가능).
    pub fn example(mut self, e: &str) -> Self {
        self.examples.push(e.to_owned());
        self
    }

    /// help 하단에 표시할 이슈/피드백 URL.
    pub fn support_url(mut self, url: &str) -> Self {
        self.support_url = Some(url.to_owned());
        self
    }

    /// help 하단에 표시할 문서 URL. `{command}`는 전체 커맨드 경로로 치환된다.
    pub fn docs_url(mut self, url: &str) -> Self {
        self.docs_url = Some(url.to_owned());
        self
    }

    /// 예기치 못한 오류 시 [`Command::execute_or_exit`]이 안내할 버그 리포트 URL.
    pub fn bug_report_url(mut self, url: &str) -> Self {
        self.bug_report_url = Some(url.to_owned());
        self
    }

    /// 러너가 없어도 도움말을 출력하고 성공(종료 코드 0)으로 끝낸다.
    ///
    /// 서브커맨드를 가진 상위 커맨드는 이 설정 없이도 자동으로 이 동작을 한다.
    pub fn help_on_missing_runner(mut self) -> Self {
        self.help_on_missing_runner = true;
        self
    }

    /// Ctrl-C(SIGINT) 수신 시 메시지를 즉시 출력하고 종료 코드 130으로 끝낸다.
    ///
    /// `signal` 피처가 필요하다.
    #[cfg(feature = "signal")]
    pub fn interrupt_message(mut self, message: &'static str) -> Self {
        self.interrupt_message = Some(message);
        self
    }

    /// 등록된 예제 목록.
    pub fn examples(&self) -> &[String] {
        &self.examples
    }

    /// 이름 또는 별칭으로 서브커맨드 조회.
    pub(crate) fn find_subcommand(&self, name: &str) -> Option<&Command> {
        self.subcommands
            .iter()
            .find(|c| c.name == name || c.aliases.iter().any(|a| a == name))
    }

    /// 이름/별칭이 `name`인 서브커맨드가 등록되어 있는지.
    pub(crate) fn has_subcommand_named(&self, name: &str) -> bool {
        self.find_subcommand(name).is_some()
    }

    /// `name`에 대한 편집 거리 기반 서브커맨드 제안 (hidden 제외, `suggest_for` 포함).
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

    /// 미등록 서브커맨드 오류 생성 (제안 포함).
    pub(crate) fn unknown_subcommand_error(&self, name: &str, parent: &str) -> WrCliError {
        WrCliError::UnknownSubcommand {
            name: name.to_owned(),
            parent: parent.to_owned(),
            suggestions: self.subcommand_suggestions(name),
        }
    }

    /// 이 그룹의 플래그 중 둘 이상을 함께 지정하면 오류.
    pub fn mutually_exclusive(mut self, flags: &[&str]) -> Self {
        self.mutually_exclusive.push(names(flags));
        self
    }

    /// 이 그룹의 플래그는 일부만 지정하면 오류 (전부 또는 전무).
    pub fn required_together(mut self, flags: &[&str]) -> Self {
        self.required_together.push(names(flags));
        self
    }

    /// 이 그룹에서 최소 하나의 플래그를 지정해야 함.
    pub fn one_required(mut self, flags: &[&str]) -> Self {
        self.one_required.push(names(flags));
        self
    }

    /// 로컬 플래그 추가 (하위 커맨드에 전파되지 않음).
    pub fn flag(mut self, flag: Flag) -> Self {
        self.flags.add(flag);
        self
    }

    /// persistent 플래그 추가 (모든 하위 커맨드에 자동 전파).
    pub fn persistent_flag(mut self, mut flag: Flag) -> Self {
        flag.persistent = true;
        self.flags.add(flag);
        self
    }

    /// clig.dev 표준 플래그 묶음을 등록한다.
    ///
    /// `-q/--quiet`, `-f/--force`, `--no-input`, `--no-color`,
    /// `--plain`, `--json`, `--color <when>`. `--plain`과 `--json`은 상호 배타로 검증된다.
    ///
    /// 앱 전역 관례에 해당하므로 persistent 플래그로 등록되어 모든 서브커맨드에 전파되고,
    /// `list --plain`과 `--plain list`가 모두 동작한다.
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

    /// 서브커맨드 추가.
    ///
    /// # Panics
    /// 이름 또는 별칭이 이미 등록된 서브커맨드와 충돌하면 패닉.
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

    /// 위치 인자 검증기 설정. [`args`] 모듈의 내장 함수 참조.
    pub fn args(mut self, validator: args::ArgValidator) -> Self {
        self.arg_validator = Some(validator);
        self
    }

    /// 포지셔널 인수 동적 completion 후보를 제공하는 함수 등록.
    ///
    /// 이미 입력된 포지셔널 인수 목록을 받아 후보를 반환한다.
    pub fn arg_candidates<F>(mut self, f: F) -> Self
    where
        F: Fn(&[String]) -> Vec<String> + Send + Sync + 'static,
    {
        self.arg_candidates = Some(Box::new(f));
        self
    }

    /// 설정 저장소(Viper 등가물)를 커맨드 트리에 연결.
    pub fn with_config(mut self, config: Config) -> Self {
        self.config = Some(config);
        self
    }

    // ── 라이프사이클 콜백 ─────────────────────────────────────────────────────

    /// 루트→리프 순서로 모든 커맨드 실행 전 호출.
    pub fn on_persistent_pre_run<F>(mut self, f: F) -> Self
    where
        F: for<'ctx> Fn(&CommandContext<'ctx>) + Send + Sync + 'static,
    {
        self.persistent_pre_run = Some(Box::new(f));
        self
    }

    /// 매칭된 리프 커맨드의 `on_run` 직전에만 호출.
    pub fn on_pre_run<F>(mut self, f: F) -> Self
    where
        F: for<'ctx> Fn(&CommandContext<'ctx>) + Send + Sync + 'static,
    {
        self.pre_run = Some(Box::new(f));
        self
    }

    /// 이 커맨드의 실행 핸들러.
    pub fn on_run<F>(mut self, f: F) -> Self
    where
        F: for<'ctx> Fn(&CommandContext<'ctx>) + Send + Sync + 'static,
    {
        self.run = Some(Box::new(f));
        self
    }

    /// 이 커맨드의 실행 핸들러. `Err` 반환 시 post-run 훅 중단.
    pub fn on_run_e<F>(mut self, f: F) -> Self
    where
        F: for<'ctx> Fn(&CommandContext<'ctx>) -> Result<()> + Send + Sync + 'static,
    {
        self.run_e = Some(Box::new(f));
        self
    }

    /// 매칭된 리프 커맨드의 `on_run` 직후에만 호출.
    pub fn on_post_run<F>(mut self, f: F) -> Self
    where
        F: for<'ctx> Fn(&CommandContext<'ctx>) + Send + Sync + 'static,
    {
        self.post_run = Some(Box::new(f));
        self
    }

    /// 리프→루트 순서로 모든 커맨드 실행 후 호출.
    pub fn on_persistent_post_run<F>(mut self, f: F) -> Self
    where
        F: for<'ctx> Fn(&CommandContext<'ctx>) + Send + Sync + 'static,
    {
        self.persistent_post_run = Some(Box::new(f));
        self
    }
}

/// 리프 커맨드 실행 전에 검증할 플래그 제약 그룹.
pub(crate) enum FlagGroup {
    MutuallyExclusive(Vec<String>),
    RequiredTogether(Vec<String>),
    OneRequired(Vec<String>),
}

/// `&[&str]`을 소유 `Vec<String>`으로 변환.
fn names(flags: &[&str]) -> Vec<String> {
    flags.iter().map(|f| (*f).to_owned()).collect()
}

/// 포지셔널 인수 동적 completion 후보 생성기 타입.
pub(crate) type ArgCandidatesFn = Box<dyn Fn(&[String]) -> Vec<String> + Send + Sync>;

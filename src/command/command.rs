use super::args;
use crate::command::context::CommandContext;
use crate::config::Config;
use crate::error::Result;
use crate::flag::{Flag, FlagSet};

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

    pub(crate) flags: FlagSet,
    pub(crate) subcommands: Vec<Command>,

    pub(crate) arg_validator: Option<args::ArgValidator>,

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
            flags,
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

    /// 이 커맨드의 인폴러블 실행 핸들러.
    pub fn on_run<F>(mut self, f: F) -> Self
    where
        F: for<'ctx> Fn(&CommandContext<'ctx>) + Send + Sync + 'static,
    {
        self.run = Some(Box::new(f));
        self
    }

    /// 이 커맨드의 폴러블 실행 핸들러. `Err` 반환 시 post-run 훅 중단.
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

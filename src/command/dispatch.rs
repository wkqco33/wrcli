use crate::config::Config;
use crate::error::{Result, WrCliError};
use super::command::{Command, RunFn};
use super::context::CommandContext;
use super::help;

impl Command {
    /// 진입점: `std::env::args()`(argv[0] 제외)를 파싱하고 실행.
    pub fn execute(self) -> Result<()> {
        let args: Vec<String> = std::env::args().skip(1).collect();
        self.execute_with(args)
    }

    /// 테스트용 변형: 주어진 인자 목록을 파싱하고 실행.
    pub fn execute_with(mut self, args: Vec<String>) -> Result<()> {
        let mut config = self.config.take().unwrap_or_default();
        let mut pre_chain: Vec<RunFn> = Vec::new();
        let mut post_chain: Vec<RunFn> = Vec::new();
        let mut command_path: Vec<String> = Vec::new();
        self.dispatch(args, &mut config, &mut pre_chain, &mut post_chain, &mut command_path)
    }

    pub(super) fn dispatch(
        mut self,
        mut args: Vec<String>,
        config: &mut Config,
        pre_chain: &mut Vec<RunFn>,
        post_chain: &mut Vec<RunFn>,
        command_path: &mut Vec<String>,
    ) -> Result<()> {
        command_path.push(self.name.clone());

        if let Some(f) = self.persistent_pre_run.take() {
            pre_chain.push(f);
        }
        if let Some(f) = self.persistent_post_run.take() {
            // post-run은 리프→루트 순서이므로 앞에 삽입
            post_chain.insert(0, f);
        }

        // 서브커맨드 라우팅을 먼저 시도해야 `app serve --help`가 serve의 help를 출력함.
        // 첫 번째 비-플래그 토큰이 서브커맨드 후보.
        let candidate = args.iter().enumerate().find(|(_, a)| !a.starts_with('-'));
        let subcommand_pos = candidate.and_then(|(idx, name)| {
            self.subcommands
                .iter()
                .position(|c| &c.name == name || c.aliases.iter().any(|a| a == name))
                .map(|pos| (idx, pos))
        });

        if let Some((arg_idx, cmd_pos)) = subcommand_pos {
            args.remove(arg_idx);
            let mut child = self.subcommands.remove(cmd_pos);
            log::debug!(
                "서브커맨드 라우팅: {} -> {}",
                command_path.join(" "),
                child.name
            );
            // persistent 플래그를 하위로 전파 — 이미 있는 경우엔 클론 없이 건너뜀
            for flag in self.flags.persistent_flags() {
                child.flags.add_if_absent(flag);
            }
            return child.dispatch(args, config, pre_chain, post_chain, command_path);
        }

        // 서브커맨드 없음 — 이 커맨드의 메타 플래그 처리
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

        // 등록된 서브커맨드가 있는데 인식 불가 토큰이 오면 명확한 에러 반환
        if !self.subcommands.is_empty() {
            if let Some(unknown) = args.iter().find(|a| !a.starts_with('-')) {
                return Err(WrCliError::UnknownSubcommand {
                    name: unknown.clone(),
                    parent: self.name.clone(),
                });
            }
        }

        // ── 리프 커맨드 ──────────────────────────────────────────────────────

        let positional = self.flags.parse(args)?;

        if let Some(ref validator) = self.arg_validator {
            validator(&positional)?;
        }

        // 사용자가 명시적으로 입력한 플래그만 Config 레이어 4로 바인딩
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

        log::trace!("라이프사이클 훅 실행 시작: {}", command_path.join(" "));

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

        log::trace!("라이프사이클 훅 실행 완료: {}", command_path.join(" "));

        Ok(())
    }
}

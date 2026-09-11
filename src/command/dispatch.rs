use super::command::{Command, FlagGroup, RunFn};
use super::context::CommandContext;
use super::help;
use crate::config::Config;
use crate::error::{Result, WrCliError};
use crate::flag::{FlagSet, FlagValue};

/// 값을 요구하는(bool이 아닌) 플래그인지 여부.
pub(crate) fn takes_value(default: &FlagValue) -> bool {
    !matches!(default, FlagValue::Bool(_))
}

/// 수집된 플래그 제약 그룹을 리프 커맨드의 실제 입력에 대해 검증.
///
/// 그룹의 플래그 중 리프 FlagSet에 등록된 것이 하나도 없으면(예: 부모 로컬 플래그
/// 제약이 서브커맨드 실행 중 수집된 경우) 검증을 건너뛴다.
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

/// 서브커맨드 후보 또는 미인식 위치 인자로 쓰일 수 있는 첫 토큰의 인덱스를 찾는다.
///
/// `flags.parse()`를 실제로 호출하지 않고도 값을 소비하는 플래그(`--name value`,
/// `-c value`)의 값 토큰을 건너뛰어, 그 값이 우연히 서브커맨드 이름과 같아도
/// 서브커맨드로 오인하지 않도록 한다. `--` sentinel을 만나면 그 이후는 전부
/// 리터럴 위치 인자이므로 후보 탐색을 중단한다.
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
    /// 진입점: `std::env::args()`(`argv[0]` 제외)를 파싱하고 실행.
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
        let mut groups: Vec<FlagGroup> = Vec::new();
        self.dispatch(
            args,
            &mut config,
            &mut pre_chain,
            &mut post_chain,
            &mut command_path,
            &mut groups,
        )
    }

    /// 실행 후 오류가 나면 stderr에 출력하고 프로세스를 종료.
    ///
    /// 성공하면 종료 코드 0, 사용법 오류는 2, 그 외는 1로 종료한다.
    /// 테스트에서 호출하면 테스트 프로세스가 종료되므로 `execute()`를 사용할 것.
    pub fn execute_or_exit(self) -> ! {
        match self.execute() {
            Ok(()) => std::process::exit(0),
            Err(e) => {
                eprintln!("Error: {}", e);
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
            // push 후 역방향 순회로 리프→루트 순서 유지 (insert(0) 대비 O(1))
            post_chain.push(f);
        }

        // 서브커맨드 라우팅을 먼저 시도해야 `app serve --help`가 serve의 help를 출력함.
        // 값을 소비하는 플래그의 값 토큰은 건너뛰고 첫 번째 진짜 위치 토큰을 후보로 삼는다.
        let candidate = find_positional_candidate(&args, &self.flags);
        let subcommand_pos = candidate.and_then(|idx| {
            let name = args[idx].as_str();
            self.subcommands
                .iter()
                .position(|c| c.name == name || c.aliases.iter().any(|a| a == name))
                .map(|pos| (idx, pos))
        });

        // 메타 플래그는 위치와 무관하게 전체를 스캔해야 `app unknown-sub --help`에서도
        // help가 우선한다. 서브커맨드 앞에 오는 부모 로컬 플래그도 이 경우엔 소비하지 않고
        // 그대로 리프로 넘겨 리프가 help/version을 처리하게 한다.
        let found_help = args.iter().any(|a| a == "--help" || a == "-h");
        let found_version = args.iter().any(|a| a == "--version" || a == "-V");
        let meta = found_help || found_version;

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
                child.flags.add_inherited(flag);
            }

            if meta {
                return child.dispatch(args, config, pre_chain, post_chain, command_path, groups);
            }

            // 서브커맨드 앞의 부모 플래그(로컬 포함)를 부모 FlagSet으로 소비하고,
            // 그 값을 하위로 넘겨 리프 컨텍스트에서도 읽을 수 있게 한다.
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
                &self.name,
                &self.short,
                &self.long,
                &self.version,
                self.usage_args.as_deref(),
                &self.flags,
                &self.subcommands,
                command_path,
            );
            return Ok(());
        }

        if found_version && self.version.is_some() {
            println!("{} {}", self.name, self.version.as_deref().unwrap_or(""));
            return Ok(());
        }

        // 등록된 서브커맨드가 있는데 인식 불가 토큰이 오면 명확한 에러 반환
        if !self.subcommands.is_empty()
            && let Some(idx) = candidate
        {
            let name = args[idx].clone();
            log::warn!("unknown subcommand '{}' for '{}'", name, self.name);
            let mut suggestions = crate::suggest::closest(
                &name,
                self.subcommands.iter().filter(|c| !c.hidden).flat_map(|c| {
                    std::iter::once(c.name.as_str()).chain(c.aliases.iter().map(String::as_str))
                }),
            );
            for cmd in self.subcommands.iter().filter(|c| !c.hidden) {
                let explicit = cmd
                    .suggest_for
                    .iter()
                    .any(|s| s.eq_ignore_ascii_case(&name));
                if explicit && !suggestions.contains(&cmd.name) {
                    suggestions.push(cmd.name.clone());
                }
            }
            return Err(WrCliError::UnknownSubcommand {
                name,
                parent: self.name.clone(),
                suggestions,
            });
        }

        // ── 리프 커맨드 ──────────────────────────────────────────────────────

        let positional = self.flags.parse(args)?;
        validate_flag_groups(groups, &self.flags)?;

        if let Some(ref validator) = self.arg_validator {
            validator(&positional)?;
        }

        // 명시적으로 설정되지 않은 플래그는 설정 저장소의 값으로 시드 (config → flag).
        self.flags.seed_from_config(config);

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
            help::print_help(
                &self.name,
                &self.short,
                &self.long,
                &self.version,
                self.usage_args.as_deref(),
                &self.flags,
                &self.subcommands,
                command_path,
            );
            return Err(WrCliError::CommandHasNoRunner(self.name.clone()));
        }

        if let Some(ref f) = self.post_run {
            f(&ctx);
        }
        for f in post_chain.iter().rev() {
            f(&ctx);
        }

        log::trace!("라이프사이클 훅 실행 완료: {}", command_path.join(" "));

        Ok(())
    }
}

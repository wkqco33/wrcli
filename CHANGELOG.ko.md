# 변경 이력

[English](CHANGELOG.md) | [한국어](CHANGELOG.ko.md)

이 프로젝트의 주요 변경 사항을 기록합니다.
형식은 [Keep a Changelog](https://keepachangelog.com/ko/1.1.0/)를 따릅니다.

## [Unreleased]

### Added

- **새로운 스타일 컴포넌트 추가**:
  - `BoxStyle` 열거형 (`Square`, `Rounded`, `Double`, `Heavy`, `Ascii`, `Markdown` 테두리 문자 세트).
  - `KeyVal`: 구분자 및 스타일 지정이 가능한 자동 정렬 키-값 뷰어.
  - `Badge`: 주요 상태 프리셋(`success`, `error`, `warn`, `info`) 및 괄호 설정 가능한 상태 태그/뱃지.
  - `List`: 중첩을 지원하는 글머리 기호(`•`, `-`, `→`) 및 번호 매기기 목록 컴포넌트.
  - `Spinner`: clig.dev 규약을 준수하는 비블로킹 터미널 스피너 인디케이터.
  - `Text::from_markup`: rich 스타일의 인라인 마크업 태그 파서 (예: `[bold green]...[/]`).
- **기존 컴포넌트 옵션 확장**:
  - `Table`: `.box_style(BoxStyle)`, `.row_separator(bool)`, `.border_style(Style)` 추가.
  - `Panel`: `.box_style(BoxStyle)`, `.content_align(Align)`, `.subtitle(&str)`, `.subtitle_style(Style)`, `.subtitle_align(Align)` 추가.
  - `Rule`: 좌측/중앙/우측 정렬을 위한 `.align(Align)` 추가.
  - `Tree`: `.guide_style(Style)` 및 멀티라인 라벨 들여쓰기 지원 추가.

### Fixed

- `display_width`: 가시 터미널 너비 계산 시 ANSI 이스케이프 시퀀스를 제외하고 CJK 문자 및 이모지 폭(2칸)을 올바르게 반영하도록 개선하여 서식 텍스트 또는 한영/이모지 혼용 시 `Table`, `Panel`, `Rule` 테두리가 어긋나는 현상 해결.
- `Table::border(false)`: 테두리 비활성화 시 열 사이에 세로선(`│`)이 잘못 출력되고 하이픈 구분선과 열 너비가 어긋나던 버그 수정.
- `Tree`: 가지선과 들여쓰기 라인이 루트 노드의 라벨 스타일로 강제 적용되던 문제 수정.

### Changed

- 문서를 영문 정본으로 전환했습니다. `README.md`, `docs/GUIDE.md`, `docs/STYLE.md`,
  `CHANGELOG.md`가 영문 정본이며, 한국어판은 `README.ko.md`, `docs/GUIDE.ko.md`,
  `docs/STYLE.ko.md`, `CHANGELOG.ko.md`로 나란히 유지합니다.
- 소스 주석(rustdoc·인라인)과 로그 메시지를 모두 영문으로 바꿨습니다.
  테스트에 남아 있는 한국어 문자열은 CJK 표시폭 테스트용 픽스처입니다.


## [0.4.0] - 2026-09-12

clig.dev(Command Line Interface Guidelines) 대응 기능을 추가했습니다.

### Added

- **내장 `help` 서브커맨드**: `app help`, `app help sub`, `app help sub subsub`.
  사용자가 `help`를 직접 등록하면 비활성화된다. `help`/`--help`/`--version`이 completion
  후보에 포함된다.
- **help 예제·지원·문서 링크**: `Command::example`, `Command::support_url`,
  `Command::docs_url`(`{command}` 치환 및 하위 상속). 예제 섹션은 Usage 바로 다음에
  출력된다.
- **러너 없는 커맨드 정책**: 서브커맨드만 가진 상위 커맨드를 인자 없이 실행하면
  도움말을 출력하고 종료 코드 0으로 끝난다. `Command::help_on_missing_runner()`로
  리프에서도 같은 동작을 opt-in 할 수 있다.
- **`Command::bug_report_url`**: `execute_or_exit()`이 예상 밖 오류에서 리포트 URL을
  안내한다.
- **표준 플래그** `Command::standard_flags()`: `-q/--quiet`, `-f/--force`,
  `--no-input`, `--no-color`, `--plain`, `--json`, `--color <when>`, `--confirm <name>`.
  persistent로 등록되어 서브커맨드에 전파되며 `--plain`과 `--json`은 상호 배타다.
- **출력 포맷**: `OutputFormat`(Human/Plain/Json)과 `CommandContext::output_format`,
  `is_quiet`, `is_force`, `no_input`, `is_plain`, `is_json`, `Table::render_plain()`.
- **대화형 입력**: `CommandContext::confirm`, `confirm_severe`, `prompt_password`,
  `is_interactive`. TTY가 아니거나 `--no-input`이면 `InteractiveInputRequired`를 반환한다.
  `confirm_severe` 불일치는 `ConfirmationFailed`.
- **민감 플래그** `Flag::sensitive()`: 도움말 기본값과 오류 메시지에서 값을 `***`로 가린다.
- **선택적 값 플래그** `Flag::optional_value()`: 특수 단어 `none`을 값 없음(빈 문자열)으로
  해석한다.
- **`wrcli::io`**: `open_reader`, `open_writer`, `read_to_string` — `-`를 stdin/stdout으로
  처리한다.
- **색상 정책**: `ColorChoice`, `set_color_choice`/`color_choice`/`reset_color_choice`,
  `set_no_color_env`, `ColorEnv`/`should_use_color`, `stdout_is_terminal`,
  `stdin_is_terminal`. `FORCE_COLOR`, `TERM=dumb`, 앱 전용 `*_NO_COLOR`,
  `--no-color`/`--color=<when>`을 지원한다.
- **페이저** `style::pager::page`: stdout이 TTY일 때만 `PAGER`(기본 `less -FIRX`)로 보낸다.
- **`Progress::draw()` / `Progress::finish()`**: 비TTY에서는 애니메이션 없이 한 줄만 출력.
- **`signal` 피처**: `Command::interrupt_message`와 `wrcli::signal` — Ctrl-C 시 메시지를
  출력하고 종료 코드 130으로 즉시 종료한다.

### Changed

- `style::print_warning`/`print_info`가 stdout 대신 **stderr**로 출력된다
  (clig.dev: messaging to stderr).
- `WrCliError::InvalidFlagValue` 메시지에 `Run with --help for usage.` 힌트를 추가했다.
- `NO_COLOR`은 **비어 있지 않을 때만** 색상을 끈다(스펙 준수). `TERM=dumb` 추가.
- 러너 없는 리프 커맨드는 도움말을 stdout에 출력하지 않고 오류만 낸다(서브커맨드를
  가진 상위 커맨드는 help + 성공).

### Fixed

- `myapp help <unknown>`이 제안(`Did you mean`)이 포함된 `UnknownSubcommand`를 낸다.

## [0.3.0] - 2026-09-12

### Added

- **오타 제안**: 미등록 서브커맨드/플래그에 편집 거리 기반 `Did you mean` 후보를
  에러 메시지에 포함 (`src/suggest.rs`).
- **숨김**: `Command::hidden()`, `Flag::hidden()` — help/completion/제안에서 제외,
  파싱·실행은 유지.
- **Deprecated**: `Command::deprecated(msg)`, `Flag::deprecated(msg)` — 사용 시 stderr 경고.
- **플래그 제약 그룹**: `Command::mutually_exclusive`, `required_together`,
  `one_required`와 전용 에러 3종.
- **종료 코드**: `WrCliError::is_usage_error()`, `WrCliError::exit_code()`
  (사용법 오류 2, 그 외 1), `Command::execute_or_exit()`.
- **CommandContext getter 확장**: `get_uint`, `get_int_vec`, `get_duration`,
  `get_time`, `get_size_in_bytes`, `get_string_map`, `is_set`.
- `FlagSet::get_uint`.
- **동적 completion**: `Command::complete`, `Command::completion_request`(`__complete`
  프로토콜), `Command::arg_candidates`.
- **CSV 슬라이스 플래그**: `Flag::comma_separated()` — `--tag a,b,c`를 여러 값으로 분리.
- **부모 로컬 플래그**: `app --profile prod deploy`처럼 서브커맨드 이름 앞의 부모
  로컬 플래그를 부모가 소비하고 값을 리프 컨텍스트로 전달.
- **usage 힌트**: `Command::usage_args("<name>")` — `--help`의 usage 줄에 포지셔널 힌트.
- **`Command::suggest_for`**: 실행되지 않는 제안 전용 별칭 (Cobra `SuggestFor`).
- `FlagSet::parse_partial`(부모 플래그 소비용, required 검증 생략), `FlagSet::inherit_values`.

### Changed

- help 출력이 상속된 persistent 플래그를 `Global Flags:` 섹션으로 분리 (Cobra 스타일).
- help 플래그 행에 deprecated 플래그를 `(deprecated)`로 표시.
- 서브커맨드 라우팅 시 부모 플래그를 부모 FlagSet으로 파싱하고, 그 값을 하위로
  전달(`inherit_values`)해 `ctx.flags`에서 읽을 수 있게 함. `--help`/`--version`이
  포함되면 기존처럼 리프가 처리.
- `FlagSet::is_set`가 argv로 명시된 플래그만 `true`로 반환. 설정에서 시드된 값은
  이제 `false`이며, Config 레이어 바인딩도 명시 입력에만 적용.
- `WrCliError::UnknownFlag` / `UnknownSubcommand`에 `suggestions` 필드 추가.

### Fixed

- `no-default-features` 빌드에서 `config/writer.rs`의 미사용 import 경고 제거.

## [0.2.0] - 2026-09-12

### Added — Config (Viper 패리티)

- 5계층 우선순위: 기본값 → 설정 파일 → 환경변수 → CLI 플래그 → `set` 명시 값.
- `set`, `is_set`, `register_alias`, `set_key_delimiter`, `set_env_key_replacer`,
  `allow_empty_env`.
- 타입 getter: `get_int64`, `get_uint`, `get_duration`, `get_time`,
  `get_size_in_bytes`, `get_string_slice`.
- 열거·하위 트리: `all_keys`, `all_settings`, `SettingsMap`/`SettingsEntry`,
  `get_string_map`/`get_string_map_string`/`get_string_map_string_slice`, `sub`.
- 런타임 읽기·병합: `read_config`, `merge_in_config`, `merge_config_map`.
- 쓰기: `write_config_as`, `safe_write_config_as`.
- 신규 포맷(피처): `ini-config`, `dotenv-config`, `properties-config` (읽기+쓰기).
- 구조체 역직렬화(피처 `serde`): `unmarshal`, `unmarshal_key`.
- 설정 파일 감시: `on_config_change`, `set_watch_interval`, `watch_config`,
  `ConfigWatcher` (폴링 기반, 기본 1초).
- 신규 에러: `ConfigTypeNotSet`, `ConfigFileExists`, `ConfigDeserializeError`,
  `ConfigWatchNotReady`.

### Fixed

- bash/zsh completion 스크립트 생성 오류 (stray quote, `#compdef` 중복,
  `_arguments` continuation 누락, fish short 플래그 누락).
- `Panel`이 `padding != 1`일 때 제목 테두리 폭이 본문과 어긋나던 문제.
- `--help` 컬럼 정렬/줄바꿈이 CJK·ANSI 출력에서 깨지던 문제 (`display_width` 적용).

### Changed

- Config getter 6종에 중복되던 4계층 조회 로직을 `resolve()`로 통합.
- 설정 탐색 경로 재계산, 플래그 시드, `Style::apply`의 실행당 할당 감소.
- 문서: 가이드를 `docs/`로 통합하고 `README.md`/`docs/GUIDE.md`/`AGENTS.md`를 갱신,
  `CHANGELOG.md` 추가.

## [0.1.0]

- 최초 릴리스: 커맨드 트리, 타입 플래그, 4계층 설정, completion, 터미널 스타일링.

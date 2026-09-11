# 변경 이력

이 프로젝트의 주요 변경 사항을 기록합니다.
형식은 [Keep a Changelog](https://keepachangelog.com/ko/1.1.0/)를 따릅니다.

## [Unreleased]

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

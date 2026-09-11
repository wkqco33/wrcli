# AGENTS.md

Rust CLI 프레임워크 라이브러리(cobra/viper에서 영감을 받음). 이 문서는 이 저장소에서
작업하는 AI 에이전트(그리고 사람)를 위한 안내서이며, 코드를 작성하기 전에 반드시 읽어야 합니다.

## 현재 기능 범위

현재 공개 API가 제공하는 기능은 다음과 같습니다.

- 커맨드 트리, 알리아스, persistent 플래그, 라이프사이클 훅, `--help`/`--version`.
- 타입 지정 플래그: `Bool`, `String`, `Int`, `Float`, `StringVec`, `IntVec`.
- 포지셔널 인수 validator: `range_args`, `valid_args` 등.
- 5계층 설정 우선순위: 기본값 → 설정 파일 → 환경변수 → CLI 플래그 → `set` 명시 값.
- `set`, `is_set`, `register_alias`, `set_key_delimiter`, `set_env_key_replacer`, `allow_empty_env`.
- 타입 getter: `get_string`, `get_int`/`get_int64`, `get_uint`, `get_bool`, `get_float`, `get_string_vec`/`get_string_slice`, `get_duration`, `get_time`, `get_size_in_bytes`.
- 열거/하위 트리: `all_keys`, `all_settings`, `get_string_map`/`get_string_map_string`/`get_string_map_string_slice`, `sub`.
- 런타임 읽기/병합: `read_config`, `merge_in_config`, `merge_config_map`.
- `set_config_file`, `set_config_name`, 경로, `automatic_env`를 통한 설정 파일 탐색 및 형식 추론.
- 명시적으로 입력하지 않은 플래그에 대한 설정 → 플래그 폴백.
- `gen_completion`을 통한 bash, zsh, fish 자동완성 생성.
- `Color`, `Style`, `Text`, `Table`, `Panel`, `Rule`, `Tree`, `Progress`를 통한 터미널 스타일링 및 렌더링.

기능을 변경하거나 문서화할 때는 구현과 함께 관련 통합 테스트 및
`README.md`/`docs/GUIDE.md`/`docs/STYLE.md` 문서를 갱신하세요.

## 양보할 수 없는 워크플로: TDD(테스트 주도 개발)

모든 코드 변경은 **Red → Green → Refactor** 사이클을 반드시 따라야 합니다.
실패하는 테스트 없이 구현을 먼저 작성하지 마세요.

### 사이클

1. **RED** — 원하는 동작을 검증하는 테스트를 작성합니다. 실행해서 **실패**하는지 확인합니다
   (`cargo test`).
2. **GREEN** — 해당 테스트를 통과시키는 *최소한*의 구현을 작성합니다.
3. **REFACTOR** — 중복 제거, 이름/구조 개선으로 정리하고 테스트는 green 상태를 유지합니다.

### 테스트 위치

| 레벨 | 위치 | 목적 |
| ---- | ---- | ---- |
| 단위 | `src/<mod>/...` `#[cfg(test)]` | 단일 모듈의 내부 로직 |
| 통합 | `tests/*.rs` | 공개 API(`Command`, `Config`, `Flag`)를 통한 라이브러리 수준 동작 |
| 바이너리 | `tests/binary.rs` | `assert_cmd` + `predicates`를 사용한 실제 프로세스 동작 |

사용자 대상 기능에는 `tests/` 아래의 **통합 테스트**를 우선 사용하세요. 도메인별로 묶습니다:
`flags.rs`, `args.rs`, `config.rs`, `errors.rs`, `lifecycle.rs`, `subcommand.rs`,
`completion.rs`, `style_*.rs`.

### 테스트 헬퍼 (`tests/common/mod.rs`)

직접 만들지 말고 다음을 재사용하세요.

- `args("--name Alice")` — 공백으로 구분된 문자열에서 `Vec<String>`을 생성합니다.
- `EnvGuard::set("KEY", "val")` — 환경변수를 설정하고 drop 시 자동 복원합니다. 환경 테스트에는
  **항상 사용**하세요(전역 뮤텍스로 병렬 안전).
- `tempdir()` / `TempDir` — 설정 파일 테스트용 자동 삭제 임시 디렉토리.

환경 의존 테스트는 병렬 안전하게 유지하세요. `EnvGuard`는 자신의 수명 동안 프로세스 전역
락을 보유하므로 설정을 실행하기 전에 생성하고, 테스트에서 `set_var`/`remove_var`를
직접 호출하지 마세요.

### 테스트에서 콜백 값 캡처

콜백은 `&CommandContext`를 받습니다. 값을 관찰하려면 `Arc<Mutex<_>>`에 캡처하세요.

```rust
let out = Arc::new(Mutex::new(String::new()));
let out2 = out.clone();
Command::new("app")
    .flag(Flag::new("name", FlagValue::String(String::new()), "name"))
    .on_run(move |ctx| *out2.lock().unwrap() = ctx.flags.get_string("name").unwrap().to_owned())
    .execute_with(args("--name Alice"))
    .unwrap();
assert_eq!(*out.lock().unwrap(), "Alice");
```

### 에러 테스트

정확한 `WrCliError` 변형을 `matches!`로 검증하세요.

```rust
let err = Command::new("app")
    .flag(Flag::new("name", FlagValue::String(String::new()), "name").required())
    .on_run(|_| {})
    .execute_with(args(""))
    .unwrap_err();
assert!(matches!(err, WrCliError::MissingRequiredFlag(n) if n == "name"));
```

빌더 시점의 잘못된 사용(중복 등록)에는 `#[should_panic(expected = "...")]`를 사용하세요.

### 테스트 실행

```sh
cargo test                      # 기본 피처
cargo test --all-features       # yaml-config 포함
cargo test --test flags          # 단일 통합 파일
cargo test --test completion     # completion API 테스트
cargo test --test style_progress # 스타일링 통합 파일 하나
cargo test -- --test-threads=1   # 관련 없는 전역 상태 경합을 진단할 때만
```

## 완료 전 검증 (필수)

변경 후 다음 세 가지가 모두 통과해야 합니다. 건너뛰지 마세요.

```sh
cargo test --all-features
cargo clippy --all-features --all-targets -- -D warnings
cargo fmt -- --check
```

## 규칙

- **빌더 패턴**: 메서드는 `self`를 소비하고 `Self`를 반환합니다(`Command::new("x").flag(...).on_run(...)`).
- **에러**: `WrCliError` 열거형을 직접 사용합니다. 새 변형은 끝에 추가합니다(열거형은
  `#[non_exhaustive]`). 사용자 에러는 `WrCliError::user(e)` 또는 `on_run_e`로 감쌉니다.
- **설정 우선순위**(낮음→높음): 기본값 → 설정 파일 → 환경변수 → CLI 플래그(명시적으로 설정한 값만) → `set` 명시 값.
- **별칭/구분자**: `register_alias`로 키 별칭을 연결하고 `set_key_delimiter`로 중첩 키 구분자를 바꿉니다.
- **env 세부**: `set_env_key_replacer`로 env 변수명 치환을, `allow_empty_env`로 빈 값 처리를 제어합니다(기본은 빈 값도 사용).
- **Persistent 플래그**: `Command::persistent_flag()`로 등록하며 서브커맨드로 전파됩니다.
- **점 표기 키**: 설정은 `"server.port"`를 지원하고, 환경변수는 `.`/`-`를 `_`로 바꾸고 대문자로 만듭니다.
- **설정 탐색**: 명시적 `set_config_file`이 우선하며, 그렇지 않으면 설정한 이름/경로와 지원
  확장자를 검색합니다. 기존 API가 그렇게 동작하는 선택적 파일 누락은 치명적으로 만들지 마세요.
- **Completion**: 새 생성기와 그에 맞는 `WrCliError::UnsupportedCompletionShell` 동작을 추가하지
  않는 한 bash, zsh, fish만 지원합니다.
- **스타일링**: `stdout_is_styled()`/`stderr_is_styled()`를 사용하고 `NO_COLOR`를 존중하세요.
  폭에 민감한 렌더링은 `display_width()`를 사용해 CJK 텍스트 정렬을 유지하세요.
- **피처 게이트 코드**: 설정 형식 백엔드(`toml-config`, `json-config`, `yaml-config`)는
  `#[cfg(feature = ...)]`로 감싸고 `--all-features`로 테스트해야 합니다.
- **요청받지 않는 한 코드에 주석을 추가하지 마세요.** 공개 API의 문서(`///`)는 환영합니다.

## 문서 구성

- `README.md` — 저장소 루트에 유지합니다(crates.io/배포용).
- `docs/GUIDE.md` — 상세 사용 레퍼런스.
- `docs/STYLE.md` — 터미널 스타일링 가이드.

가이드 성격의 문서는 `docs/`에 두고, 상호 링크는 저장소 루트 기준 상대 경로로 유지하세요.

## 커밋 스타일

저장소 이력에 맞는 간결한 명령형 메시지를 사용하세요. 예:
`feat: add config auto-discovery` / `fix: correct help width for non-ASCII`.

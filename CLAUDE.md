# wrcli

Go의 [cobra](https://github.com/spf13/cobra) / [viper](https://github.com/spf13/viper)를 모델로 한 Rust CLI 프레임워크 라이브러리.

---

## 프로젝트 구조

```
src/
  lib.rs              # 크레이트 루트, public re-export
  command/
    mod.rs            # Command 빌더 & 실행 엔진 (dispatch)
    context.rs        # CommandContext — on_run 콜백에 전달
    args.rs           # 위치 인자 검증기 (ArgValidator)
    help.rs           # --help 렌더러
  config/
    mod.rs            # Config (Viper 등가물) — 4계층 우선순위
  flag/
    mod.rs            # Flag, FlagSet, FlagValue
  style/
    mod.rs            # ANSI 스타일, Table, Panel, Rule
  error.rs            # WrCliError, Result 타입
  bin/
    testapp.rs        # assert_cmd 통합 테스트용 바이너리
tests/
  integration.rs      # 라이브러리 수준 통합 테스트
  binary.rs           # assert_cmd 바이너리 테스트
examples/
  basic.rs            # 기본 사용 예제
  styled.rs           # 스타일 API 데모
```

---

## 핵심 타입

| 타입 | 위치 | 역할 |
|------|------|------|
| `Command` | `command/mod.rs` | 명령 트리 노드, 플루언트 빌더 |
| `CommandContext<'a>` | `command/context.rs` | on_run 콜백에 전달되는 컨텍스트 |
| `Flag` / `FlagSet` / `FlagValue` | `flag/mod.rs` | 플래그 정의·파싱·조회 |
| `Config` / `ConfigValue` | `config/mod.rs` | Viper식 설정 저장소 |
| `WrCliError` | `error.rs` | 에러 열거형 |
| `Style`, `Color`, `Table`, `Panel`, `Rule` | `style/mod.rs` | ANSI 터미널 출력 |

---

## Config 우선순위 (낮음→높음)

1. 프로그래밍 기본값 (`Config::set_default`)
2. 설정 파일 (`Config::read_in_config`)
3. 환경 변수 (`automatic_env` / `bind_env`)
4. CLI 플래그 (사용자가 실제로 입력한 값만, 기본값 제외)

---

## FlagValue 타입

`Bool`, `String`, `Int(i64)`, `Float(f64)`, `StringVec`, `IntVec`

- `StringVec` / `IntVec`는 `--tag a --tag b` 식으로 반복 입력 지원
- `--` sentinel 이후는 모두 위치 인자로 처리

---

## 라이프사이클 훅 실행 순서

```
root:persistent_pre_run
  └─ sub:persistent_pre_run
       ├─ sub:pre_run
       ├─ sub:run  (또는 run_e — 에러 시 이후 훅 중단)
       ├─ sub:post_run
  └─ sub:persistent_post_run
root:persistent_post_run
```

- `on_run_e`가 `Err`를 반환하면 post_run / persistent_post_run은 실행되지 않음

---

## Cargo features

| Feature | 기본 | 설명 |
|---------|------|------|
| `toml-config` | ✔ | TOML 설정 파일 파싱 |
| `json-config` | ✔ | JSON 설정 파일 파싱 |
| `yaml-config` | — | YAML 설정 파일 파싱 (`serde_yml`) |

---

## 빌드 & 테스트

```sh
cargo build                  # 빌드
cargo test                   # 전체 테스트
cargo test --test integration  # 통합 테스트만
cargo run --example styled   # 스타일 데모
```

`yaml-config` 기능 포함 테스트:
```sh
cargo test --features yaml-config
```

---

## 코딩 규칙

- **빌더 패턴**: `Command::new("name").flag(...).on_run(...).execute()` — 모든 메서드는 `Self`를 consume해서 반환
- **에러**: `WrCliError` 열거형을 직접 사용; 사용자 커스텀 에러는 `WrCliError::user(e)` 또는 `on_run_e`에서 반환
- **스타일**: TTY 감지는 `stdout_is_styled()` / `stderr_is_styled()` 사용; `NO_COLOR` 환경변수 자동 준수
- **persistent flag**: `Command::persistent_flag()`로 등록하면 하위 서브커맨드에 자동 전파
- **설정 키**: 점 표기법 지원 (`"server.port"`)

---

## 주요 의존성

| 크레이트 | 용도 |
|----------|------|
| `indexmap` | 플래그 삽입 순서 보존 (help 출력용) |
| `shellexpand` | config path에서 `~` / `$VAR` 확장 |
| `toml` | TOML 파싱 (optional) |
| `serde_json` | JSON 파싱 (optional) |
| `serde_yml` | YAML 파싱 (optional) |
| `assert_cmd` / `predicates` | 바이너리 통합 테스트 (dev) |

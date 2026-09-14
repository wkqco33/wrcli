# wrcli

[English](README.md) | [한국어](README.ko.md)

[![CI](https://github.com/wkqco33/wrcli/actions/workflows/ci.yml/badge.svg)](https://github.com/wkqco33/wrcli/actions/workflows/ci.yml)

Releases are published to crates.io automatically when a matching version tag
is pushed (for example, `v0.5.0`). Configure the repository secret
`CARGO_REGISTRY_TOKEN` with a crates.io API token before creating a release tag.
[![Crates.io](https://img.shields.io/crates/v/wrcli.svg)](https://crates.io/crates/wrcli)
[![Documentation](https://docs.rs/wrcli/badge.svg)](https://docs.rs/wrcli)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Go의 [cobra](https://github.com/spf13/cobra) + [viper](https://github.com/spf13/viper)에서 영감을 받은 Rust CLI 프레임워크 라이브러리.  
트리 구조의 서브커맨드, 타입 안전 플래그, 다중 소스 설정을 플루언트 빌더 API로 조합할 수 있습니다.

---

## 특징

- 무한 중첩 서브커맨드 + 알리아스
- 타입 안전 플래그 (`bool`, `string`, `int`, `float`, `string[]`, `int[]`)
- persistent 플래그 — 루트에 등록하면 모든 서브커맨드에 자동 전파
- **숨김(`hidden`)** 커맨드·플래그 — help/completion에서 제외, 실행은 유지
- **Deprecated** 커맨드·플래그 — 사용 시 stderr 경고
- **플래그 제약 그룹**: `mutually_exclusive`, `required_together`, `one_required`
- **오타 제안(`Did you mean`)** — 미등록 커맨드/플래그에 편집 거리 기반 후보 제시, `Command::suggest_for`
- **동적 completion** — `Command::complete` / `completion_request` / `arg_candidates`
- **CSV 슬라이스 플래그** — `Flag::comma_separated()`로 `--tag a,b,c` 분리
- **부모 로컬 플래그** — `app --profile prod deploy`처럼 서브커맨드 앞에서도 파싱
- **usage 힌트** — `Command::usage_args("<name>")`
- **종료 코드 분류**: `WrCliError::exit_code()` / `is_usage_error()`, `Command::execute_or_exit()`
- 5계층 설정 우선순위: 기본값 → 파일(TOML/JSON/YAML/INI/dotenv/properties) → 환경변수 → CLI 플래그 → 명시 값(`set`)
- 설정 파일 **자동 탐지** (`set_config_file`, 형식/경로 자동 판별)
- **별칭(`register_alias`)**, 커스텀 키 구분자, env key replacer, 빈 env 처리 제어
- **Config ↔ Flag 자동 바인딩** — 명시되지 않은 플래그를 설정값으로 시드
- 타입 조회: `get_string`/`get_int`/`get_uint`/`get_bool`/`get_float`/`get_string_vec`/`get_duration`/`get_time`/`get_size_in_bytes`
- **열거·하위 트리**: `all_keys`, `all_settings`, `get_string_map*`, `sub`
- **런타임 읽기·병합**: `read_config`, `merge_in_config`, `merge_config_map`
- **설정 저장**: `write_config_as`, `safe_write_config_as`
- **구조체 역직렬화**(`serde` 피처): `unmarshal`, `unmarshal_key`
- **설정 파일 감시**: `on_config_change`, `watch_config`, `ConfigWatcher`
- **포맷**: TOML·JSON(기본), YAML·INI·dotenv·Java properties(피처)
- 라이프사이클 훅: `persistent_pre_run` → `pre_run` → `run` → `post_run` → `persistent_post_run`
- `CommandContext` getter: `get_string`/`get_int`/`get_uint`/`get_bool`/`get_float`/`get_string_vec`/`get_int_vec`/`get_duration`/`get_time`/`get_size_in_bytes`/`get_string_map`/`is_set`
- **내장 `help` 서브커맨드** — `app help`, `app help sub [subsub]`
- **help 예제·지원·문서 링크** — `example`, `support_url`, `docs_url`(`{command}` 치환, 하위 상속)
- **러너 없는 커맨드 정책** — 서브커맨드만 있으면 help + 종료 코드 0, `help_on_missing_runner()`
- **표준 플래그** — `standard_flags()`: `-q/--quiet`, `-f/--force`, `--no-input`, `--no-color`, `--plain`, `--json`, `--color`, `--confirm`
- **출력 포맷** — `OutputFormat`(Human/Plain/Json), `Table::render_plain()`
- **대화형 프롬프트** — `confirm`, `confirm_severe`, `prompt_password` (TTY/`--no-input` 안전)
- **민감 플래그** — `Flag::sensitive()` 값 마스킹
- **선택적 값** — `Flag::optional_value()` (`none` = 값 없음)
- **`-` 입출력** — `wrcli::io::{open_reader, open_writer, read_to_string}`
- **색상 정책** — `FORCE_COLOR`/`NO_COLOR`/`TERM=dumb`/`*_NO_COLOR`/`--no-color`, `ColorChoice`
- **페이저** — `style::pager::page` (`PAGER`, 기본 `less -FIRX`)
- **SIGINT** — `signal` 피처, `interrupt_message` (종료 코드 130)
- **버그 리포트 URL** — `bug_report_url`
- **Completion 스크립트 생성** (bash / zsh / fish)
- 풍부한 터미널 스타일링: `Style`, `Color`, `Table`, `Panel`, `Rule`, `Tree`, `Text`, `Progress`, `KeyVal`, `Badge`, `List`, `Spinner`
- `execute_with()` — 실제 argv 없이 인수를 직접 주입해 단위 테스트 가능

- `--help` / `--version` 자동 생성

---

## 설치

```toml
[dependencies]
wrcli = "0.5"

# YAML 설정 파일도 필요한 경우
wrcli = { version = "0.5", features = ["yaml-config"] }
```

---

## 빠른 시작

```rust
use wrcli::{Command, Flag, FlagValue, Config};
use wrcli::args::minimum_n_args;

fn main() {
    let config = Config::new()
        .set_default("server.port", 8080i64)
        .automatic_env()
        .set_env_prefix("MYAPP");

    Command::new("myapp")
        .version("1.0.0")
        .short("My awesome CLI")
        .with_config(config)
        .persistent_flag(
            Flag::new("verbose", FlagValue::Bool(false), "enable verbose output").short('v'),
        )
        .subcommand(
            Command::new("greet")
                .short("Print a greeting")
                .args(minimum_n_args(1))
                .on_run(|ctx| {
                    for name in &ctx.args {
                        println!("Hello, {}!", name);
                    }
                }),
        )
        .execute()
        .unwrap();
}
```

```bash
$ myapp greet Alice Bob
Hello, Alice!
Hello, Bob!

$ myapp --help
Usage:
  myapp [command]
  myapp [flags]
...
```

예제 전체 실행:

```sh
cargo run --example basic -- --help
cargo run --example styled
```

---

## 문서

상세 레퍼런스는 [docs/GUIDE.ko.md](docs/GUIDE.ko.md)를, 터미널 스타일링은 [docs/STYLE.ko.md](docs/STYLE.ko.md)를 참고하세요.

| 항목 | 바로가기 |
| ---- | ------- |
| 커맨드 & 서브커맨드 | [docs/GUIDE.ko.md#커맨드](docs/GUIDE.ko.md#커맨드) |
| 플래그 타입 & 파싱 문법 | [docs/GUIDE.ko.md#플래그](docs/GUIDE.ko.md#플래그) |
| 설정(Config) & 우선순위 | [docs/GUIDE.ko.md#설정config](docs/GUIDE.ko.md#설정config) |
| 설정 파일 자동 탐지 | [docs/GUIDE.ko.md#설정-파일-자동-탐지](docs/GUIDE.ko.md#설정-파일-자동-탐지) |
| Config ↔ Flag 바인딩 | [docs/GUIDE.ko.md#config--flag-자동-바인딩](docs/GUIDE.ko.md#config--flag-자동-바인딩) |
| 명시 값 & 별칭 | [docs/GUIDE.ko.md#명시-값과-별칭](docs/GUIDE.ko.md#명시-값과-별칭) |
| 키 구분자 & env 설정 | [docs/GUIDE.ko.md#키-구분자와-env-설정](docs/GUIDE.ko.md#키-구분자와-env-설정) |
| 설정 조회 (typed getter) | [docs/GUIDE.ko.md#설정-조회](docs/GUIDE.ko.md#설정-조회) |
| 열거 & 맵 조회 | [docs/GUIDE.ko.md#열거와-맵-조회](docs/GUIDE.ko.md#열거와-맵-조회) |
| 런타임 읽기 & 병합 | [docs/GUIDE.ko.md#런타임-읽기와-병합](docs/GUIDE.ko.md#런타임-읽기와-병합) |
| 설정 저장 | [docs/GUIDE.ko.md#설정-저장](docs/GUIDE.ko.md#설정-저장) |
| 구조체 역직렬화 | [docs/GUIDE.ko.md#구조체-역직렬화](docs/GUIDE.ko.md#구조체-역직렬화) |
| 설정 파일 감시 | [docs/GUIDE.ko.md#설정-파일-감시](docs/GUIDE.ko.md#설정-파일-감시) |
| Completion 생성 | [docs/GUIDE.ko.md#completion-스크립트-생성](docs/GUIDE.ko.md#completion-스크립트-생성) |
| 라이프사이클 훅 | [docs/GUIDE.ko.md#라이프사이클-훅](docs/GUIDE.ko.md#라이프사이클-훅) |
| 숨김 · Deprecated · 제약 | [docs/GUIDE.ko.md#숨김--deprecated--플래그-제약](docs/GUIDE.ko.md#숨김--deprecated--플래그-제약) |
| 동적 completion | [docs/GUIDE.ko.md#동적-completion](docs/GUIDE.ko.md#동적-completion) |
| CommandContext | [docs/GUIDE.ko.md#commandcontext](docs/GUIDE.ko.md#commandcontext) |
| 에러 처리 | [docs/GUIDE.ko.md#에러-처리](docs/GUIDE.ko.md#에러-처리) |
| 테스트 작성 | [docs/GUIDE.ko.md#테스트-작성](docs/GUIDE.ko.md#테스트-작성) |
| 피처 플래그 | [docs/GUIDE.ko.md#피처-플래그](docs/GUIDE.ko.md#피처-플래그) |
| 터미널 스타일링 | [docs/STYLE.ko.md](docs/STYLE.ko.md) |
| 변경 이력 | [CHANGELOG.ko.md](CHANGELOG.ko.md) |

---

## 라이선스

[MIT](LICENSE)

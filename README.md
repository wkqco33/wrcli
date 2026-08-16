# wrcli

[![CI](https://github.com/wkqco33/wrcli/actions/workflows/ci.yml/badge.svg)](https://github.com/wkqco33/wrcli/actions/workflows/ci.yml)
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
- 4계층 설정 우선순위: 기본값 → 파일(TOML/JSON/YAML) → 환경변수 → CLI 플래그
- 설정 파일 **자동 탐지** (`set_config_file`, 형식/경로 자동 판별)
- **Config ↔ Flag 자동 바인딩** — 명시되지 않은 플래그를 설정값으로 시드
- 라이프사이클 훅: `persistent_pre_run` → `pre_run` → `run` → `post_run` → `persistent_post_run`
- **Completion 스크립트 생성** (bash / zsh / fish)
- 풍부한 터미널 스타일링: `Style`, `Color`, `Table`, `Panel`, `Rule`, `Tree`, `Text`, `Progress`
- `execute_with()` — 실제 argv 없이 인수를 직접 주입해 단위 테스트 가능
- `--help` / `--version` 자동 생성

---

## 설치

```toml
[dependencies]
wrcli = "0.1"

# YAML 설정 파일도 필요한 경우
wrcli = { version = "0.1", features = ["yaml-config"] }
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

상세 레퍼런스는 [GUIDE.md](GUIDE.md)를, 터미널 스타일링은 [STYLE.md](STYLE.md)를 참고하세요.

| 항목 | 바로가기 |
| ---- | ------- |
| 커맨드 & 서브커맨드 | [GUIDE.md#커맨드](GUIDE.md#커맨드) |
| 플래그 타입 & 파싱 문법 | [GUIDE.md#플래그](GUIDE.md#플래그) |
| 설정(Config) & 우선순위 | [GUIDE.md#설정config](GUIDE.md#설정config) |
| 설정 파일 자동 탐지 | [GUIDE.md#설정-파일-자동-탐지](GUIDE.md#설정-파일-자동-탐지) |
| Config ↔ Flag 바인딩 | [GUIDE.md#config--flag-자동-바인딩](GUIDE.md#config--flag-자동-바인딩) |
| Completion 생성 | [GUIDE.md#completion-스크립트-생성](GUIDE.md#completion-스크립트-생성) |
| 라이프사이클 훅 | [GUIDE.md#라이프사이클-훅](GUIDE.md#라이프사이클-훅) |
| CommandContext | [GUIDE.md#commandcontext](GUIDE.md#commandcontext) |
| 에러 처리 | [GUIDE.md#에러-처리](GUIDE.md#에러-처리) |
| 테스트 작성 | [GUIDE.md#테스트-작성](GUIDE.md#테스트-작성) |
| 피처 플래그 | [GUIDE.md#피처-플래그](GUIDE.md#피처-플래그) |
| 터미널 스타일링 | [STYLE.md](STYLE.md) |

---

## 라이선스

[MIT](LICENSE)

# wrcli

[![Crates.io](https://img.shields.io/crates/v/wrcli.svg)](https://crates.io/crates/wrcli)
[![Docs.rs](https://docs.rs/wrcli/badge.svg)](https://docs.rs/wrcli)

Go의 [cobra](https://github.com/spf13/cobra) + [viper](https://github.com/spf13/viper)에서 영감을 받은 Rust CLI 프레임워크 라이브러리입니다.  
트리 구조의 서브커맨드, 타입 안전 플래그, 다중 소스 설정(파일·환경변수·CLI 플래그)을 유창한(fluent) 빌더 API로 조합할 수 있습니다.

---

## 목차

- [특징](#특징)
- [빠른 시작](#빠른-시작)
- [설치](#설치)
  - [Private Git 저장소에서 설치](#private-git-저장소에서-설치)
- [커맨드](#커맨드)
  - [서브커맨드 & 알리아스](#서브커맨드--알리아스)
  - [버전 플래그](#버전-플래그)
- [플래그](#플래그)
  - [플래그 타입](#플래그-타입)
  - [숏 플래그 & 파싱 문법](#숏-플래그--파싱-문법)
  - [필수 플래그](#필수-플래그)
  - [퍼시스턴트 플래그](#퍼시스턴트-플래그)
  - [반복 가능한 벡터 플래그](#반복-가능한-벡터-플래그)
- [포지셔널 인수 검증](#포지셔널-인수-검증)
- [라이프사이클 훅](#라이프사이클-훅)
- [설정(Config)](#설정config)
  - [기본값](#기본값)
  - [설정 파일](#설정-파일)
  - [환경 변수](#환경-변수)
  - [우선순위 규칙](#우선순위-규칙)
  - [설정 조회](#설정-조회)
- [CommandContext](#commandcontext)
- [에러 처리](#에러-처리)
- [CLI 테스트 작성](#cli-테스트-작성)
- [피처 플래그](#피처-플래그)

---

## 특징

- **트리형 서브커맨드** — 무한 중첩, 알리아스 지원
- **타입 안전 플래그** — `bool`, `string`, `int`, `float`, `string[]`, `int[]`
- **퍼시스턴트 플래그** — 루트에서 선언하면 모든 서브커맨드에 자동 전파
- **다중 소스 설정** — 기본값 → 설정 파일(TOML/JSON/YAML) → 환경 변수 → CLI 플래그 순 우선순위
- **자동 환경 변수 바인딩** — 키 `server.port` → `MYAPP_SERVER_PORT` 자동 매핑
- **라이프사이클 훅** — `persistent_pre_run` → `pre_run` → `run` / `run_e` → `post_run` → `persistent_post_run`
- **포지셔널 인수 검증** — 내장 validator 또는 커스텀 validator
- **자동 도움말** — `--help` / `-h` 플래그 자동 생성
- **`execute_with()`** — 실제 `argv` 없이 직접 인수 주입으로 단위 테스트 가능

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

```
$ myapp greet Alice Bob
Hello, Alice!
Hello, Bob!

$ myapp --help
myapp 1.0.0
My awesome CLI

USAGE:
    myapp [FLAGS] <SUBCOMMAND>

FLAGS:
    -h, --help       Print help information
    -V, --version    Print version information
    -v, --verbose    enable verbose output

SUBCOMMANDS:
    greet    Print a greeting
```

---

## 설치

`Cargo.toml`에 추가합니다:

```toml
[dependencies]
wrcli = "0.1"
```

설정 파일 형식별 피처 플래그 (기본값: TOML + JSON 활성화):

```toml
# TOML + JSON만 필요한 경우 (기본)
wrcli = "0.1"

# YAML도 필요한 경우
wrcli = { version = "0.1", features = ["yaml-config"] }

# 설정 파일 지원이 불필요한 경우
wrcli = { version = "0.1", default-features = false }
```

### Private Git 저장소에서 설치

crates.io에 배포하지 않고 private Git 저장소를 직접 참조할 수 있습니다.

**SSH (권장)**

SSH 키가 `ssh-agent`에 등록되어 있으면 추가 설정 없이 동작합니다:

```toml
[dependencies]
wrcli = { git = "git@github.com:your-org/wrcli.git" }
```

**HTTPS + 토큰**

```toml
[dependencies]
wrcli = { git = "https://github.com/your-org/wrcli.git" }
```

GitHub Actions 등 CI 환경에서는 아래와 같이 인증을 설정합니다:

```yaml
- name: Configure git credentials
  run: |
    git config --global \
      url."https://x-access-token:${{ secrets.GITHUB_TOKEN }}@github.com/".insteadOf \
      "https://github.com/"
```

**브랜치 / 태그 / 커밋 고정**

```toml
wrcli = { git = "git@github.com:your-org/wrcli.git", branch = "main"    }
wrcli = { git = "git@github.com:your-org/wrcli.git", tag    = "v0.2.0"  }
wrcli = { git = "git@github.com:your-org/wrcli.git", rev    = "a1b2c3d" }
```

`rev`로 특정 커밋을 고정하면 `Cargo.lock`과 함께 완전히 재현 가능한 빌드를 보장합니다.

**로컬 경로 (모노레포 / 개발 중)**

같은 워크스페이스나 인접 디렉토리에 있을 때는 경로 의존성을 사용합니다:

```toml
wrcli = { path = "../wrcli" }
```

lock 파일을 최신으로 유지하려면 `cargo update -p wrcli`를 실행합니다.

---

## 커맨드

모든 커맨드는 `Command::new("이름")`으로 시작하는 빌더 체인으로 구성합니다.  
루트 커맨드에서 `.execute()`를 호출하면 `std::env::args()`를 파싱하여 실행합니다.

```rust
Command::new("app")
    .short("한 줄 설명 (부모 커맨드의 도움말 목록에 표시)")
    .long("긴 설명 (이 커맨드의 --help에 표시)")
    .on_run(|ctx| {
        println!("실행됨!");
    })
    .execute()
    .unwrap();
```

### 서브커맨드 & 알리아스

```rust
Command::new("app")
    .subcommand(
        Command::new("deploy")
            .alias("d")        // `app d`로도 호출 가능
            .alias("ship")
            .short("Deploy the application")
            .on_run(|ctx| {
                println!("Deploying...");
            }),
    )
    .execute()
    .unwrap();
```

중첩 서브커맨드도 동일한 방식으로 `.subcommand()` 안에 또 다른 `Command`를 넣으면 됩니다.

### 버전 플래그

```rust
Command::new("app")
    .version("2.3.1")   // --version / -V 자동 활성화
    .on_run(|_| {})
    .execute()
    .unwrap();
```

```
$ app --version
app 2.3.1
```

---

## 플래그

### 플래그 타입

| `FlagValue` 변형 | Rust 타입 | 조회 메서드 |
|---|---|---|
| `Bool(bool)` | `bool` | `get_bool("name")` |
| `String(String)` | `String` | `get_string("name")` |
| `Int(i64)` | `i64` | `get_int("name")` |
| `Float(f64)` | `f64` | `get_float("name")` |
| `StringVec(Vec<String>)` | `Vec<String>` | `get_string_vec("name")` |
| `IntVec(Vec<i64>)` | `Vec<i64>` | — (raw `FlagValue` 사용) |

```rust
Command::new("app")
    .flag(Flag::new("output",  FlagValue::String(String::new()), "output file"))
    .flag(Flag::new("count",   FlagValue::Int(1),                "repeat count"))
    .flag(Flag::new("ratio",   FlagValue::Float(1.0),            "compression ratio"))
    .flag(Flag::new("verbose", FlagValue::Bool(false),           "verbose mode"))
    .on_run(|ctx| {
        let output  = ctx.flags.get_string("output").unwrap_or("out.txt");
        let count   = ctx.flags.get_int("count").unwrap_or(1);
        let ratio   = ctx.flags.get_float("ratio").unwrap_or(1.0);
        let verbose = ctx.flags.get_bool("verbose").unwrap_or(false);
        // ...
    })
    .execute()
    .unwrap();
```

### 숏 플래그 & 파싱 문법

```rust
Flag::new("output", FlagValue::String(String::new()), "output file").short('o')
```

지원하는 파싱 문법:

```
--output result.txt   # 롱 플래그, 공백 구분
--output=result.txt   # 롱 플래그, = 구분
-o result.txt         # 숏 플래그
-abc                  # 불 플래그 묶음 (-a -b -c 동일)
--verbose             # 불 플래그 (값 생략 시 true)
--verbose=false       # 불 플래그 명시적 false
--                    # 이후 모두 포지셔널 인수로 처리
```

### 필수 플래그

```rust
Flag::new("token", FlagValue::String(String::new()), "API token")
    .required()
```

플래그가 제공되지 않으면 `WrCliError::MissingRequiredFlag`를 반환합니다.

### 퍼시스턴트 플래그

루트(또는 중간) 커맨드에 등록하면 모든 하위 서브커맨드에서 자동으로 사용할 수 있습니다.

```rust
Command::new("app")
    .persistent_flag(
        Flag::new("config", FlagValue::String(String::new()), "config file path").short('c'),
    )
    .subcommand(
        Command::new("serve")
            .on_run(|ctx| {
                // 여기서도 "config" 플래그에 접근 가능
                let cfg_path = ctx.flags.get_string("config").unwrap_or("config.toml");
            }),
    )
    .execute()
    .unwrap();
```

### 반복 가능한 벡터 플래그

`--tag` 플래그를 여러 번 지정해 값을 누적할 수 있습니다:

```rust
Command::new("app")
    .flag(Flag::new("tag", FlagValue::StringVec(vec![]), "add a tag (repeatable)"))
    .on_run(|ctx| {
        let tags = ctx.get_string_vec("tag").unwrap_or_default();
        // tags: ["frontend", "prod", "v2"]
        for tag in &tags {
            println!("tag: {}", tag);
        }
    })
    .execute()
    .unwrap();
```

```
$ app --tag frontend --tag prod --tag v2
tag: frontend
tag: prod
tag: v2
```

---

## 포지셔널 인수 검증

`wrcli::args` 모듈에 내장 validator가 있습니다:

```rust
use wrcli::args::{no_args, arbitrary_args, exact_args,
                  minimum_n_args, maximum_n_args, range_args, valid_args};

Command::new("copy")
    .args(exact_args(2))          // 정확히 2개
    .on_run(|ctx| {
        let src = &ctx.args[0];
        let dst = &ctx.args[1];
    })
    // ...
```

| 함수 | 설명 |
|---|---|
| `no_args()` | 포지셔널 인수 없음 |
| `arbitrary_args()` | 제한 없음 |
| `exact_args(n)` | 정확히 n개 |
| `minimum_n_args(n)` | 최소 n개 |
| `maximum_n_args(n)` | 최대 n개 |
| `range_args(min, max)` | min 이상 max 이하 |
| `valid_args(vec![...])` | 허용 목록에 포함된 값만 허용 |

커스텀 validator도 `ArgValidator` 타입으로 직접 작성할 수 있습니다:

```rust
use wrcli::args::ArgValidator;
use wrcli::error::WrCliError;

fn only_files() -> ArgValidator {
    Box::new(|args| {
        for arg in args {
            if !std::path::Path::new(arg).exists() {
                return Err(WrCliError::ArgValidationFailed(
                    format!("파일을 찾을 수 없음: {}", arg)
                ));
            }
        }
        Ok(())
    })
}
```

---

## 라이프사이클 훅

커맨드 실행 순서는 다음과 같습니다 (cobra의 훅 체계와 동일):

```
persistent_pre_run  (루트 → 리프 순서로 체인)
pre_run             (매칭된 리프 커맨드만)
run / run_e         (매칭된 리프 커맨드만)
post_run            (매칭된 리프 커맨드만)
persistent_post_run (리프 → 루트 순서로 체인)
```

- `on_run_e`에서 `Err`를 반환하면 `post_run` / `persistent_post_run`은 실행되지 않습니다.

```rust
Command::new("app")
    .on_persistent_pre_run(|_ctx| {
        println!("항상 실행: 로깅 초기화");
    })
    .subcommand(
        Command::new("deploy")
            .on_pre_run(|_ctx| println!("배포 전 검증"))
            .on_run_e(|ctx| {
                // 에러 반환 시 post_run은 건너뜀
                do_deploy()?;
                Ok(())
            })
            .on_post_run(|_ctx| println!("배포 완료 알림"))
    )
    .on_persistent_post_run(|_ctx| {
        println!("항상 실행: 정리 작업");
    })
    .execute()
    .unwrap();
```

---

## 설정(Config)

Go의 Viper에 해당하는 설정 스토어입니다. 루트 커맨드에 `.with_config(config)`로 연결하면 모든 서브커맨드의 `ctx.config`로 접근할 수 있습니다.

### 기본값

```rust
let config = Config::new()
    .set_default("server.host", "127.0.0.1")
    .set_default("server.port", 8080i64)
    .set_default("debug",       false);
```

### 설정 파일

```rust
let config = Config::new()
    .set_config_name("myapp")      // 파일명 (확장자 제외)
    .set_config_type("toml")       // "toml" | "json" | "yaml"
    .add_config_path(".")          // 검색 디렉토리 (여러 개 가능)
    .add_config_path("~/.config/myapp");

// 커맨드 실행 전에 수동으로 읽거나
config.read_in_config().unwrap();

// 파일이 없어도 무시하고 싶다면
config.read_in_config().ok();
```

TOML 예시 (`myapp.toml`):

```toml
[server]
host = "0.0.0.0"
port = 9000

[database]
url = "postgres://localhost/mydb"
```

점(dot) 표기법으로 중첩 키에 접근합니다: `config.get_string("server.host")`.

### 환경 변수

```rust
let config = Config::new()
    .automatic_env()               // 모든 키를 자동으로 환경 변수에 매핑
    .set_env_prefix("MYAPP");      // MYAPP_ 접두사 추가
```

`automatic_env()` + `set_env_prefix("MYAPP")`이면:

| 설정 키 | 환경 변수 |
|---|---|
| `server.port` | `MYAPP_SERVER_PORT` |
| `database.url` | `MYAPP_DATABASE_URL` |
| `debug` | `MYAPP_DEBUG` |

특정 키와 환경 변수를 명시적으로 연결하려면:

```rust
config.bind_env("token", "API_TOKEN");
// config.get_string("token") → $API_TOKEN 값 반환
```

### 우선순위 규칙

높은 숫자가 낮은 숫자를 덮어씁니다:

```
1. 기본값           (set_default)         ← 가장 낮음
2. 설정 파일        (read_in_config)
3. 환경 변수        (automatic_env / bind_env)
4. CLI 플래그 값    (사용자가 실제로 입력한 경우)  ← 가장 높음
```

> CLI에서 플래그 기본값은 주입되지 않습니다. 사용자가 실제로 지정한 값만 설정을 덮어씁니다.

### 설정 조회

```rust
// Config에서 직접
ctx.config.get_string("server.host")      // Option<String>
ctx.config.get_int("server.port")         // Option<i64>
ctx.config.get_bool("debug")              // Option<bool>
ctx.config.get_float("ratio")             // Option<f64>
ctx.config.get_string_vec("allowed.ips")  // Option<Vec<String>>
ctx.config.get(&"server.port")            // Option<&ConfigValue>
```

---

## CommandContext

`on_run` 콜백이 받는 `&CommandContext`는 플래그·설정·포지셔널 인수를 묶어서 제공합니다.

```rust
.on_run(|ctx| {
    // 포지셔널 인수
    let first_arg = &ctx.args[0];

    // 플래그만 조회
    let verbose = ctx.flags.get_bool("verbose").unwrap_or(false);
    let name    = ctx.flags.get_string("name").unwrap_or("");

    // 플래그를 먼저, 없으면 config를 fallback으로 자동 조회
    let host = ctx.get_string("server.host").unwrap_or_default();
    let port = ctx.get_int("server.port").unwrap_or(8080);
    let dbg  = ctx.get_bool("debug").unwrap_or(false);
    let tags = ctx.get_string_vec("tags").unwrap_or_default();

    // 현재 커맨드 경로 (예: ["myapp", "config", "get"])
    println!("커맨드: {}", ctx.command_name());
    println!("경로:   {:?}", ctx.command_path);
})
```

`ctx.get_*(key)` 메서드는 **플래그 → 설정** 순으로 자동 탐색합니다.  
플래그명과 설정 키가 같을 때 편리하게 사용할 수 있습니다.

---

## 에러 처리

`.execute()`는 `Result<(), WrCliError>`를 반환합니다. 에러 종류:

| 변형 | 발생 시점 |
|---|---|
| `UnknownFlag` | 미등록 플래그를 사용한 경우 |
| `UnknownSubcommand` | 미등록 서브커맨드를 사용한 경우 |
| `MissingRequiredFlag` | `.required()` 플래그 미입력 |
| `InvalidFlagValue` | 타입 불일치 (예: `--count abc`) |
| `ArgValidationFailed` | 포지셔널 인수 검증 실패 |
| `CommandHasNoRunner` | `on_run` 미등록 커맨드 실행 시 |
| `ConfigFileNotFound` | 설정 파일을 찾을 수 없음 |
| `ConfigParseError` | 설정 파일 파싱 실패 |
| `UserError` | `on_run_e`에서 반환한 에러 |

권장 패턴:

```rust
fn main() {
    if let Err(e) = build_cli().execute() {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}
```

`on_run_e`에서 임의의 에러 타입을 반환하려면 `WrCliError::user(e)` 헬퍼를 사용합니다:

```rust
.on_run_e(|ctx| {
    let data = std::fs::read_to_string("data.txt")
        .map_err(WrCliError::user)?;
    Ok(())
})
```

---

## CLI 테스트 작성

### 단위 테스트 — `execute_with()`

```rust
#[test]
fn test_greet_command() {
    use std::sync::{Arc, Mutex};
    use wrcli::{Command, Flag, FlagValue};

    let output = Arc::new(Mutex::new(String::new()));
    let out2   = output.clone();

    Command::new("app")
        .flag(Flag::new("name", FlagValue::String(String::new()), "name").short('n'))
        .on_run(move |ctx| {
            *out2.lock().unwrap() =
                ctx.flags.get_string("name").unwrap_or("").to_owned();
        })
        .execute_with(vec!["--name".into(), "Alice".into()])
        .unwrap();

    assert_eq!(*output.lock().unwrap(), "Alice");
}
```

### 이진 테스트 — `assert_cmd`

실제 프로세스를 실행해 stdout / stderr / exit code를 검증합니다.

```toml
[dev-dependencies]
assert_cmd = "2"
predicates = "3"
```

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn greet_basic() {
    Command::cargo_bin("myapp").unwrap()
        .args(["greet", "Alice"])
        .assert()
        .success()
        .stdout("Hello, Alice!\n");
}

#[test]
fn unknown_flag_fails() {
    Command::cargo_bin("myapp").unwrap()
        .args(["--no-such-flag"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown flag"));
}
```

---

## 피처 플래그

| 피처 | 기본 활성화 | 설명 |
|---|:---:|---|
| `toml-config` | ✅ | TOML 설정 파일 지원 |
| `json-config` | ✅ | JSON 설정 파일 지원 |
| `yaml-config` | ❌ | YAML 설정 파일 지원 (`serde_yml`) |

```toml
# 모든 형식 활성화
wrcli = { version = "0.1", features = ["yaml-config"] }

# 최소 빌드 (설정 파일 지원 없음)
wrcli = { version = "0.1", default-features = false }
```

---

## 전체 예제

`examples/basic.rs`를 실행해 주요 기능을 한눈에 확인할 수 있습니다:

```
cargo run --example basic -- --help
cargo run --example basic -- serve --port 9000
cargo run --example basic -- config get server.host
MYAPP_SERVER_PORT=3000 cargo run --example basic -- serve
```

---

## 라이선스

MIT

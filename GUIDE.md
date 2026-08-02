# wrcli 상세 가이드

---

## 목차

- [설치](#설치)
- [커맨드](#커맨드)
- [플래그](#플래그)
- [포지셔널 인수 검증](#포지셔널-인수-검증)
- [라이프사이클 훅](#라이프사이클-훅)
- [설정(Config)](#설정config)
- [설정 파일 자동 탐지](#설정-파일-자동-탐지)
- [Config ↔ Flag 자동 바인딩](#config--flag-자동-바인딩)
- [CommandContext](#commandcontext)
- [Completion 스크립트 생성](#completion-스크립트-생성)
- [에러 처리](#에러-처리)
- [테스트 작성](#테스트-작성)
- [피처 플래그](#피처-플래그)

---

## 설치

```toml
[dependencies]
wrcli = "0.1"

# YAML도 필요한 경우
wrcli = { version = "0.1", features = ["yaml-config"] }

# 설정 파일 지원 없이 최소 빌드
wrcli = { version = "0.1", default-features = false }
```

### Git 저장소 직접 참조

```toml
# SSH (권장)
wrcli = { git = "git@github.com:your-org/wrcli.git" }

# 브랜치 / 태그 / 커밋 고정
wrcli = { git = "git@github.com:your-org/wrcli.git", tag = "v0.2.0" }
wrcli = { git = "git@github.com:your-org/wrcli.git", rev = "a1b2c3d" }

# 로컬 경로 (모노레포 / 개발 중)
wrcli = { path = "../wrcli" }
```

CI 환경에서 HTTPS 인증:

```yaml
- name: Configure git credentials
  run: |
    git config --global \
      url."https://x-access-token:${{ secrets.GITHUB_TOKEN }}@github.com/".insteadOf \
      "https://github.com/"
```

---

## 커맨드

모든 커맨드는 `Command::new("이름")`으로 시작하는 빌더 체인으로 구성합니다.

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
            .on_run(|_| println!("Deploying...")),
    )
    .execute()
    .unwrap();
```

서브커맨드 안에 또 다른 `Command`를 넣으면 무한 중첩이 가능합니다.

### 버전 플래그

```rust
Command::new("app")
    .version("2.3.1")   // --version / -V 자동 활성화
    .on_run(|_| {})
    .execute()
    .unwrap();
```

```bash
$ app --version
app 2.3.1
```

---

## 플래그

### 플래그 타입

| `FlagValue` 변형 | Rust 타입 | 조회 메서드 |
| ---------------- | --------- | ----------- |
| `Bool(bool)` | `bool` | `get_bool("name")` |
| `String(String)` | `String` | `get_string("name")` |
| `Int(i64)` | `i64` | `get_int("name")` |
| `Float(f64)` | `f64` | `get_float("name")` |
| `StringVec(Vec<String>)` | `Vec<String>` | `get_string_vec("name")` |
| `IntVec(Vec<i64>)` | `Vec<i64>` | raw `FlagValue` 사용 |

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
    })
    .execute()
    .unwrap();
```

### 숏 플래그 & 파싱 문법

```rust
Flag::new("output", FlagValue::String(String::new()), "output file").short('o')
```

지원하는 파싱 문법:

```bash
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

제공되지 않으면 `WrCliError::MissingRequiredFlag`를 반환합니다.

### Persistent 플래그

루트(또는 중간) 커맨드에 등록하면 모든 하위 서브커맨드에서 자동으로 사용할 수 있습니다.

```rust
Command::new("app")
    .persistent_flag(
        Flag::new("config", FlagValue::String(String::new()), "config file path").short('c'),
    )
    .subcommand(
        Command::new("serve").on_run(|ctx| {
            let cfg_path = ctx.flags.get_string("config").unwrap_or("config.toml");
        }),
    )
    .execute()
    .unwrap();
```

### 반복 가능한 벡터 플래그

`StringVec` / `IntVec` 플래그는 같은 이름을 여러 번 지정해 값을 누적합니다.

```rust
Command::new("app")
    .flag(Flag::new("tag", FlagValue::StringVec(vec![]), "add a tag (repeatable)"))
    .on_run(|ctx| {
        let tags = ctx.get_string_vec("tag").unwrap_or_default();
        for tag in &tags { println!("tag: {}", tag); }
    })
    .execute()
    .unwrap();
```

```bash
$ app --tag frontend --tag prod --tag v2
tag: frontend
tag: prod
tag: v2
```

---

## 포지셔널 인수 검증

`wrcli::args` 모듈에 내장 validator가 있습니다.

```rust
use wrcli::args::{no_args, arbitrary_args, exact_args,
                  minimum_n_args, maximum_n_args, range_args, valid_args};

Command::new("copy")
    .args(exact_args(2))
    .on_run(|ctx| {
        let src = &ctx.args[0];
        let dst = &ctx.args[1];
    })
```

| 함수 | 설명 |
| ---- | ---- |
| `no_args()` | 포지셔널 인수 없음 |
| `arbitrary_args()` | 제한 없음 |
| `exact_args(n)` | 정확히 n개 |
| `minimum_n_args(n)` | 최소 n개 |
| `maximum_n_args(n)` | 최대 n개 |
| `range_args(min, max)` | min 이상 max 이하 |
| `valid_args(vec![...])` | 허용 목록에 포함된 값만 |

커스텀 validator:

```rust
use wrcli::args::ArgValidator;
use wrcli::error::WrCliError;

fn only_existing_files() -> ArgValidator {
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

```bash
persistent_pre_run  (루트 → 리프 순서로 체인)
pre_run             (매칭된 리프 커맨드만)
run / run_e         (매칭된 리프 커맨드만)
post_run            (매칭된 리프 커맨드만)
persistent_post_run (리프 → 루트 순서로 체인)
```

`on_run_e`에서 `Err`를 반환하면 `post_run` / `persistent_post_run`은 실행되지 않습니다.

```rust
Command::new("app")
    .on_persistent_pre_run(|_| println!("항상 실행: 초기화"))
    .subcommand(
        Command::new("deploy")
            .on_pre_run(|_| println!("배포 전 검증"))
            .on_run_e(|_| {
                do_deploy()?;
                Ok(())
            })
            .on_post_run(|_| println!("배포 완료 알림")),
    )
    .on_persistent_post_run(|_| println!("항상 실행: 정리"))
    .execute()
    .unwrap();
```

---

## 설정(Config)

Go의 Viper에 해당하는 설정 스토어입니다. `.with_config(config)`로 루트 커맨드에 연결하면 모든 서브커맨드의 `ctx.config`로 접근할 수 있습니다.

### 기본값

```rust
let config = Config::new()
    .set_default("server.host", "127.0.0.1")
    .set_default("server.port", 8080i64)
    .set_default("debug",       false);
```

### 설정 파일

```rust
let mut config = Config::new()
    .set_config_name("myapp")          // 파일명 (확장자 제외)
    .set_config_type("toml")           // "toml" | "json" | "yaml"
    .add_config_path(".")              // 검색 디렉토리 (여러 개 가능)
    .add_config_path("~/.config/myapp");

config.read_in_config().ok();          // 파일 없어도 무시
```

TOML 예시:

```toml
[server]
host = "0.0.0.0"
port = 9000

[database]
url = "postgres://localhost/mydb"
```

중첩 키에는 점 표기법으로 접근합니다: `config.get_string("server.host")`.

### 설정 파일 자동 탐지

`config_type`을 지정하지 않으면 **모든 지원 형식**(TOML, JSON, YAML)을 순서대로
시도합니다. 또 검색 경로를 직접 지정하지 않아도 표준 위치를 자동으로 탐지합니다.

**검색 순서** (Viper 스타일):

```text
1. add_config_path로 추가한 경로 (지정한 경우)
2. $XDG_CONFIG_HOME/<name>  또는  ~/.config/<name>
3. ~/.<name>
4. 현재 디렉토리 (.)
```

```rust
let mut config = Config::new()
    .set_config_name("myapp");   // 타입 미지정 → 자동 판별, 경로 미지정 → 자동 탐지

config.read_in_config().ok();    // ~/.config/myapp/{toml,json,yaml} 등에서 검색
```

**단일 파일 직접 지정** — `set_config_file`:

```rust
let mut config = Config::new()
    .set_config_file("~/.config/myapp/custom.toml");   // 확장자에서 포맷 자동 판별

config.read_in_config()?;
```

`set_config_file`은 설정 이름/타입/검색 경로와 무관하게 그 경로에서 바로 로드합니다.

### Config ↔ Flag 자동 바인딩

명시적으로 설정되지 않은 플래그는 **설정 저장소의 값으로 자동 시드**됩니다.
즉, 설정 파일/환경변수/기본값에서 값을 찾아 플래그에 주입하므로
`ctx.flags.get_*()`와 `ctx.get_*()`가 일관된 값을 반환합니다.

```rust
// 설정에 server.port = 9000 이 있는 경우
Command::new("app")
    .flag(Flag::new("port", FlagValue::Int(0), "port"))
    .with_config(config)              // 설정 로드 완료 상태
    .on_run(|ctx| {
        // 플래그를 명시적으로 안 줬다면 9000이 들어옴
        let port = ctx.flags.get_int("port").unwrap();
    })
    .execute_with(vec![])             // "--port 5000" 같은 명시적 입력은 우선
    .unwrap();
```

**우선순위**: 명시적 CLI 플래그 > 설정(파일/환경변수/기본값). 플래그가 타입이
맞지 않는 설정값과 충돌하면 설정값은 무시됩니다.

### 환경 변수

```rust
let config = Config::new()
    .automatic_env()
    .set_env_prefix("MYAPP");
```

`automatic_env()` + `set_env_prefix("MYAPP")` 적용 시:

| 설정 키 | 환경 변수 |
| ------- | --------- |
| `server.port` | `MYAPP_SERVER_PORT` |
| `database.url` | `MYAPP_DATABASE_URL` |
| `debug` | `MYAPP_DEBUG` |

특정 키를 환경 변수에 명시적으로 연결:

```rust
config.bind_env("token", "API_TOKEN");
```

### 우선순위 규칙

```text
1. 기본값        (set_default)               ← 가장 낮음
2. 설정 파일     (read_in_config)
3. 환경 변수     (automatic_env / bind_env)
4. CLI 플래그    (사용자가 실제로 입력한 경우) ← 가장 높음
```

CLI 플래그 기본값은 주입되지 않습니다. 사용자가 실제로 지정한 값만 설정을 덮어씁니다.

### 설정 조회

```rust
ctx.config.get_string("server.host")      // Option<String>
ctx.config.get_int("server.port")         // Option<i64>
ctx.config.get_bool("debug")              // Option<bool>
ctx.config.get_float("ratio")             // Option<f64>
ctx.config.get_string_vec("allowed.ips")  // Option<Vec<String>>
```

---

## CommandContext

`on_run` / `on_run_e` 콜백이 받는 `&CommandContext`는 플래그, 설정, 포지셔널 인수를 묶어서 제공합니다.

```rust
.on_run(|ctx| {
    // 포지셔널 인수
    let first = &ctx.args[0];

    // 플래그만 조회
    let verbose = ctx.flags.get_bool("verbose").unwrap_or(false);

    // 플래그 → 설정 순으로 자동 탐색
    let host = ctx.get_string("server.host").unwrap_or_default();
    let port = ctx.get_int("server.port").unwrap_or(8080);

    // 현재 커맨드 경로 (예: ["myapp", "config", "get"])
    println!("{}", ctx.command_name());
    println!("{:?}", ctx.command_path);
})
```

`ctx.get_*(key)`는 플래그명과 설정 키가 같을 때 편리하게 사용할 수 있습니다.

---

## Completion 스크립트 생성

`Command::gen_completion(shell)`로 bash / zsh / fish용 자동완성 스크립트를
생성합니다. 서브커맨드와 플래그를 재귀적으로 수집합니다.

```rust
let bash_script = Command::new("myapp")
    .subcommand(Command::new("serve"))
    .gen_completion("bash")
    .unwrap();

std::fs::write("myapp.bash", bash_script)?;
```

지원 셸: `"bash"`, `"zsh"`, `"fish"`. 그 외 셸은
`WrCliError::UnsupportedCompletionShell`을 반환합니다.

```bash
$ myapp gen-completion bash > /etc/bash_completion.d/myapp
```

---

## 에러 처리

`.execute()`는 `Result<(), WrCliError>`를 반환합니다.

| 변형 | 발생 시점 |
| ---- | --------- |
| `UnknownFlag` | 미등록 플래그 사용 |
| `UnknownSubcommand` | 미등록 서브커맨드 사용 |
| `MissingRequiredFlag` | `.required()` 플래그 미입력 |
| `InvalidFlagValue` | 타입 불일치 (예: `--count abc`) |
| `ArgValidationFailed` | 포지셔널 인수 검증 실패 |
| `CommandHasNoRunner` | `on_run` 미등록 커맨드 실행 |
| `ConfigFileNotFound` | 설정 파일을 찾을 수 없음 |
| `ConfigParseError` | 설정 파일 파싱 실패 |
| `UserError` | `on_run_e`에서 반환한 에러 |
| `UnsupportedCompletionShell` | 지원하지 않는 셸로 completion 생성 |

권장 패턴:

```rust
fn main() {
    if let Err(e) = build_cli().execute() {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}
```

임의의 에러 타입을 `on_run_e`에서 반환하려면 `WrCliError::user(e)` 사용:

```rust
.on_run_e(|_| {
    let data = std::fs::read_to_string("data.txt")
        .map_err(WrCliError::user)?;
    Ok(())
})
```

---

## 테스트 작성

### 단위 테스트 — `execute_with()`

```rust
#[test]
fn test_greet_command() {
    use std::sync::{Arc, Mutex};

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

### 바이너리 테스트 — `assert_cmd`

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
| ---- | :---------: | ---- |
| `toml-config` | ✅ | TOML 설정 파일 지원 |
| `json-config` | ✅ | JSON 설정 파일 지원 |
| `yaml-config` | ❌ | YAML 설정 파일 지원 (`noyalib`) |

```toml
# 모든 형식 활성화
wrcli = { version = "0.1", features = ["yaml-config"] }

# 최소 빌드 (설정 파일 지원 없음)
wrcli = { version = "0.1", default-features = false }
```

스타일(`Style`, `Table`, `Panel`, `Rule`, `Tree`, `Text`, `Progress` 등)은
기본으로 제공되며 별도 피처가 필요 없습니다. 사용법은
[STYLE.md](STYLE.md)를 참고하세요.

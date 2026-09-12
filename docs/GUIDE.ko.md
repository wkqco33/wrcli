# wrcli 상세 가이드

[English](GUIDE.md) | [한국어](GUIDE.ko.md)

---

## 목차

- [설치](#설치)
- [커맨드](#커맨드)
- [플래그](#플래그)
- [Help 규약 (clig.dev)](#help-규약-cligdev)
- [표준 플래그와 출력 포맷](#표준-플래그와-출력-포맷)
- [대화형 입력과 확인 프롬프트](#대화형-입력과-확인-프롬프트)
- [민감 플래그](#민감-플래그)
- [선택적 값 플래그](#선택적-값-플래그)
- [표준 입출력 대체](#표준-입출력-대체)
- [색상 정책과 페이저](#색상-정책과-페이저)
- [Ctrl-C(SIGINT) 처리](#ctrl-csigint-처리)
- [숨김 · Deprecated · 플래그 제약](#숨김--deprecated--플래그-제약)
- [포지셔널 인수 검증](#포지셔널-인수-검증)
- [라이프사이클 훅](#라이프사이클-훅)
- [설정(Config)](#설정config)
  - [기본값](#기본값)
  - [설정 파일](#설정-파일)
  - [설정 파일 자동 탐지](#설정-파일-자동-탐지)
  - [Config ↔ Flag 자동 바인딩](#config--flag-자동-바인딩)
  - [환경 변수](#환경-변수)
  - [우선순위 규칙](#우선순위-규칙)
  - [명시 값과 별칭](#명시-값과-별칭)
  - [키 구분자와 env 설정](#키-구분자와-env-설정)
  - [설정 조회](#설정-조회)
  - [열거와 맵 조회](#열거와-맵-조회)
  - [런타임 읽기와 병합](#런타임-읽기와-병합)
  - [설정 저장](#설정-저장)
  - [구조체 역직렬화](#구조체-역직렬화)
  - [설정 파일 감시](#설정-파일-감시)
- [CommandContext](#commandcontext)
- [Completion 스크립트 생성](#completion-스크립트-생성)
  - [동적 completion](#동적-completion)
- [에러 처리](#에러-처리)
- [테스트 작성](#테스트-작성)
- [피처 플래그](#피처-플래그)

---

## 설치

```toml
[dependencies]
wrcli = "0.4"

# YAML도 필요한 경우
wrcli = { version = "0.4", features = ["yaml-config"] }

# 설정 파일 지원 없이 최소 빌드
wrcli = { version = "0.4", default-features = false }
```

### Git 저장소 직접 참조

```toml
# SSH (권장)
wrcli = { git = "git@github.com:wkqco33/wrcli.git" }

# 브랜치 / 태그 / 커밋 고정
wrcli = { git = "git@github.com:wkqco33/wrcli.git", tag = "v0.4.0" }
wrcli = { git = "git@github.com:wkqco33/wrcli.git", rev = "a1b2c3d" }

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

### usage 힌트

`.usage_args("<name>")`로 `--help`의 usage 줄에 포지셔널 인수 힌트를 표시합니다.

```rust
Command::new("greet")
    .usage_args("<name>")
    .on_run(|_| {})
    .execute()
    .unwrap();
```

```text
Usage:
  app greet <name> [flags]
```

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

`.comma_separated()`를 붙이면 `--tag a,b,c`를 여러 값으로 분리합니다(공백 제거,
빈 항목 무시). 기본은 분리하지 않습니다.

```rust
Flag::new("tag", FlagValue::StringVec(vec![]), "tags").comma_separated()
```

```bash
$ app --tag frontend,prod --tag=v2
tag: frontend
tag: prod
tag: v2
```

### 부모 로컬 플래그 (서브커맨드 앞)

`persistent`가 아닌 부모 커맨드의 플래그도 **서브커맨드 이름 앞에** 지정하면
부모가 소비하고, 그 값은 리프 컨텍스트에서도 읽을 수 있습니다. 서브커맨드 이름
뒤에 지정한 부모 로컬 플래그는 알 수 없는 플래그로 처리됩니다.

```rust
Command::new("app")
    .flag(Flag::new("profile", FlagValue::String(String::new()), "profile name"))
    .subcommand(Command::new("deploy").on_run(|ctx| {
        // `app --profile prod deploy` → "prod"
        let profile = ctx.flags.get_string("profile").unwrap_or("dev");
        println!("deploying with {profile}");
    }))
    .execute()
    .unwrap();
```

```bash
app --profile prod deploy   # OK
app deploy --profile prod   # 오류: 알 수 없는 플래그
```

부모 로컬 플래그는 help/completion 목록에 상속되지 않으므로, 리프의 `Flags`
섹션에는 나타나지 않습니다. `--help`/`--version`이 포함되면 부모는 플래그를
소비하지 않고 리프가 그대로 처리합니다.

---

## Help 규약 (clig.dev)

### 내장 `help` 서브커맨드

`help`를 직접 서브커맨드로 등록하지 않았다면 내장 `help`가 자동으로 동작한다.
사용자가 `help`를 등록하면 내장 동작은 비활성화된다.

```sh
myapp help                # 루트 도움말
myapp help config         # 서브커맨드 도움말 (별칭도 가능)
myapp help config get     # 중첩 경로
```

경로 중간에 없은 이름을 주면 제안(`Did you mean`)과 함께 `UnknownSubcommand` 오류를 낸다.
completion 스크립트에도 `help`가 후보로 포함된다.

### 예제 · 지원 링크 · 문서 링크

clig.dev는 “예제를 앞에 두고”, 피드백 경로와 웹 문서 링크를 도움말에 넣으라고 권한다.
예제는 Usage 바로 다음에 출력된다.

```rust
Command::new("myapp")
    .short("My CLI")
    .example("myapp greet Alice")
    .example("myapp greet Bob --upper --count 3")
    .support_url("https://github.com/me/myapp/issues")
    .docs_url("https://docs.example.com/{command}") // {command} 치환
```

`docs_url`/`support_url`은 하위 커맨드가 직접 정의하지 않으면 상속된다.
따라서 `myapp greet --help`는 `https://docs.example.com/myapp greet` 링크를 보여 준다.

### 러너 없는 커맨드

서브커맨드만 가진 상위 커맨드를 인자 없이 실행하면 도움말을 출력하고 **성공(종료 코드 0)** 으로 끝난다.
서브커맨드도 러너도 없는 리프 커맨드는 실수가 많으므로 `CommandHasNoRunner` 오류를 낸다.
리프에서도 도움말만 보여주고 성공으로 끝내려면 `help_on_missing_runner()`를 쓴다.

```rust
Command::new("myapp").short("My CLI").help_on_missing_runner()
```

### 버그 리포트 URL

`bug_report_url`을 설정하면 `execute_or_exit()`이 사용법 오류가 아닌 예상 밖 오류에서
해당 URL을 안내한다.

```rust
Command::new("myapp").bug_report_url("https://github.com/me/myapp/issues/new")
```

---

## 표준 플래그와 출력 포맷

`standard_flags()`는 clig.dev가 반복해 언급하는 관례를 한 번에 등록한다.
persistent 플래그이므로 모든 서브커맨드에 전파되고 `app --plain list`와 `app list --plain`이 모두 동작한다.

| 플래그 | 단축 | 접근자 |
| ---- | ---- | ---- |
| `--quiet` | `-q` | `ctx.is_quiet()` |
| `--force` | `-f` | `ctx.is_force()` |
| `--no-input` | | `ctx.no_input()` |
| `--no-color` | | |
| `--plain` | | `ctx.is_plain()` |
| `--json` | | `ctx.is_json()` |
| `--color <when>` | | `style::color_choice()` |
| `--confirm <name>` | | `ctx.confirm_severe()` |

`--plain`과 `--json`은 `mutually_exclusive`로 검증된다.

```rust
use wrcli::{Command, OutputFormat};

Command::new("myapp")
    .standard_flags()
    .on_run(|ctx| {
        match ctx.output_format() {
            OutputFormat::Human => { /* styled table */ }
            OutputFormat::Plain => { /* 한 줄에 레코드 하나 */ }
            OutputFormat::Json => { /* JSON */ }
        }
    });
```

테이블은 `Table::render_plain()`으로 테두리 없이 탭 구분 한 줄/레코드로 출력할 수 있다.

```rust
use wrcli::style::Table;
let tsv = Table::new()
    .headers(["Name", "Version"])
    .row(["wrcli", "0.4.0"])
    .render_plain();
assert_eq!(tsv, "Name\tVersion\nwrcli\t0.4.0\n");
```

---

## 대화형 입력과 확인 프롬프트

clig.dev: “Never require a prompt”, “Only use prompts if stdin is an interactive terminal”,
“Confirm before doing anything dangerous”.

```rust
.on_run_e(|ctx| {
    if ctx.confirm("Delete 3 items?")? {
        // --force가 있거나 사용자가 y를 입력한 경우
    }
    ctx.confirm_severe("myapp")?;   // --confirm="myapp" 또는 이름 직접 입력
    let pw = ctx.prompt_password("Password: ")?; // unix: stty -echo
    Ok(())
})
```

동작 규칙:

- `--force`(`-f`)가 있으면 `confirm()`은 프롬프트 없이 `true`.
- `--confirm="<name>"`이 일치하면 `confirm_severe()`가 `true`. 값이 다르면 `ConfirmationFailed`.
- `--no-input`이거나 stdin이 TTY가 아니면 프롬프트 대신 `InteractiveInputRequired` 오류를 내고,
  대신 사용할 플래그(`--force`, `--confirm="<name>"`)를 안내한다. (`cat`처럼 hang 하지 않음)
- `ctx.is_interactive()`로 프롬프트 가능 여부를 직접 확인할 수 있다.
- `prompt_password()`는 unix에서 `stty -echo`로 입력을 가린다. 다른 플랫폼에서는 echo를 끌 수 없다.

---

## 민감 플래그

clig.dev는 비밀을 플래그로 직접 받지 말 것을 권장한다. 부득이한 경우 최소한
값이 도움말·오류 메시지로 새지 않도록 `sensitive()`를 붙인다.

```rust
Flag::new("token", FlagValue::String(String::new()), "API token").sensitive()
```

- 도움말에서 기본값 표시를 생략한다.
- 값 변환 오류 메시지에서 값을 `***`로 가린다.

비밀은 `--password-file`이나 stdin으로 받는 것을 권장한다.

---

## 선택적 값 플래그

값이 선택적인 플래그는 특수 단어 `none`을 “값 없음”(빈 문자열)으로 해석한다.
clig.dev: “allow a special word like 'none'. Don't just use a blank value.”

```rust
Flag::new("config", FlagValue::String("/etc/app.toml".to_owned()), "config path").optional_value()
// --config /tmp/x.toml  -> "/tmp/x.toml"
// --config none        -> ""
```

---

## 표준 입출력 대체

clig.dev: “If input or output is a file, support `-` to read from stdin or write to stdout.”

```rust
use wrcli::io::{open_reader, open_writer, read_to_string};
use std::io::{Read, Write};

fn cat(path: &str) -> wrcli::Result<()> {
    let mut buf = String::new();
    open_reader(path)?.read_to_string(&mut buf)?;   // "-" -> stdin
    open_writer("-")?.write_all(buf.as_bytes())?;   // "-" -> stdout
    Ok(())
}
```

`read_to_string(path)`는 `-`면 stdin에서 전체를 읽는다.

---

## 색상 정책과 페이저

색상 사용 여부는 clig.dev 규칙을 따른다. 우선순위는 다음과 같다.

1. 전역 override (`--no-color` = Never, `--color=always|never|auto`)
2. `FORCE_COLOR`(비어 있지 않음)
3. `NO_COLOR`(비어 있지 않음)
4. `TERM=dumb`
5. 앱 전용 `*_NO_COLOR`
6. stdout/stderr이 TTY인지 여부

```rust
use wrcli::style::{ColorChoice, set_color_choice, set_no_color_env, stdout_is_styled};

set_no_color_env(Some("MYAPP_NO_COLOR"));
set_color_choice(ColorChoice::Never);
assert!(!stdout_is_styled());
```

`stdout_is_terminal()` / `stdin_is_terminal()`로 색상과 무관한 TTY 여부를 확인할 수 있다.

긴 출력은 `style::pager::page(&text)`로 넘기면 stdout이 TTY일 때만 `PAGER`(기본 `less -FIRX`)로
보내고, 파이프/CI에서는 그대로 출력한다.

```rust
wrcli::style::pager::page(&long_text)?;
```

`Progress::draw()` / `Progress::finish()`도 비TTY에서는 애니메이션(`\r`)을 쓰지 않고
마지막 상태만 한 줄로 출력한다.

---

## Ctrl-C(SIGINT) 처리

`signal` 피처를 켜면 Ctrl-C 시 즉시 메시지를 출력하고 종료 코드 `130`으로 끝낼 수 있다.
핸들러는 async-signal-safe 연산만 수행하며 정리(clean-up) 작업을 하지 않는다 (crash-only).

```toml
wrcli = { version = "0.4", features = ["signal"] }
```

```rust
Command::new("myapp")
    .interrupt_message("interrupted\n")
    .on_run(|_| { /* 장시간 작업 */ });
```

피처 없이 사용하려면 `wrcli::signal::install(msg)`를 직접 호출해도 된다.

---

## 숨김 · Deprecated · 플래그 제약

### 숨김 (hidden)

`.hidden()`이 붙은 커맨드와 플래그는 `--help`와 completion 스크립트에서
빠지지만, 파싱과 실행은 그대로 동작합니다. 내부용·실험적 기능에 사용하세요.

```rust
Command::new("app")
    .flag(Flag::new("internal", FlagValue::Bool(false), "internal use only").hidden())
    .subcommand(Command::new("secret").hidden().on_run(|_| println!("secret")))
    .execute()
    .unwrap();
```

오타 제안(`Did you mean`)에도 숨겨진 항목은 포함되지 않습니다.

### Deprecated

`.deprecated("메시지")`를 지정하면 해당 커맨드/플래그가 실제로 사용될 때
stderr로 경고가 출력됩니다. 실행 자체는 계속됩니다.

```rust
Command::new("app")
    .flag(Flag::new("old", FlagValue::Bool(false), "legacy").deprecated("use --new"))
    .subcommand(Command::new("legacy").deprecated("use `app new`").on_run(|_| {}))
    .execute()
    .unwrap();
```

```bash
$ app legacy
Command "legacy" is deprecated: use `app new`
```

### 플래그 제약 그룹

세 가지 제약을 선언하면 리프 커맨드 실행 직전에 검증됩니다. 설정에서 시드된
값은 "사용자가 지정한 것"으로 치지 않으므로 제약을 발동시키지 않습니다.

```rust
Command::new("app")
    .flag(Flag::new("json", FlagValue::Bool(false), "JSON output"))
    .flag(Flag::new("yaml", FlagValue::Bool(false), "YAML output"))
    .flag(Flag::new("user", FlagValue::String(String::new()), "user"))
    .flag(Flag::new("pass", FlagValue::String(String::new()), "password"))
    .mutually_exclusive(&["json", "yaml"])   // 둘 다 지정하면 오류
    .required_together(&["user", "pass"])    // 일부만 지정하면 오류
    .one_required(&["json", "yaml"])         // 최소 하나 필요
    .on_run(|_| {})
    .execute()
    .unwrap();
```

| 위반 시 | 에러 변형 |
| ------- | --------- |
| 둘 이상 지정 | `MutuallyExclusiveFlags` |
| 일부만 지정 | `RequiredFlagsTogether` |
| 하나도 미지정 | `OneFlagRequired` |

### 오타 제안 (Did you mean)

미등록 커맨드/플래그에는 편집 거리 기반 후보가 에러 메시지에 포함됩니다.
`.suggest_for("별칭")`로 실행되지 않는 **제안 전용** 이름을 추가할 수 있습니다
(Cobra의 `SuggestFor`).

```rust
Command::new("app")
    .subcommand(
        Command::new("remove")
            .suggest_for("delete")   // `app delete` → "Did you mean: remove"
            .on_run(|_| {}),
    )
    .execute()
    .unwrap();
```

숨김(`hidden`) 항목은 제안 대상에서 제외됩니다.

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
    .set_config_type("toml")           // toml | json | yaml | ini | env(dotenv) | properties
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

`config_type`을 지정하지 않으면 **활성화된 모든 지원 형식**(TOML, JSON, YAML, INI,
dotenv, properties)을 순서대로 시도합니다. 또 검색 경로를 직접 지정하지 않아도
표준 위치를 자동으로 탐지합니다.

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

config.read_in_config().ok();    // ~/.config/myapp/{toml,json,yaml,ini,...} 등에서 검색
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
4. CLI 플래그    (사용자가 실제로 입력한 경우)
5. 명시 값       (set)                       ← 가장 높음
```

CLI 플래그 기본값은 주입되지 않습니다. 사용자가 실제로 지정한 값만 설정을 덮어씁니다.
`set`은 플래그보다 우선하며, `is_set(key)`로 어느 레이어든 값이 있는지 확인할 수 있습니다.

### 명시 값과 별칭

```rust
let config = Config::new()
    .set_default("server.port", 8080i64)
    .set("server.port", 9000i64)          // 최우선 레이어
    .register_alias("port", "server.port"); // "port"로도 조회

assert!(config.is_set("port"));
assert_eq!(config.get_int("port"), Some(9000));
```

별칭은 체인으로 연결할 수 있습니다(`a` → `b` → 정규 키).

### 키 구분자와 env 설정

```rust
let config = Config::new()
    .set_key_delimiter('/')                  // "server/port"로 중첩 접근
    .set_env_key_replacer(&[(".", "__")])   // 대문자 키에 순서대로 적용
    .allow_empty_env(false);                 // 빈 환경변수를 미설정으로 취급 (Viper 기본)
```

기본값: 구분자 `'.'`, replacer `.`/`-` → `_`, `allow_empty_env = true`(빈 값도 사용).

### 설정 조회

```rust
ctx.config.get_string("server.host")      // Option<String>
ctx.config.get_int("server.port")         // Option<i64> (get_int64 동일)
ctx.config.get_uint("workers")            // Option<u64>
ctx.config.get_bool("debug")              // Option<bool>
ctx.config.get_float("ratio")             // Option<f64>
ctx.config.get_string_vec("allowed.ips")  // Option<Vec<String>> (get_string_slice 동일)
ctx.config.get_duration("timeout")        // Option<Duration> — "1h30m", "250ms", 숫자=초
ctx.config.get_time("started_at")         // Option<SystemTime> — RFC3339 또는 Unix 초
ctx.config.get_size_in_bytes("max_body")  // Option<u64> — "1.5MB", "2GiB" (1024 기반)
```

### 열거와 맵 조회

```rust
let keys = ctx.config.all_keys();          // Vec<String>, 정렬됨 (Viper AllKeys)
let tree = ctx.config.all_settings();      // SettingsMap — 점 키를 중첩으로 재구성

// 키 하위 값을 맵으로
let server = ctx.config.get_string_map_string("server");      // BTreeMap<String, String>
let ips = ctx.config.get_string_map_string_slice("allowed");  // BTreeMap<String, Vec<String>>

// 하위 트리만 담은 Config
let sub = ctx.config.sub("server");
let host = sub.get_string("host");
```

### 런타임 읽기와 병합

```rust
let mut cfg = Config::new().set_config_type("toml");
cfg.read_config("[a]\nb = 1\n".as_bytes())?;   // 파일 레이어 교체
cfg.merge_in_config("extra.toml")?;            // 기존 키를 유지하며 병합
cfg.merge_config_map([("c".to_owned(), ConfigValue::Int(3))]);
```

`read_config`는 [`set_config_type`](Config::set_config_type)으로 포맷을 먼저 지정해야 하며,
미지정 시 `WrCliError::ConfigTypeNotSet`을 반환합니다.

### 설정 저장

```rust
// 모든 레이어를 병합한 현재 설정을 파일로 저장 (확장자로 포맷 판별)
ctx.config.write_config_as("out.toml")?;

// 대상 파일이 이미 있으면 실패
ctx.config.safe_write_config_as("out.json")?; // Err(ConfigFileExists)
```

쓰기 지원 포맷: TOML, JSON, INI, dotenv, properties (해당 피처 활성화 시).

### 구조체 역직렬화

`serde` 피처를 켜면 설정을 구조체로 바로 매핑할 수 있습니다.

```rust
#[derive(serde::Deserialize)]
struct Server {
    host: String,
    port: i64,
}

let server: Server = ctx.config.unmarshal_key("server")?; // 하위 트리
let app: App = ctx.config.unmarshal()?;                    // 전체 트리
```

지원: struct/map, `Vec`, `Option`, 원시 타입, unit enum variant.

### 설정 파일 감시

```rust
let mut cfg = Config::new()
    .set_config_file("app.toml")
    .set_watch_interval(Duration::from_millis(500))
    .on_config_change(|reloaded| {
        println!("port = {:?}", reloaded.get_int("port"));
    });
cfg.read_in_config()?;

let _watcher = cfg.watch_config()?; // drop하면 감시 중단
```

폴링 기반(기본 1초)이며 `on_config_change`와 `read_in_config`가 선행되어야 합니다.
미충족 시 `WrCliError::ConfigWatchNotReady`를 반환합니다.

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

    // 타입 변환 getter (플래그 → 설정)
    let _limit = ctx.get_uint("limit");
    let _timeout = ctx.get_duration("timeout");
    let _max_size = ctx.get_size_in_bytes("max_size");
    let _nums = ctx.get_int_vec("nums");
    let _server = ctx.get_string_map("server");

    // 플래그로 명시 지정되었거나 설정에 존재하는지
    let explicit = ctx.is_set("port");

    // 현재 커맨드 경로 (예: ["myapp", "config", "get"])
    println!("{}", ctx.command_name());
    println!("{:?}", ctx.command_path);
    let _ = explicit;
})
```

`ctx.get_*(key)`는 플래그명과 설정 키가 같을 때 편리하게 사용할 수 있습니다.
`get_duration` / `get_time` / `get_size_in_bytes` / `get_string_map`은 플래그에
해당 타입이 없으면 설정에서 조회합니다. `is_set`은 argv로 명시된 플래그 또는
설정에 존재하는 키에 대해 `true`를 반환합니다.

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

설치용 서브커맨드(`gen-completion` 등)는 내장되어 있지 않습니다. 런타임에
생성하려면 커맨드 트리를 만든 뒤 `gen_completion`을 호출하는 서브커맨드를
직접 등록하세요.

```bash
# 직접 만든 서브커맨드가 스크립트를 stdout으로 출력하는 경우
myapp gen-completion bash > /etc/bash_completion.d/myapp
```

### 동적 completion

`gen_completion`은 커맨드 트리를 정적으로 박아 넣습니다. 설정·파일·서버 상태에
따라 후보가 달라져야 하면 동적 API를 사용하세요.

- `Command::complete(&[String])` — 마지막 토큰(완성 중)에 대한 후보를 반환.
  서브커맨드·플래그·`arg_candidates` 후보를 자동으로 수집하고 prefix로 필터링합니다.
- `Command::completion_request(Vec<String>)` — 첫 토큰이 `__complete`이면 후보를
  `Some`으로 반환하는 `main` 진입점 헬퍼.
- `Command::arg_candidates(f)` — 포지셔널 인수 후보를 만드는 함수 등록.

```rust
fn main() {
    let cmd = Command::new("myapp")
        .subcommand(
            Command::new("run")
                .arg_candidates(|prior| {
                    if prior.is_empty() {
                        vec!["build".into(), "test".into(), "deploy".into()]
                    } else {
                        Vec::new()
                    }
                })
                .on_run(|_| {}),
        );

    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Some(candidates) = cmd.completion_request(args) {
        for c in candidates {
            println!("{}", c);
        }
        return;
    }

    cmd.execute().unwrap();
}
```

```bash
$ myapp __complete run ""
build
test
deploy
```

생성된 스크립트에서 이 프로토콜을 호출하도록 바꾸려면 `COMPREPLY`를
`"$1" __complete "${COMP_WORDS[@]:1}"` 결과로 채우면 됩니다.

---

## 에러 처리

`.execute()`는 `Result<(), WrCliError>`를 반환합니다.

| 변형 | 발생 시점 |
| ---- | --------- |
| `UnknownFlag` | 미등록 플래그 사용 |
| `UnknownSubcommand` | 미등록 서브커맨드 사용 |
| `MissingRequiredFlag` | `.required()` 플래그 미입력 |
| `MissingFlagValue` | 값을 요구하는 플래그에 값 미제공 (예: `--name`) |
| `InvalidFlagValue` | 타입 불일치 (예: `--count abc`) |
| `ArgValidationFailed` | 포지셔널 인수 검증 실패 |
| `CommandHasNoRunner` | `on_run` 미등록 커맨드 실행 |
| `ConfigFileNotFound` | 설정 파일을 찾을 수 없음 |
| `ConfigParseError` | 설정 파일 파싱 실패 |
| `ConfigTypeNotSet` | `read_config`에 포맷 미지정 |
| `ConfigFileExists` | `safe_write_config_as` 대상 파일이 이미 존재 |
| `ConfigDeserializeError` | `unmarshal` 역직렬화 실패 |
| `ConfigWatchNotReady` | `watch_config` 선행 조건 미충족 |
| `MutuallyExclusiveFlags` | `mutually_exclusive` 그룹 위반 |
| `RequiredFlagsTogether` | `required_together` 그룹 위반 |
| `OneFlagRequired` | `one_required` 그룹 위반 |
| `UnsupportedConfigFormat` | 활성화되지 않은 설정 포맷 사용 |
| `UserError` | `on_run_e`에서 반환한 에러 |
| `Io` | 설정 파일 읽기 등 I/O 실패 |
| `UnsupportedCompletionShell` | 지원하지 않는 셸로 completion 생성 |

`UnknownFlag` / `UnknownSubcommand`는 편집 거리 기반 오타 제안을 메시지에
포함합니다.

```text
unknown command 'gret' for 'app'  Run with --help for available commands.

Did you mean this?
 greet
```

### 종료 코드

`WrCliError::is_usage_error()`와 `WrCliError::exit_code()`로 분류할 수 있습니다.
사용법 오류(미등록 플래그/커맨드, 필수 플래그 누락, 타입 오류, 제약 위반 등)는
**2**, 그 외 실행 오류는 **1**입니다.

가장 간단한 패턴은 `Command::execute_or_exit()`입니다. 오류를 stderr에
`Error: ...` 형태로 출력하고 알맞은 코드로 종료합니다.

```rust
fn main() {
    build_cli().execute_or_exit(); // 성공 시 0, 오류 시 1 또는 2
}
```

수동으로 제어하려면:

```rust
fn main() {
    if let Err(e) = build_cli().execute() {
        eprintln!("error: {}", e);
        std::process::exit(e.exit_code());
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
| `ini-config` | ❌ | INI 설정 파일 지원 |
| `dotenv-config` | ❌ | `.env` / dotenv 설정 파일 지원 |
| `properties-config` | ❌ | Java properties 설정 파일 지원 |
| `serde` | ❌ | `unmarshal`/`unmarshal_key` 구조체 역직렬화 |
| `signal` | ❌ | Ctrl-C(SIGINT) 핸들러 (`interrupt_message`) |

```toml
# 모든 형식 활성화
wrcli = { version = "0.4", features = ["yaml-config", "ini-config", "dotenv-config", "properties-config"] }

# 최소 빌드 (설정 파일 지원 없음)
wrcli = { version = "0.4", default-features = false }
```

스타일(`Style`, `Table`, `Panel`, `Rule`, `Tree`, `Text`, `Progress` 등)은
기본으로 제공되며 별도 피처가 필요 없습니다. 사용법은
[STYLE.ko.md](/docs/STYLE.ko.md)를 참고하세요.

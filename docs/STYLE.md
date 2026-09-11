# wrcli 스타일 가이드

터미널 출력을 꾸미기 위한 `wrcli::style` 모듈입니다.
rich 라이브러리에서 영감을 받아 색상, 텍스트 속성, 테이블, 패널, 구분선, 트리,
진행률 표시줄 등을 제공합니다.

## 목차

- [시작하기](#시작하기)
- [Color](#color)
- [Style](#style)
- [Text](#text)
- [Table](#table)
- [Panel](#panel)
- [Rule](#rule)
- [Tree](#tree)
- [Progress](#progress)
- [편의 출력 헬퍼](#편의-출력-헬퍼)

---

## 시작하기

```rust
use wrcli::style::{Color, Style, Table, Panel, Rule, Tree, Text, Progress, Align};
```

스타일링은 다음 경우 **자동으로 비활성화**됩니다:

- `NO_COLOR` 환경변수가 설정된 경우
- 출력 스트림이 터미널이 아닌 경우(파이프 등)

모든 렌더링 타입은 `render(styled: bool)`로 문자열을 얻고, `print()`로
stdout에 출력합니다. `styled`는 `stdout_is_styled()`로 감지합니다.

```rust
use wrcli::style::stdout_is_styled;

let table = Table::new().headers(["A", "B"]).row(["1", "2"]);
println!("{}", table.render(stdout_is_styled()));
```

---

## Color

16개 표준 ANSI 색상, 8비트(256색), 24비트 RGB 트루컬러를 지원합니다.

| 이름 | 설명 |
| ---- | ---- |
| `Black` … `White` | 표준 8색 |
| `BrightBlack` … `BrightWhite` | 밝은 8색 |
| `Fixed(u8)` | 8비트(256색) 인덱스 |
| `Rgb(u8, u8, u8)` | 24비트 트루컬러 |

### 문자열에서 파싱 — `Color::from_name`

```rust
use wrcli::style::Color;

let c1 = Color::from_name("red");            // Some(Color::Red)
let c2 = Color::from_name("bright_cyan");    // Some(Color::BrightCyan)
let c3 = Color::from_name("bright cyan");    // 밑줄 대신 공백도 허용
let c4 = Color::from_name("42");             // Some(Color::Fixed(42))
let c5 = Color::from_name("#ff0000");        // Some(Color::Rgb(255, 0, 0))
let c6 = Color::from_name("rgb(0,128,255)"); // Some(Color::Rgb(0, 128, 255))
let c7 = Color::from_name("nope");           // None
```

---

## Style

색상과 텍스트 장식(속성)의 집합입니다. 빌더로 구성 후 `apply(text, styled)`로
ANSI 이스케이프 문자열을 만듭니다.

### 속성

| 메서드 | ANSI | 설명 |
| ------ | :--: | ---- |
| `.fg(Color)` | 30–97 | 전경색 |
| `.bg(Color)` | 40–107 | 배경색 |
| `.bold()` | 1 | 굵게 |
| `.dim()` | 2 | 흐리게 |
| `.italic()` | 3 | 기울임 |
| `.underline()` | 4 | 밑줄 |
| `.blink()` | 5 | 깜빡임 |
| `.reverse()` | 7 | 반전 |
| `.hide()` | 8 | 숨김 |
| `.strikethrough()` | 9 | 취소선 |
| `.overline()` | 53 | 윗줄 |

### 사용 예

```rust
use wrcli::style::{Style, Color};

let style = Style::new()
    .fg(Color::Green)
    .bg(Color::Black)
    .bold()
    .underline();

let out = style.apply("Success", true);   // "\x1b[1;4;32;40mSuccess\x1b[0m"
let plain = style.apply("Success", false); // "Success"  (원본 그대로)
```

`styled = false`이거나 스타일이 비어 있으면 원본 텍스트를 그대로 반환합니다.

---

## Text

서로 다른 스타일의 스팬(span)을 이어붙여 하나의 텍스트로 렌더링합니다.

```rust
use wrcli::style::{Text, Style, Color};

let text = Text::new()
    .plain("Error: ")
    .span("boom", Style::new().fg(Color::Red).bold());

println!("{}", text.render(false)); // "Error: boom"
```

| 메서드 | 설명 |
| ------ | ---- |
| `.plain("...")` | 스타일 없는 스팬 추가 |
| `.span("...", style)` | 주어진 스타일의 스팬 추가 |
| `.plain_styled("...", style)` | `span`의 별칭 |

---

## Table

Unicode 박스 문자로 테두리를 표시하는 테이블입니다.

```rust
use wrcli::style::{Table, Align};

let out = Table::new()
    .headers(["이름", "버전", "설명"])
    .row(["wrcli", "0.1.0", "CLI 프레임워크"])
    .row(["serde", "1.0",  "직렬화"])
    .align(vec![Align::Left, Align::Center, Align::Right])
    .render(false);
```

```
┌──────┬────────┬──────────────────┐
│ 이름 │ 버전   │  설명            │
├──────┼────────┼──────────────────┤
│ wrcli │ 0.1.0  │     CLI 프레임워크 │
│ serde │ 1.0    │     직렬화       │
└──────┴────────┴──────────────────┘
```

| 메서드 | 설명 |
| ------ | ---- |
| `.headers([...])` | 헤더 행 |
| `.row([...])` | 데이터 행 (여러 번 호출) |
| `.align(Vec<Align>)` | 컬럼별 정렬 (`Left`/`Center`/`Right`) |
| `.border(bool)` | 테두리 표시 여부 (기본 `true`) |
| `.header_style(Style)` | 헤더 스타일 |

CJK 문자(한글 등)는 `display_width` 기준으로 2칸으로 계산되어 정렬이 유지됩니다.

---

## Panel

테두리와 선택적 제목이 있는 박스입니다.

```rust
use wrcli::style::{Panel, Style, Color};

let out = Panel::new("배포 완료.\n모든 서비스가 정상입니다.")
    .title("상태")
    .border_style(Style::new().fg(Color::Green))
    .padding(1)
    .width(40)          // 고정 폭 (미지정 시 콘텐츠에 맞춤)
    .render(false);
```

```
╭─ 상태 ─────────────────────────────────────────────╮
│ 배포 완료.                                           │
│ 모든 서비스가 정상입니다.                             │
╰─────────────────────────────────────────────────────╯
```

| 메서드 | 설명 |
| ------ | ---- |
| `.title("...")` | 제목 (선택) |
| `.border_style(Style)` | 테두리 스타일 |
| `.title_style(Style)` | 제목 스타일 |
| `.padding(usize)` | 좌우 패딩 |
| `.width(usize)` | 내부 고정 폭 |

---

## Rule

선택적으로 중앙 제목이 있는 수평 구분선입니다.

```rust
use wrcli::style::{Rule, Style, Color};

let out = Rule::new()
    .title("Configuration")
    .style(Style::new().fg(Color::Yellow))
    .width(60)
    .line_char('─')    // 기본값
    .render(false);
```

| 메서드 | 설명 |
| ------ | ---- |
| `.title("...")` | 중앙 제목 (선택) |
| `.style(Style)` | 선 스타일 |
| `.title_style(Style)` | 제목 스타일 |
| `.width(usize)` | 선 폭 (기본 80) |
| `.line_char(char)` | 선 문자 (기본 `─`) |

---

## Tree

계층 트리를 Unicode 박스 문자로 렌더링합니다.

```rust
use wrcli::style::Tree;

let tree = Tree::new("root")
    .child(Tree::new("child1"))
    .child(
        Tree::new("child2")
            .child(Tree::new("grandchild1"))
            .child(Tree::new("grandchild2")),
    );

println!("{}", tree.render(false));
```

```
root
├── child1
└── child2
    ├── grandchild1
    └── grandchild2
```

| 메서드 | 설명 |
| ------ | ---- |
| `Tree::new("...")` | 노드 생성 |
| `.child(Tree)` | 자식 노드 추가 (여러 번 호출) |
| `.style(Style)` | 이 노드 레이블 스타일 (기본: 청록색) |

---

## Progress

터미널 진행률 표시줄입니다.

```rust
use wrcli::style::{Progress, Style, Color};

let bar = Progress::new(100)
    .progress(42)
    .width(20)
    .label("Downloading")
    .bar_style(Style::new().fg(Color::Green))
    .render(false);
```

```
Downloading [#######-------------]  42%
```

| 메서드 | 설명 |
| ------ | ---- |
| `Progress::new(total)` | 총량으로 생성 |
| `.progress(n)` | 현재 진행량 (0..total) |
| `.width(usize)` | 표시줄 폭 (기본 30) |
| `.label("...")` | 앞에 붙는 라벨 |
| `.bar_style(Style)` | 채워진 부분 스타일 (기본: 녹색) |
| `.filled_char(char)` | 채워진 문자 (기본 `#`) |
| `.empty_char(char)` | 빈 문자 (기본 `-`) |

---

## 편의 출력 헬퍼

`wrcli::style`에 상태 메시지를 빠르게 출력하는 헬퍼가 있습니다.

```rust
use wrcli::style::{print_success, print_error, print_warning, print_info};

print_success("빌드 완료");   // ✓ (녹색)
print_error("작업 실패");     // ✗ (빨간색, stderr)
print_warning("설정 누락");   // ⚠ (노란색)
print_info("처리 중...");     // ℹ (청록색)
```

또한 `stdout_is_styled()`, `stderr_is_styled()`, `display_width(s)`(CJK 2칸
계산) 유틸리티를 제공합니다.

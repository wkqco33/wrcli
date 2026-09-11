use super::value::FlagValue;

/// 단일 플래그 정의 (로컬 플래그와 persistent 플래그 모두 이 타입 사용).
#[derive(Debug, Clone)]
pub struct Flag {
    pub name: String,
    pub short: Option<char>,
    pub usage: String,
    pub default: FlagValue,
    pub required: bool,
    pub persistent: bool,
    /// help/completion에서 숨김 (파싱은 계속 동작).
    pub hidden: bool,
    /// 지정 시 경고 메시지. 파싱되면 stderr로 출력.
    pub deprecated: Option<String>,
    /// 부모 커맨드에서 상속된 persistent 플래그인지 여부 (help 섹션 분리용).
    pub inherited: bool,
    /// `StringVec`/`IntVec` 값을 쉼표로 분리할지 여부 (opt-in).
    pub comma_separated: bool,
}

impl Flag {
    pub fn new(name: &str, default: FlagValue, usage: &str) -> Self {
        Flag {
            name: name.to_owned(),
            short: None,
            usage: usage.to_owned(),
            default,
            required: false,
            persistent: false,
            hidden: false,
            deprecated: None,
            inherited: false,
            comma_separated: false,
        }
    }

    pub fn short(mut self, c: char) -> Self {
        self.short = Some(c);
        self
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    pub fn persistent(mut self) -> Self {
        self.persistent = true;
        self
    }

    /// help와 completion에서 이 플래그를 숨김.
    pub fn hidden(mut self) -> Self {
        self.hidden = true;
        self
    }

    /// deprecated 플래그로 표시. 사용 시 stderr에 경고를 출력.
    pub fn deprecated(mut self, msg: &str) -> Self {
        self.deprecated = Some(msg.to_owned());
        self
    }

    /// `StringVec`/`IntVec`에서 `--tag a,b,c`를 여러 값으로 분리 (기본은 분리 안 함).
    pub fn comma_separated(mut self) -> Self {
        self.comma_separated = true;
        self
    }
}

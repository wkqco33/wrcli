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
}

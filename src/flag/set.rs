use std::collections::HashMap;
use indexmap::IndexMap;

use crate::error::{Result, WrCliError};
use super::definition::Flag;
use super::value::FlagValue;

/// 단일 커맨드의 모든 플래그를 담는 컨테이너. 삽입 순서 보존 (help 출력용).
#[derive(Debug, Default, Clone)]
pub struct FlagSet {
    flags: IndexMap<String, Flag>,
    short_map: HashMap<char, String>,
    values: HashMap<String, FlagValue>,
}

impl FlagSet {
    pub fn new() -> Self {
        Default::default()
    }

    /// 플래그 추가.
    ///
    /// # Panics
    /// 이름 또는 short 문자가 이미 등록된 플래그와 충돌하면 패닉. 조용히 덮어쓰면
    /// 잘못된 커맨드 트리 구성을 런타임까지 숨기게 되므로, 구성 시점에 즉시 실패시킴.
    pub fn add(&mut self, flag: Flag) {
        let name = flag.name.clone();
        assert!(
            !self.flags.contains_key(&name),
            "wrcli: flag \"--{name}\" is already registered"
        );
        if let Some(c) = flag.short {
            assert!(
                !self.short_map.contains_key(&c),
                "wrcli: short flag \"-{c}\" is already registered for \"--{}\"",
                self.short_map[&c]
            );
            self.short_map.insert(c, name.clone());
        }
        self.flags.insert(name, flag);
    }

    /// 이름이 없는 경우에만 추가 (persistent 플래그 주입용).
    pub fn add_if_absent(&mut self, flag: &Flag) {
        if !self.flags.contains_key(&flag.name) {
            self.add(flag.clone());
        }
    }

    /// 이름으로 플래그 정의 조회.
    pub fn get_flag(&self, name: &str) -> Option<&Flag> {
        self.flags.get(name)
    }

    /// short 문자로 등록된 플래그 정의 조회 (서브커맨드 라우팅 dry-run용).
    pub(crate) fn short_flag(&self, c: char) -> Option<&Flag> {
        let name = self.short_map.get(&c)?;
        self.flags.get(name.as_str())
    }

    /// 플래그 값 조회. 없으면 기본값 반환.
    pub fn get(&self, name: &str) -> Option<&FlagValue> {
        self.values
            .get(name)
            .or_else(|| self.flags.get(name).map(|f| &f.default))
    }

    pub fn get_bool(&self, name: &str) -> Option<bool> {
        match self.get(name)? {
            FlagValue::Bool(v) => Some(*v),
            _ => None,
        }
    }

    pub fn get_string(&self, name: &str) -> Option<&str> {
        match self.get(name)? {
            FlagValue::String(v) => Some(v.as_str()),
            _ => None,
        }
    }

    pub fn get_int(&self, name: &str) -> Option<i64> {
        match self.get(name)? {
            FlagValue::Int(v) => Some(*v),
            _ => None,
        }
    }

    pub fn get_float(&self, name: &str) -> Option<f64> {
        match self.get(name)? {
            FlagValue::Float(v) => Some(*v),
            _ => None,
        }
    }

    pub fn get_string_vec(&self, name: &str) -> Option<&[String]> {
        match self.get(name)? {
            FlagValue::StringVec(v) => Some(v.as_slice()),
            _ => None,
        }
    }

    /// 삽입 순서대로 모든 플래그 반복 (help 출력용).
    pub fn flags_iter(&self) -> impl Iterator<Item = &Flag> {
        self.flags.values()
    }

    /// persistent 플래그만 반복 (하위 커맨드 전파용).
    pub fn persistent_flags(&self) -> impl Iterator<Item = &Flag> {
        self.flags.values().filter(|f| f.persistent)
    }

    /// 사용자가 명시적으로 입력한 값만 반복 (기본값 제외).
    /// 디스패치 엔진이 Config 레이어 4 바인딩 시 사용.
    pub(crate) fn values_iter(&self) -> impl Iterator<Item = (&str, &FlagValue)> {
        self.values.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// 플래그 값이 사용자에 의해 명시적으로 설정되었는지 확인.
    pub fn is_set(&self, name: &str) -> bool {
        self.values.contains_key(name)
    }

    /// argv 토큰 파싱. 플래그 아닌 나머지 토큰을 위치 인자로 반환.
    ///
    /// 지원 형식:
    /// - `--name=value`, `--name value`
    /// - `-c value` (그룹 마지막 short 플래그가 값을 가질 수 있음)
    /// - `-abc` (모두 bool인 short 플래그 그룹)
    /// - `--` sentinel (이후 모두 위치 인자)
    /// - Bool 플래그는 값 없이 존재하면 `true`로 처리
    pub fn parse(&mut self, args: Vec<String>) -> Result<Vec<String>> {
        log::trace!("플래그 파싱 시작: {:?}", args);
        let mut positional = Vec::new();
        let mut iter = args.into_iter().peekable();

        while let Some(arg) = iter.next() {
            if arg == "--" {
                positional.extend(iter);
                break;
            }
            if let Some(rest) = arg.strip_prefix("--") {
                self.parse_long(rest, &mut iter, &mut positional)?;
            } else if arg.starts_with('-') && arg.len() > 1 {
                self.parse_short(&arg[1..], &mut iter)?;
            } else {
                positional.push(arg);
            }
        }

        for flag in self.flags.values() {
            if flag.required && !self.values.contains_key(&flag.name) {
                return Err(WrCliError::MissingRequiredFlag(flag.name.clone()));
            }
        }

        Ok(positional)
    }

    fn parse_long(
        &mut self,
        rest: &str,
        iter: &mut impl Iterator<Item = String>,
        _positional: &mut Vec<String>,
    ) -> Result<()> {
        let (name, value_opt) = if let Some(eq) = rest.find('=') {
            (&rest[..eq], Some(rest[eq + 1..].to_owned()))
        } else {
            (rest, None)
        };

        let flags = &self.flags;
        let values = &mut self.values;

        let flag_ref = flags.get(name).ok_or_else(|| WrCliError::UnknownFlag {
            flag: format!("--{}", name),
            command: String::new(),
        })?;

        match &flag_ref.default {
            FlagValue::Bool(_) => {
                let v = value_opt
                    .as_deref()
                    .map(|s| matches!(s, "true" | "1" | "yes"))
                    .unwrap_or(true);
                values.insert(flag_ref.name.clone(), FlagValue::Bool(v));
            }
            FlagValue::StringVec(_) => {
                let s = value_opt
                    .or_else(|| iter.next())
                    .ok_or_else(|| WrCliError::MissingRequiredFlag(flag_ref.name.clone()))?;
                let entry = values
                    .entry(flag_ref.name.clone())
                    .or_insert(FlagValue::StringVec(vec![]));
                if let FlagValue::StringVec(v) = entry {
                    v.push(s);
                }
                return Ok(());
            }
            FlagValue::IntVec(_) => {
                let s = value_opt
                    .or_else(|| iter.next())
                    .ok_or_else(|| WrCliError::MissingRequiredFlag(flag_ref.name.clone()))?;
                let n = s.parse::<i64>().map_err(|_| WrCliError::InvalidFlagValue {
                    flag: flag_ref.name.clone(),
                    expected: "integer",
                    got: s.clone(),
                })?;
                let entry = values
                    .entry(flag_ref.name.clone())
                    .or_insert(FlagValue::IntVec(vec![]));
                if let FlagValue::IntVec(v) = entry {
                    v.push(n);
                }
                return Ok(());
            }
            _ => {
                let s = value_opt
                    .or_else(|| iter.next())
                    .ok_or_else(|| WrCliError::MissingRequiredFlag(flag_ref.name.clone()))?;
                let parsed = Self::coerce(flag_ref, &s)?;
                values.insert(flag_ref.name.clone(), parsed);
            }
        }
        Ok(())
    }

    fn parse_short(
        &mut self,
        chars_str: &str,
        iter: &mut impl Iterator<Item = String>,
    ) -> Result<()> {
        let short_map = &self.short_map;
        let flags = &self.flags;
        let values = &mut self.values;

        let mut chars = chars_str.chars().peekable();
        while let Some(c) = chars.next() {
            let is_last = chars.peek().is_none();

            let flag_name = short_map.get(&c).ok_or_else(|| WrCliError::UnknownFlag {
                flag: format!("-{}", c),
                command: String::new(),
            })?;
            let flag_ref = flags.get(flag_name.as_str()).unwrap();

            let parsed = match &flag_ref.default {
                FlagValue::Bool(_) => FlagValue::Bool(true),
                _ => {
                    if is_last {
                        let s = iter.next().ok_or_else(|| {
                            WrCliError::MissingRequiredFlag(flag_ref.name.clone())
                        })?;
                        Self::coerce(flag_ref, &s)?
                    } else {
                        return Err(WrCliError::InvalidFlagValue {
                            flag: format!("-{}", c),
                            expected: "bool (only the last short flag in a group may take a value)",
                            got: flag_ref.default.type_name().to_owned(),
                        });
                    }
                }
            };

            values.insert(flag_name.clone(), parsed);
        }

        Ok(())
    }

    fn coerce(flag: &Flag, s: &str) -> Result<FlagValue> {
        match &flag.default {
            FlagValue::String(_) => Ok(FlagValue::String(s.to_owned())),
            FlagValue::Int(_) => s.parse::<i64>().map(FlagValue::Int).map_err(|_| {
                WrCliError::InvalidFlagValue {
                    flag: flag.name.clone(),
                    expected: "integer",
                    got: s.to_owned(),
                }
            }),
            FlagValue::Float(_) => s.parse::<f64>().map(FlagValue::Float).map_err(|_| {
                WrCliError::InvalidFlagValue {
                    flag: flag.name.clone(),
                    expected: "float",
                    got: s.to_owned(),
                }
            }),
            FlagValue::Bool(_) => match s {
                "true" | "1" | "yes" => Ok(FlagValue::Bool(true)),
                "false" | "0" | "no" => Ok(FlagValue::Bool(false)),
                _ => Err(WrCliError::InvalidFlagValue {
                    flag: flag.name.clone(),
                    expected: "bool (true/false/1/0/yes/no)",
                    got: s.to_owned(),
                }),
            },
            _ => Ok(FlagValue::String(s.to_owned())),
        }
    }
}

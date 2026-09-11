use indexmap::IndexMap;
use std::collections::{HashMap, HashSet};

use super::definition::Flag;
use super::value::FlagValue;
use crate::config::Config;
use crate::error::{Result, WrCliError};

/// 쉼표 분리 대상 값을 분리하고 공백을 제거. `comma`가 false면 원본 하나.
fn split_values(s: String, comma: bool) -> Vec<String> {
    if comma {
        s.split(',')
            .map(|p| p.trim().to_owned())
            .filter(|p| !p.is_empty())
            .collect()
    } else {
        vec![s]
    }
}

/// 단일 커맨드의 모든 플래그를 담는 컨테이너. 삽입 순서 보존 (help 출력용).
#[derive(Debug, Default, Clone)]
pub struct FlagSet {
    flags: IndexMap<String, Flag>,
    short_map: HashMap<char, String>,
    values: HashMap<String, FlagValue>,
    /// 사용자가 argv로 명시한 플래그 이름 (설정에서 시드된 값은 제외).
    user_set: HashSet<String>,
    command_name: String,
}

impl FlagSet {
    pub fn new() -> Self {
        Default::default()
    }

    pub(crate) fn set_command_name(&mut self, name: &str) {
        self.command_name = name.to_owned();
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

    /// 부모 커맨드에서 상속된 persistent 플래그를 추가 (help에서 Global Flags로 분리).
    pub(crate) fn add_inherited(&mut self, flag: &Flag) {
        if !self.flags.contains_key(&flag.name) {
            let mut inherited = flag.clone();
            inherited.inherited = true;
            self.add(inherited);
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

    /// `u64` 값 조회 (음수 `Int`는 `None`).
    pub fn get_uint(&self, name: &str) -> Option<u64> {
        match self.get(name)? {
            FlagValue::Int(v) if *v >= 0 => Some(*v as u64),
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

    pub fn get_int_vec(&self, name: &str) -> Option<&[i64]> {
        match self.get(name)? {
            FlagValue::IntVec(v) => Some(v.as_slice()),
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

    /// 사용자가 명시적으로 입력한 값만 반복 (기본값/설정 시드 제외).
    /// 디스패치 엔진이 Config 레이어 4 바인딩 시 사용.
    pub(crate) fn values_iter(&self) -> impl Iterator<Item = (&str, &FlagValue)> {
        self.user_set
            .iter()
            .filter_map(|k| self.values.get(k).map(|v| (k.as_str(), v)))
    }

    /// 사용자가 argv로 이 플래그를 명시적으로 지정했는지 확인.
    ///
    /// 설정에서 시드된 값이나 기본값은 `false`를 반환한다.
    pub fn is_set(&self, name: &str) -> bool {
        self.user_set.contains(name)
    }

    /// 명시적으로 설정되지 않은 플래그를 설정 저장소 값으로 시드.
    ///
    /// 플래그 기본값 타입에 맞는 설정값만 주입한다.
    pub(crate) fn seed_from_config(&mut self, config: &Config) {
        let FlagSet { flags, values, .. } = self;
        for name in flags.keys() {
            if values.contains_key(name) {
                continue;
            }
            let Some(flag) = flags.get(name) else {
                continue;
            };
            let Some(cv) = config.get(name) else {
                continue;
            };
            if let Some(fv) = super::value::flag_value_from_config(&flag.default, cv) {
                values.insert(name.clone(), fv);
            }
        }
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
        self.parse_inner(args, true)
    }

    /// 서브커맨드 앞의 부모 플래그를 소비하기 위한 변형.
    ///
    /// required 플래그 검증은 리프 커맨드에서만 수행하므로 여기서는 건너뛴다.
    pub(crate) fn parse_partial(&mut self, args: Vec<String>) -> Result<Vec<String>> {
        self.parse_inner(args, false)
    }

    fn parse_inner(&mut self, args: Vec<String>, validate_required: bool) -> Result<Vec<String>> {
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

        if validate_required {
            for flag in self.flags.values() {
                if flag.required && !self.values.contains_key(&flag.name) {
                    return Err(WrCliError::MissingRequiredFlag(flag.name.clone()));
                }
            }
        }

        Ok(positional)
    }

    /// 부모에서 소비된 플래그 값을 하위 FlagSet으로 복사.
    ///
    /// 하위가 같은 이름을 재정의하면 하위 파싱이 덮어쓴다. 정의(help/completion)는
    /// 추가하지 않으므로 부모 로컬 플래그는 하위 help에 노출되지 않는다.
    pub(crate) fn inherit_values(&mut self, parent: &FlagSet) {
        for (name, value) in &parent.values {
            self.values
                .entry(name.clone())
                .or_insert_with(|| value.clone());
        }
        for name in &parent.user_set {
            self.user_set.insert(name.clone());
        }
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
        let user_set = &mut self.user_set;

        let flag_ref = flags.get(name).ok_or_else(|| {
            log::warn!("unknown long flag '--{}' for '{}'", name, self.command_name);
            WrCliError::UnknownFlag {
                flag: format!("--{}", name),
                command: self.command_name.clone(),
                suggestions: crate::suggest::closest(
                    name,
                    flags
                        .values()
                        .filter(|f| !f.hidden)
                        .map(|f| f.name.as_str()),
                ),
            }
        })?;

        if let Some(msg) = &flag_ref.deprecated {
            eprintln!("Flag --{} is deprecated: {}", flag_ref.name, msg);
        }

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
                    .ok_or_else(|| WrCliError::MissingFlagValue(flag_ref.name.clone()))?;
                let parts = split_values(s, flag_ref.comma_separated);
                let entry = values
                    .entry(flag_ref.name.clone())
                    .or_insert(FlagValue::StringVec(vec![]));
                if let FlagValue::StringVec(v) = entry {
                    v.extend(parts);
                }
                user_set.insert(flag_ref.name.clone());
                return Ok(());
            }
            FlagValue::IntVec(_) => {
                let s = value_opt
                    .or_else(|| iter.next())
                    .ok_or_else(|| WrCliError::MissingFlagValue(flag_ref.name.clone()))?;
                let mut parsed = Vec::new();
                for part in split_values(s, flag_ref.comma_separated) {
                    let n = part
                        .parse::<i64>()
                        .map_err(|_| WrCliError::InvalidFlagValue {
                            flag: flag_ref.name.clone(),
                            expected: "integer",
                            got: part.clone(),
                        })?;
                    parsed.push(n);
                }
                let entry = values
                    .entry(flag_ref.name.clone())
                    .or_insert(FlagValue::IntVec(vec![]));
                if let FlagValue::IntVec(v) = entry {
                    v.extend(parsed);
                }
                user_set.insert(flag_ref.name.clone());
                return Ok(());
            }
            _ => {
                let s = value_opt
                    .or_else(|| iter.next())
                    .ok_or_else(|| WrCliError::MissingFlagValue(flag_ref.name.clone()))?;
                let parsed = Self::coerce(flag_ref, &s)?;
                values.insert(flag_ref.name.clone(), parsed);
            }
        }
        user_set.insert(flag_ref.name.clone());
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
        let user_set = &mut self.user_set;

        let mut chars = chars_str.chars().peekable();
        while let Some(c) = chars.next() {
            let is_last = chars.peek().is_none();

            let flag_name = short_map.get(&c).ok_or_else(|| {
                log::warn!("unknown short flag '-{}' for '{}'", c, self.command_name);
                WrCliError::UnknownFlag {
                    flag: format!("-{}", c),
                    command: self.command_name.clone(),
                    suggestions: Vec::new(),
                }
            })?;
            let flag_ref = flags.get(flag_name.as_str()).unwrap();

            if let Some(msg) = &flag_ref.deprecated {
                eprintln!("Flag -{} (--{}) is deprecated: {}", c, flag_ref.name, msg);
            }

            let parsed = match &flag_ref.default {
                FlagValue::Bool(_) => FlagValue::Bool(true),
                _ => {
                    if is_last {
                        let s = iter
                            .next()
                            .ok_or_else(|| WrCliError::MissingFlagValue(flag_ref.name.clone()))?;
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
            user_set.insert(flag_name.clone());
        }

        Ok(())
    }

    fn coerce(flag: &Flag, s: &str) -> Result<FlagValue> {
        match &flag.default {
            FlagValue::String(_) => Ok(FlagValue::String(s.to_owned())),
            FlagValue::Int(_) => {
                s.parse::<i64>()
                    .map(FlagValue::Int)
                    .map_err(|_| WrCliError::InvalidFlagValue {
                        flag: flag.name.clone(),
                        expected: "integer",
                        got: s.to_owned(),
                    })
            }
            FlagValue::Float(_) => {
                s.parse::<f64>()
                    .map(FlagValue::Float)
                    .map_err(|_| WrCliError::InvalidFlagValue {
                        flag: flag.name.clone(),
                        expected: "float",
                        got: s.to_owned(),
                    })
            }
            FlagValue::Bool(_) => match s {
                "true" | "1" | "yes" => Ok(FlagValue::Bool(true)),
                "false" | "0" | "no" => Ok(FlagValue::Bool(false)),
                _ => Err(WrCliError::InvalidFlagValue {
                    flag: flag.name.clone(),
                    expected: "bool (true/false/1/0/yes/no)",
                    got: s.to_owned(),
                }),
            },
            FlagValue::StringVec(_) | FlagValue::IntVec(_) => {
                // StringVec/IntVec는 parse_long에서 직접 처리되므로 여기 도달하면 버그
                Ok(FlagValue::String(s.to_owned()))
            }
        }
    }
}

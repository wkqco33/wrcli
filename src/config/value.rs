use crate::flag::FlagValue;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// 설정 시스템에 저장되는 타입별 값.
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Array(Vec<ConfigValue>),
}

impl ConfigValue {
    pub fn as_bool(&self) -> Option<bool> {
        if let ConfigValue::Bool(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        if let ConfigValue::Int(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        if let ConfigValue::Float(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        if let ConfigValue::String(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_array(&self) -> Option<&[ConfigValue]> {
        if let ConfigValue::Array(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub(crate) fn to_string_coerce(&self) -> Option<String> {
        match self {
            ConfigValue::String(s) => Some(s.clone()),
            ConfigValue::Int(i) => Some(i.to_string()),
            ConfigValue::Float(f) => Some(f.to_string()),
            ConfigValue::Bool(b) => Some(b.to_string()),
            ConfigValue::Array(_) => None,
        }
    }

    pub(crate) fn to_int_coerce(&self) -> Option<i64> {
        match self {
            ConfigValue::Int(i) => Some(*i),
            ConfigValue::Float(f) => Some(*f as i64),
            ConfigValue::String(s) => s.parse().ok(),
            _ => None,
        }
    }

    pub(crate) fn to_float_coerce(&self) -> Option<f64> {
        match self {
            ConfigValue::Float(f) => Some(*f),
            ConfigValue::Int(i) => Some(*i as f64),
            ConfigValue::String(s) => s.parse().ok(),
            _ => None,
        }
    }

    pub(crate) fn to_bool_coerce(&self) -> Option<bool> {
        match self {
            ConfigValue::Bool(b) => Some(*b),
            ConfigValue::String(s) => match s.as_str() {
                "true" | "1" | "yes" => Some(true),
                "false" | "0" | "no" => Some(false),
                _ => None,
            },
            ConfigValue::Int(i) => Some(*i != 0),
            _ => None,
        }
    }

    pub(crate) fn to_uint_coerce(&self) -> Option<u64> {
        match self {
            ConfigValue::Int(i) if *i >= 0 => Some(*i as u64),
            ConfigValue::Float(f) if *f >= 0.0 => Some(*f as u64),
            ConfigValue::String(s) => s.parse().ok(),
            _ => None,
        }
    }

    /// 문자열은 Go 스타일 duration(`"1h30m"`, `"250ms"`), 숫자는 초로 해석.
    pub(crate) fn to_duration_coerce(&self) -> Option<Duration> {
        match self {
            ConfigValue::Int(i) if *i >= 0 => Some(Duration::from_secs(*i as u64)),
            ConfigValue::Float(f) if *f >= 0.0 => Duration::try_from_secs_f64(*f).ok(),
            ConfigValue::String(s) => parse_duration(s),
            _ => None,
        }
    }

    /// 숫자는 Unix epoch 초, 문자열은 RFC3339 또는 Unix 초로 해석.
    pub(crate) fn to_time_coerce(&self) -> Option<SystemTime> {
        match self {
            ConfigValue::Int(i) => secs_to_system_time(*i),
            ConfigValue::Float(f) => secs_to_system_time_f(*f),
            ConfigValue::String(s) => parse_time(s),
            _ => None,
        }
    }

    /// `"1KB"`, `"1.5MB"`, `"2GiB"` 등 1024 기반 단위를 바이트로 변환.
    pub(crate) fn to_size_in_bytes_coerce(&self) -> Option<u64> {
        match self {
            ConfigValue::Int(i) if *i >= 0 => Some(*i as u64),
            ConfigValue::Float(f) if *f >= 0.0 => Some(*f as u64),
            ConfigValue::String(s) => parse_size(s),
            _ => None,
        }
    }
}

/// Go `time.ParseDuration` 스타일 문자열을 파싱.
fn parse_duration(s: &str) -> Option<Duration> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    let mut total = 0f64;
    let mut rest = s;
    while !rest.is_empty() {
        let num_len = rest
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .count();
        if num_len == 0 {
            return None;
        }
        let value: f64 = rest[..num_len].parse().ok()?;
        let after = &rest[num_len..];
        let unit_len = after
            .chars()
            .take_while(|c| !c.is_ascii_digit() && *c != '.')
            .count();
        let multiplier = match &after[..unit_len] {
            "ns" => 1e-9,
            "us" | "µs" | "μs" => 1e-6,
            "ms" => 1e-3,
            "s" => 1.0,
            "m" => 60.0,
            "h" => 3600.0,
            _ => return None,
        };
        total += value * multiplier;
        rest = &after[unit_len..];
    }
    Duration::try_from_secs_f64(total).ok()
}

/// `"10MB"`, `"1.5 GiB"`, `"512"` 등을 바이트로 파싱 (1024 기반).
fn parse_size(s: &str) -> Option<u64> {
    let s = s.trim();
    let num_len = s
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .count();
    if num_len == 0 {
        return None;
    }
    let value: f64 = s[..num_len].parse().ok()?;
    let multiplier = match s[num_len..].trim().to_ascii_lowercase().as_str() {
        "" | "b" => 1.0,
        "k" | "kb" | "kib" => 1024.0,
        "m" | "mb" | "mib" => 1024f64.powi(2),
        "g" | "gb" | "gib" => 1024f64.powi(3),
        "t" | "tb" | "tib" => 1024f64.powi(4),
        "p" | "pb" | "pib" => 1024f64.powi(5),
        _ => return None,
    };
    let bytes = value * multiplier;
    if bytes < 0.0 {
        return None;
    }
    Some(bytes as u64)
}

fn parse_time(s: &str) -> Option<SystemTime> {
    let s = s.trim();
    if let Some(t) = parse_rfc3339(s) {
        return Some(t);
    }
    if let Ok(i) = s.parse::<i64>() {
        return secs_to_system_time(i);
    }
    s.parse::<f64>().ok().and_then(secs_to_system_time_f)
}

fn secs_to_system_time(secs: i64) -> Option<SystemTime> {
    if secs >= 0 {
        UNIX_EPOCH.checked_add(Duration::from_secs(secs as u64))
    } else {
        UNIX_EPOCH.checked_sub(Duration::from_secs(secs.unsigned_abs()))
    }
}

fn secs_to_system_time_f(secs: f64) -> Option<SystemTime> {
    if secs >= 0.0 {
        UNIX_EPOCH.checked_add(Duration::try_from_secs_f64(secs).ok()?)
    } else {
        UNIX_EPOCH.checked_sub(Duration::try_from_secs_f64(-secs).ok()?)
    }
}

/// `YYYY-MM-DDThh:mm:ss[.frac][Z|±hh:mm]` 를 UTC로 파싱.
fn parse_rfc3339(s: &str) -> Option<SystemTime> {
    if s.len() < 19 {
        return None;
    }
    let year: i64 = s.get(0..4)?.parse().ok()?;
    let month: u32 = s.get(5..7)?.parse().ok()?;
    let day: u32 = s.get(8..10)?.parse().ok()?;
    if &s[4..5] != "-" || &s[7..8] != "-" || !matches!(s.as_bytes()[10], b'T' | b't' | b' ') {
        return None;
    }
    let hour: u32 = s.get(11..13)?.parse().ok()?;
    let minute: u32 = s.get(14..16)?.parse().ok()?;
    let second: u32 = s.get(17..19)?.parse().ok()?;
    if &s[13..14] != ":"
        || &s[16..17] != ":"
        || !(1..=12).contains(&month)
        || !(1..=31).contains(&day)
        || hour > 23
        || minute > 59
        || second > 60
    {
        return None;
    }

    let mut rest = &s[19..];
    let mut nanos = 0u32;
    if let Some(fraction) = rest.strip_prefix('.') {
        let digits: String = fraction.chars().take_while(char::is_ascii_digit).collect();
        if digits.is_empty() {
            return None;
        }
        let mut padded = digits[..digits.len().min(9)].to_owned();
        while padded.len() < 9 {
            padded.push('0');
        }
        nanos = padded.parse().ok()?;
        rest = &fraction[digits.len()..];
    }

    let offset_secs: i64 = match rest {
        "" | "Z" | "z" => 0,
        _ => {
            let sign = match rest.as_bytes().first() {
                Some(b'+') => 1,
                Some(b'-') => -1,
                _ => return None,
            };
            if &rest[3..4] != ":" {
                return None;
            }
            let hours: i64 = rest.get(1..3)?.parse().ok()?;
            let minutes: i64 = rest.get(4..6)?.parse().ok()?;
            sign * (hours * 3600 + minutes * 60)
        }
    };

    let days = days_from_civil(year, month, day);
    let secs =
        days * 86_400 + hour as i64 * 3600 + minute as i64 * 60 + second as i64 - offset_secs;
    let base = secs_to_system_time(secs)?;
    base.checked_add(Duration::from_nanos(nanos as u64))
}

/// Civil date → Unix epoch 기준 일수 (Howard Hinnant 알고리즘).
fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let yoe = year - era * 400;
    let mp = (month as i64 + 9) % 12;
    let doy = (153 * mp + 2) / 5 + day as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

// ── From impls ────────────────────────────────────────────────────────────────

impl From<bool> for ConfigValue {
    fn from(v: bool) -> Self {
        ConfigValue::Bool(v)
    }
}
impl From<i64> for ConfigValue {
    fn from(v: i64) -> Self {
        ConfigValue::Int(v)
    }
}
impl From<i32> for ConfigValue {
    fn from(v: i32) -> Self {
        ConfigValue::Int(v as i64)
    }
}
impl From<u32> for ConfigValue {
    fn from(v: u32) -> Self {
        ConfigValue::Int(v as i64)
    }
}
impl From<usize> for ConfigValue {
    fn from(v: usize) -> Self {
        ConfigValue::Int(v as i64)
    }
}
impl From<f64> for ConfigValue {
    fn from(v: f64) -> Self {
        ConfigValue::Float(v)
    }
}
impl From<f32> for ConfigValue {
    fn from(v: f32) -> Self {
        ConfigValue::Float(v as f64)
    }
}
impl From<String> for ConfigValue {
    fn from(v: String) -> Self {
        ConfigValue::String(v)
    }
}
impl From<&str> for ConfigValue {
    fn from(v: &str) -> Self {
        ConfigValue::String(v.to_owned())
    }
}
impl From<Vec<&str>> for ConfigValue {
    fn from(v: Vec<&str>) -> Self {
        ConfigValue::Array(v.into_iter().map(ConfigValue::from).collect())
    }
}
impl From<Vec<String>> for ConfigValue {
    fn from(v: Vec<String>) -> Self {
        ConfigValue::Array(v.into_iter().map(ConfigValue::String).collect())
    }
}
impl From<Vec<i64>> for ConfigValue {
    fn from(v: Vec<i64>) -> Self {
        ConfigValue::Array(v.into_iter().map(ConfigValue::Int).collect())
    }
}

/// 파싱된 `FlagValue`를 레이어 4 바인딩용 `ConfigValue`로 변환.
impl From<&FlagValue> for ConfigValue {
    fn from(fv: &FlagValue) -> Self {
        match fv {
            FlagValue::Bool(b) => ConfigValue::Bool(*b),
            FlagValue::String(s) => ConfigValue::String(s.clone()),
            FlagValue::Int(i) => ConfigValue::Int(*i),
            FlagValue::Float(f) => ConfigValue::Float(*f),
            FlagValue::StringVec(v) => {
                ConfigValue::Array(v.iter().map(|s| ConfigValue::String(s.clone())).collect())
            }
            FlagValue::IntVec(v) => {
                ConfigValue::Array(v.iter().map(|i| ConfigValue::Int(*i)).collect())
            }
        }
    }
}

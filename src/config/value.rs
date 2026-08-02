use crate::flag::FlagValue;

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

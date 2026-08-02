/// 플래그의 타입별 값 — 기본값을 통해 타입 태그 역할도 함.
#[derive(Debug, Clone, PartialEq)]
pub enum FlagValue {
    Bool(bool),
    String(String),
    Int(i64),
    Float(f64),
    StringVec(Vec<String>),
    IntVec(Vec<i64>),
}

impl FlagValue {
    pub fn type_name(&self) -> &'static str {
        match self {
            FlagValue::Bool(_) => "bool",
            FlagValue::String(_) => "string",
            FlagValue::Int(_) => "int",
            FlagValue::Float(_) => "float",
            FlagValue::StringVec(_) => "string...",
            FlagValue::IntVec(_) => "int...",
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        if let FlagValue::Bool(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        if let FlagValue::String(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        if let FlagValue::Int(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        if let FlagValue::Float(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_string_vec(&self) -> Option<&[String]> {
        if let FlagValue::StringVec(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_int_vec(&self) -> Option<&[i64]> {
        if let FlagValue::IntVec(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

/// 설정값을 플래그 값으로 변환. 플래그 기본값의 타입에 맞춰 변환하며,
/// 타입이 맞지 않으면 `None` 반환.
pub(crate) fn flag_value_from_config(default: &FlagValue, cv: ConfigValue) -> Option<FlagValue> {
    match default {
        FlagValue::Bool(_) => cv.as_bool().map(FlagValue::Bool),
        FlagValue::Int(_) => cv.as_int().map(FlagValue::Int),
        FlagValue::Float(_) => cv.as_float().map(FlagValue::Float),
        FlagValue::String(_) => cv.as_str().map(str::to_owned).map(FlagValue::String),
        FlagValue::StringVec(_) => cv.as_array().map(|arr| {
            FlagValue::StringVec(
                arr.iter()
                    .filter_map(ConfigValue::as_str)
                    .map(str::to_owned)
                    .collect(),
            )
        }),
        FlagValue::IntVec(_) => cv
            .as_array()
            .map(|arr| FlagValue::IntVec(arr.iter().filter_map(ConfigValue::as_int).collect())),
    }
}

use crate::config::ConfigValue;

use super::ConfigValue;
use super::settings::{SettingsEntry, SettingsMap};
use crate::error::{Result, WrCliError};

/// 중첩 설정 트리를 `ext` 포맷의 문자열로 직렬화.
#[allow(unused_variables)]
pub(super) fn serialize(values: &SettingsMap, ext: &str, path: &str) -> Result<String> {
    match ext {
        #[cfg(feature = "toml-config")]
        "toml" => to_toml(values, path),
        #[cfg(feature = "json-config")]
        "json" => to_json(values, path),
        #[cfg(feature = "dotenv-config")]
        "env" | "dotenv" => Ok(to_dotenv(values)),
        #[cfg(feature = "properties-config")]
        "properties" | "props" | "prop" => Ok(to_properties(values)),
        #[cfg(feature = "ini-config")]
        "ini" => Ok(to_ini(values)),
        other => Err(WrCliError::UnsupportedConfigFormat(other.to_owned())),
    }
}

/// 리프 값을 점 표기 키 목록으로 평탄화.
#[cfg(any(
    feature = "dotenv-config",
    feature = "properties-config",
    feature = "ini-config"
))]
fn flatten<'a>(values: &'a SettingsMap, prefix: &str, out: &mut Vec<(String, &'a ConfigValue)>) {
    for (key, entry) in values {
        let full = if prefix.is_empty() {
            key.clone()
        } else {
            format!("{}.{}", prefix, key)
        };
        match entry {
            SettingsEntry::Value(v) => out.push((full, v)),
            SettingsEntry::Map(m) => flatten(m, &full, out),
        }
    }
}

#[cfg(any(
    feature = "dotenv-config",
    feature = "properties-config",
    feature = "ini-config"
))]
fn scalar_string(value: &ConfigValue) -> String {
    match value {
        ConfigValue::String(s) => s.clone(),
        other => other.to_string_coerce().unwrap_or_default(),
    }
}

#[cfg(feature = "json-config")]
fn to_json(values: &SettingsMap, path: &str) -> Result<String> {
    serde_json::to_string_pretty(&json_value(values)).map_err(|e| WrCliError::ConfigParseError {
        path: path.to_owned(),
        source: e.to_string(),
    })
}

#[cfg(feature = "json-config")]
fn json_value(values: &SettingsMap) -> serde_json::Value {
    let mut object = serde_json::Map::new();
    for (key, entry) in values {
        object.insert(
            key.clone(),
            match entry {
                SettingsEntry::Value(v) => config_to_json(v),
                SettingsEntry::Map(m) => json_value(m),
            },
        );
    }
    serde_json::Value::Object(object)
}

#[cfg(feature = "json-config")]
fn config_to_json(value: &ConfigValue) -> serde_json::Value {
    match value {
        ConfigValue::Bool(b) => serde_json::Value::Bool(*b),
        ConfigValue::Int(i) => serde_json::Value::Number((*i).into()),
        ConfigValue::Float(f) => serde_json::Number::from_f64(*f)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        ConfigValue::String(s) => serde_json::Value::String(s.clone()),
        ConfigValue::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(config_to_json).collect())
        }
    }
}

#[cfg(feature = "toml-config")]
fn to_toml(values: &SettingsMap, path: &str) -> Result<String> {
    let table = toml::Value::Table(toml_table(values));
    toml::to_string_pretty(&table).map_err(|e| WrCliError::ConfigParseError {
        path: path.to_owned(),
        source: e.to_string(),
    })
}

#[cfg(feature = "toml-config")]
fn toml_table(values: &SettingsMap) -> toml::map::Map<String, toml::Value> {
    let mut table = toml::map::Map::new();
    for (key, entry) in values {
        table.insert(
            key.clone(),
            match entry {
                SettingsEntry::Value(v) => config_to_toml(v),
                SettingsEntry::Map(m) => toml::Value::Table(toml_table(m)),
            },
        );
    }
    table
}

#[cfg(feature = "toml-config")]
fn config_to_toml(value: &ConfigValue) -> toml::Value {
    match value {
        ConfigValue::Bool(b) => toml::Value::Boolean(*b),
        ConfigValue::Int(i) => toml::Value::Integer(*i),
        ConfigValue::Float(f) => toml::Value::Float(*f),
        ConfigValue::String(s) => toml::Value::String(s.clone()),
        ConfigValue::Array(arr) => toml::Value::Array(arr.iter().map(config_to_toml).collect()),
    }
}

#[cfg(feature = "dotenv-config")]
fn to_dotenv(values: &SettingsMap) -> String {
    let mut flat = Vec::new();
    flatten(values, "", &mut flat);
    let mut out = String::new();
    for (key, value) in flat {
        out.push_str(&format!(
            "{}={}\n",
            key.to_uppercase(),
            scalar_string(value)
        ));
    }
    out
}

#[cfg(feature = "properties-config")]
fn to_properties(values: &SettingsMap) -> String {
    let mut flat = Vec::new();
    flatten(values, "", &mut flat);
    let mut out = String::new();
    for (key, value) in flat {
        out.push_str(&format!("{}={}\n", key, scalar_string(value)));
    }
    out
}

#[cfg(feature = "ini-config")]
fn to_ini(values: &SettingsMap) -> String {
    let mut out = String::new();
    for (key, entry) in values {
        match entry {
            SettingsEntry::Map(m) => {
                out.push_str(&format!("[{}]\n", key));
                for (child_key, child) in m {
                    if let SettingsEntry::Value(v) = child {
                        out.push_str(&format!("{}={}\n", child_key, scalar_string(v)));
                    }
                }
                out.push('\n');
            }
            SettingsEntry::Value(v) => out.push_str(&format!("{}={}\n", key, scalar_string(v))),
        }
    }
    out
}

use super::value::ConfigValue;
use crate::error::{Result, WrCliError};
use std::collections::HashMap;

/// 설정 파일 내용을 파싱해서 플랫 키-값 맵으로 변환.
pub(super) fn parse_config_content(
    #[allow(unused)] content: &str,
    ext: &str,
    path: &str,
) -> Result<HashMap<String, ConfigValue>> {
    log::debug!("설정 파일 파싱 시작: {} (포맷: {})", path, ext);
    match ext {
        #[cfg(feature = "toml-config")]
        "toml" => parse_toml(content, path),

        #[cfg(feature = "json-config")]
        "json" => parse_json(content, path),

        #[cfg(feature = "yaml-config")]
        "yaml" | "yml" => parse_yaml(content, path),

        other => Err(WrCliError::UnsupportedConfigFormat(other.to_owned())),
    }
}

#[allow(dead_code)]
fn child_key(prefix: &str, key: &str) -> String {
    if prefix.is_empty() {
        key.to_owned()
    } else {
        format!("{}.{}", prefix, key)
    }
}

// ── TOML ──────────────────────────────────────────────────────────────────────

#[cfg(feature = "toml-config")]
fn parse_toml(content: &str, path: &str) -> Result<HashMap<String, ConfigValue>> {
    let value: toml::Value = toml::from_str(content).map_err(|e| WrCliError::ConfigParseError {
        path: path.to_owned(),
        source: e.to_string(),
    })?;
    let mut map = HashMap::new();
    flatten_toml("", &value, &mut map);
    Ok(map)
}

#[cfg(feature = "toml-config")]
fn toml_scalar(v: &toml::Value) -> Option<ConfigValue> {
    match v {
        toml::Value::String(s) => Some(ConfigValue::String(s.clone())),
        toml::Value::Integer(i) => Some(ConfigValue::Int(*i)),
        toml::Value::Float(f) => Some(ConfigValue::Float(*f)),
        toml::Value::Boolean(b) => Some(ConfigValue::Bool(*b)),
        toml::Value::Datetime(dt) => Some(ConfigValue::String(dt.to_string())),
        _ => None,
    }
}

#[cfg(feature = "toml-config")]
fn flatten_toml(prefix: &str, value: &toml::Value, map: &mut HashMap<String, ConfigValue>) {
    match value {
        toml::Value::Table(t) => {
            for (k, v) in t {
                let key = child_key(prefix, k);
                flatten_toml(&key, v, map);
            }
        }
        toml::Value::Array(arr) => {
            let cv: Vec<ConfigValue> = arr.iter().filter_map(toml_scalar).collect();
            map.insert(prefix.to_owned(), ConfigValue::Array(cv));
        }
        other => {
            if let Some(cv) = toml_scalar(other) {
                map.insert(prefix.to_owned(), cv);
            }
        }
    }
}

// ── JSON ──────────────────────────────────────────────────────────────────────

#[cfg(feature = "json-config")]
fn parse_json(content: &str, path: &str) -> Result<HashMap<String, ConfigValue>> {
    let value: serde_json::Value =
        serde_json::from_str(content).map_err(|e| WrCliError::ConfigParseError {
            path: path.to_owned(),
            source: e.to_string(),
        })?;
    let mut map = HashMap::new();
    flatten_json("", &value, &mut map);
    Ok(map)
}

#[cfg(feature = "json-config")]
fn json_scalar(v: &serde_json::Value) -> Option<ConfigValue> {
    match v {
        serde_json::Value::String(s) => Some(ConfigValue::String(s.clone())),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Some(ConfigValue::Int(i))
            } else {
                n.as_f64().map(ConfigValue::Float)
            }
        }
        serde_json::Value::Bool(b) => Some(ConfigValue::Bool(*b)),
        _ => None,
    }
}

#[cfg(feature = "json-config")]
fn flatten_json(prefix: &str, value: &serde_json::Value, map: &mut HashMap<String, ConfigValue>) {
    match value {
        serde_json::Value::Object(obj) => {
            for (k, v) in obj {
                let key = child_key(prefix, k);
                flatten_json(&key, v, map);
            }
        }
        serde_json::Value::Array(arr) => {
            let cv: Vec<ConfigValue> = arr.iter().filter_map(json_scalar).collect();
            map.insert(prefix.to_owned(), ConfigValue::Array(cv));
        }
        serde_json::Value::Null => {}
        other => {
            if let Some(cv) = json_scalar(other) {
                map.insert(prefix.to_owned(), cv);
            }
        }
    }
}

// ── YAML ──────────────────────────────────────────────────────────────────────

#[cfg(feature = "yaml-config")]
fn parse_yaml(content: &str, path: &str) -> Result<HashMap<String, ConfigValue>> {
    let value: noyalib::Value =
        noyalib::from_str(content).map_err(|e| WrCliError::ConfigParseError {
            path: path.to_owned(),
            source: e.to_string(),
        })?;
    let mut map = HashMap::new();
    flatten_yaml("", &value, &mut map);
    Ok(map)
}

#[cfg(feature = "yaml-config")]
fn yaml_scalar(v: &noyalib::Value) -> Option<ConfigValue> {
    match v {
        noyalib::Value::String(s) => Some(ConfigValue::String(s.clone())),
        noyalib::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Some(ConfigValue::Int(i))
            } else {
                Some(ConfigValue::Float(n.as_f64()))
            }
        }
        noyalib::Value::Bool(b) => Some(ConfigValue::Bool(*b)),
        _ => None,
    }
}

#[cfg(feature = "yaml-config")]
fn flatten_yaml(prefix: &str, value: &noyalib::Value, map: &mut HashMap<String, ConfigValue>) {
    match value {
        noyalib::Value::Mapping(m) => {
            for (k, v) in m {
                let key = child_key(prefix, k.as_str());
                flatten_yaml(&key, v, map);
            }
        }
        noyalib::Value::Sequence(arr) => {
            let cv: Vec<ConfigValue> = arr.iter().filter_map(yaml_scalar).collect();
            map.insert(prefix.to_owned(), ConfigValue::Array(cv));
        }
        noyalib::Value::Null => {}
        noyalib::Value::Tagged(tagged) => {
            flatten_yaml(prefix, tagged.value(), map);
        }
        other => {
            if let Some(cv) = yaml_scalar(other) {
                map.insert(prefix.to_owned(), cv);
            }
        }
    }
}

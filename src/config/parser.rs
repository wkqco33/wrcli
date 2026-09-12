use super::value::ConfigValue;
use crate::error::{Result, WrCliError};
use std::collections::HashMap;

/// Parses config file contents into a flat key-value map.
pub(super) fn parse_config_content(
    #[allow(unused)] content: &str,
    ext: &str,
    path: &str,
) -> Result<HashMap<String, ConfigValue>> {
    log::debug!("parsing config file: {} (format: {})", path, ext);
    match ext {
        #[cfg(feature = "toml-config")]
        "toml" => parse_toml(content, path),

        #[cfg(feature = "json-config")]
        "json" => parse_json(content, path),

        #[cfg(feature = "yaml-config")]
        "yaml" | "yml" => parse_yaml(content, path),

        #[cfg(feature = "dotenv-config")]
        "env" | "dotenv" => Ok(parse_dotenv(content)),

        #[cfg(feature = "properties-config")]
        "properties" | "props" | "prop" => Ok(parse_properties(content)),

        #[cfg(feature = "ini-config")]
        "ini" => Ok(parse_ini(content)),

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

/// Removes matching quotes from both ends.
#[cfg(any(
    feature = "dotenv-config",
    feature = "properties-config",
    feature = "ini-config"
))]
fn unquote(s: &str) -> String {
    let bytes = s.as_bytes();
    let quoted = s.len() >= 2
        && ((bytes[0] == b'"' && bytes[s.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[s.len() - 1] == b'\''));
    if quoted {
        s[1..s.len() - 1].to_owned()
    } else {
        s.to_owned()
    }
}

// ── dotenv ────────────────────────────────────────────────────────────────────

#[cfg(feature = "dotenv-config")]
fn parse_dotenv(content: &str) -> HashMap<String, ConfigValue> {
    let mut map = HashMap::new();
    for raw in content.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let line = line.strip_prefix("export ").unwrap_or(line);
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim().to_lowercase();
            if !key.is_empty() {
                map.insert(key, ConfigValue::String(unquote(value.trim())));
            }
        }
    }
    map
}

// ── Java properties ───────────────────────────────────────────────────────────

#[cfg(feature = "properties-config")]
fn parse_properties(content: &str) -> HashMap<String, ConfigValue> {
    let mut map = HashMap::new();
    for raw in content.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with('!') {
            continue;
        }
        if let Some(idx) = line.find(['=', ':']) {
            let key = line[..idx].trim().to_lowercase();
            if !key.is_empty() {
                map.insert(key, ConfigValue::String(unquote(line[idx + 1..].trim())));
            }
        }
    }
    map
}

// ── INI ───────────────────────────────────────────────────────────────────────

#[cfg(feature = "ini-config")]
fn parse_ini(content: &str) -> HashMap<String, ConfigValue> {
    let mut map = HashMap::new();
    let mut section = String::new();
    for raw in content.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
            continue;
        }
        if let Some(inner) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            section = inner.trim().to_lowercase();
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim().to_lowercase();
            let full = if section.is_empty() {
                key
            } else {
                format!("{}.{}", section, key)
            };
            if !full.is_empty() && !full.ends_with('.') {
                map.insert(full, ConfigValue::String(unquote(value.trim())));
            }
        }
    }
    map
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

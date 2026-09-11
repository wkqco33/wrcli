use super::ConfigValue;
use std::collections::BTreeMap;

/// 평탄한 점 표기 키를 재구성한 중첩 설정 트리.
pub type SettingsMap = BTreeMap<String, SettingsEntry>;

/// 중첩 트리의 노드: 리프 값 또는 하위 맵.
#[derive(Debug, Clone, PartialEq)]
pub enum SettingsEntry {
    Value(ConfigValue),
    Map(SettingsMap),
}

impl SettingsEntry {
    pub fn as_value(&self) -> Option<&ConfigValue> {
        match self {
            SettingsEntry::Value(v) => Some(v),
            SettingsEntry::Map(_) => None,
        }
    }

    pub fn as_map(&self) -> Option<&SettingsMap> {
        match self {
            SettingsEntry::Map(m) => Some(m),
            SettingsEntry::Value(_) => None,
        }
    }
}

/// `"a.b.c"` 형태의 평탄 키들을 중첩 트리로 재구성.
pub(crate) fn build_settings(
    entries: impl IntoIterator<Item = (String, ConfigValue)>,
) -> SettingsMap {
    let mut root = SettingsMap::new();
    for (key, value) in entries {
        let parts: Vec<&str> = key.split('.').filter(|p| !p.is_empty()).collect();
        if !parts.is_empty() {
            insert_nested(&mut root, &parts, value);
        }
    }
    root
}

fn insert_nested(map: &mut SettingsMap, parts: &[&str], value: ConfigValue) {
    if parts.len() == 1 {
        map.insert(parts[0].to_owned(), SettingsEntry::Value(value));
        return;
    }
    let entry = map
        .entry(parts[0].to_owned())
        .or_insert_with(|| SettingsEntry::Map(SettingsMap::new()));
    if !matches!(entry, SettingsEntry::Map(_)) {
        *entry = SettingsEntry::Map(SettingsMap::new());
    }
    if let SettingsEntry::Map(child) = entry {
        insert_nested(child, &parts[1..], value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_nested_tree_from_flat_keys() {
        let tree = build_settings([
            (
                "server.host".to_owned(),
                ConfigValue::String("h".to_owned()),
            ),
            ("server.port".to_owned(), ConfigValue::Int(1)),
            ("debug".to_owned(), ConfigValue::Bool(true)),
        ]);
        let server = tree.get("server").and_then(SettingsEntry::as_map).unwrap();
        assert_eq!(server.len(), 2);
        assert!(matches!(tree.get("debug"), Some(SettingsEntry::Value(_))));
    }

    #[test]
    fn leaf_and_map_conflict_keeps_map() {
        let tree = build_settings([
            ("a".to_owned(), ConfigValue::Int(1)),
            ("a.b".to_owned(), ConfigValue::Int(2)),
        ]);
        assert!(matches!(tree.get("a"), Some(SettingsEntry::Map(_))));
    }
}

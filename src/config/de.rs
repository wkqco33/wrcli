//! `serde`-based deserialization ([`super::Config::unmarshal`]).

use super::ConfigValue;
use super::settings::{SettingsEntry, SettingsMap};
use crate::error::WrCliError;
use serde::de::{
    DeserializeSeed, Deserializer, EnumAccess, MapAccess, SeqAccess, VariantAccess, Visitor,
};

enum Node<'a> {
    Value(&'a ConfigValue),
    Map(&'a SettingsMap),
}

pub(crate) struct ConfigDeserializer<'a> {
    node: Node<'a>,
}

impl<'a> ConfigDeserializer<'a> {
    pub(crate) fn value(value: &'a ConfigValue) -> Self {
        ConfigDeserializer {
            node: Node::Value(value),
        }
    }

    pub(crate) fn map(map: &'a SettingsMap) -> Self {
        ConfigDeserializer {
            node: Node::Map(map),
        }
    }
}

impl<'de, 'a> Deserializer<'de> for ConfigDeserializer<'a> {
    type Error = WrCliError;

    fn deserialize_any<V: Visitor<'de>>(
        self,
        visitor: V,
    ) -> std::result::Result<V::Value, WrCliError> {
        match self.node {
            Node::Value(value) => match value {
                ConfigValue::Bool(b) => visitor.visit_bool(*b),
                ConfigValue::Int(i) => visitor.visit_i64(*i),
                ConfigValue::Float(f) => visitor.visit_f64(*f),
                ConfigValue::String(s) => visitor.visit_str(s),
                ConfigValue::Array(items) => {
                    visitor.visit_seq(SeqDeserializer { iter: items.iter() })
                }
            },
            Node::Map(map) => visitor.visit_map(MapDeserializer::new(map)),
        }
    }

    fn deserialize_option<V: Visitor<'de>>(
        self,
        visitor: V,
    ) -> std::result::Result<V::Value, WrCliError> {
        visitor.visit_some(self)
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> std::result::Result<V::Value, WrCliError> {
        match self.node {
            Node::Value(ConfigValue::String(variant)) => {
                visitor.visit_enum(UnitEnumAccess { variant })
            }
            _ => Err(WrCliError::ConfigDeserializeError(
                "expected a string enum variant".to_owned(),
            )),
        }
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct newtype_struct seq tuple tuple_struct map
        struct identifier ignored_any
    }
}

struct SeqDeserializer<'a> {
    iter: std::slice::Iter<'a, ConfigValue>,
}

impl<'de, 'a> SeqAccess<'de> for SeqDeserializer<'a> {
    type Error = WrCliError;

    fn next_element_seed<T: DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> std::result::Result<Option<T::Value>, WrCliError> {
        match self.iter.next() {
            Some(value) => seed.deserialize(ConfigDeserializer::value(value)).map(Some),
            None => Ok(None),
        }
    }
}

struct MapDeserializer<'a> {
    iter: std::collections::btree_map::Iter<'a, String, SettingsEntry>,
    value: Option<&'a SettingsEntry>,
}

impl<'a> MapDeserializer<'a> {
    fn new(map: &'a SettingsMap) -> Self {
        MapDeserializer {
            iter: map.iter(),
            value: None,
        }
    }
}

impl<'de, 'a> MapAccess<'de> for MapDeserializer<'a> {
    type Error = WrCliError;

    fn next_key_seed<K: DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> std::result::Result<Option<K::Value>, WrCliError> {
        match self.iter.next() {
            Some((key, entry)) => {
                self.value = Some(entry);
                seed.deserialize(KeyDeserializer { key: key.as_str() })
                    .map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(
        &mut self,
        seed: V,
    ) -> std::result::Result<V::Value, WrCliError> {
        let entry = self.value.take().ok_or_else(|| {
            WrCliError::ConfigDeserializeError("map value requested before key".to_owned())
        })?;
        match entry {
            SettingsEntry::Value(value) => seed.deserialize(ConfigDeserializer::value(value)),
            SettingsEntry::Map(map) => seed.deserialize(ConfigDeserializer::map(map)),
        }
    }
}

struct KeyDeserializer<'a> {
    key: &'a str,
}

impl<'de, 'a> Deserializer<'de> for KeyDeserializer<'a> {
    type Error = WrCliError;

    fn deserialize_any<V: Visitor<'de>>(
        self,
        visitor: V,
    ) -> std::result::Result<V::Value, WrCliError> {
        visitor.visit_str(self.key)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple tuple_struct
        map struct enum identifier ignored_any
    }
}

struct UnitEnumAccess<'a> {
    variant: &'a str,
}

impl<'de, 'a> EnumAccess<'de> for UnitEnumAccess<'a> {
    type Error = WrCliError;
    type Variant = UnitVariantAccess;

    fn variant_seed<V: DeserializeSeed<'de>>(
        self,
        seed: V,
    ) -> std::result::Result<(V::Value, Self::Variant), WrCliError> {
        let value = seed.deserialize(KeyDeserializer { key: self.variant })?;
        Ok((value, UnitVariantAccess))
    }
}

struct UnitVariantAccess;

impl<'de> VariantAccess<'de> for UnitVariantAccess {
    type Error = WrCliError;

    fn unit_variant(self) -> std::result::Result<(), WrCliError> {
        Ok(())
    }

    fn newtype_variant_seed<T: DeserializeSeed<'de>>(
        self,
        _seed: T,
    ) -> std::result::Result<T::Value, WrCliError> {
        Err(WrCliError::ConfigDeserializeError(
            "newtype enum variants are not supported".to_owned(),
        ))
    }

    fn tuple_variant<V: Visitor<'de>>(
        self,
        _len: usize,
        _visitor: V,
    ) -> std::result::Result<V::Value, WrCliError> {
        Err(WrCliError::ConfigDeserializeError(
            "tuple enum variants are not supported".to_owned(),
        ))
    }

    fn struct_variant<V: Visitor<'de>>(
        self,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> std::result::Result<V::Value, WrCliError> {
        Err(WrCliError::ConfigDeserializeError(
            "struct enum variants are not supported".to_owned(),
        ))
    }
}

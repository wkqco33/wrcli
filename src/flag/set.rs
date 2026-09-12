use indexmap::IndexMap;
use std::collections::{HashMap, HashSet};

use super::definition::Flag;
use super::value::FlagValue;
use crate::config::Config;
use crate::error::{Result, WrCliError};

/// Redact the actual value to `***` for sensitive flags.
fn redact(flag: &Flag, value: &str) -> String {
    if flag.sensitive {
        "***".to_owned()
    } else {
        value.to_owned()
    }
}

/// Split a comma-separated value and trim whitespace. If `comma` is false, returns the original as a single element.
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

/// Container holding all flags of a single command. Preserves insertion order (for help output).
#[derive(Debug, Default, Clone)]
pub struct FlagSet {
    flags: IndexMap<String, Flag>,
    short_map: HashMap<char, String>,
    values: HashMap<String, FlagValue>,
    /// Flag names the user specified on argv (excluding values seeded from config).
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

    /// Add a flag.
    ///
    /// # Panics
    /// Panics if the name or short character conflicts with an already registered flag. A silent
    /// overwrite would hide an invalid command tree until runtime, so fail immediately at build time.
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

    /// Add only if the name is absent (for injecting persistent flags).
    pub fn add_if_absent(&mut self, flag: &Flag) {
        if !self.flags.contains_key(&flag.name) {
            self.add(flag.clone());
        }
    }

    /// Add a persistent flag inherited from a parent command (separated as Global Flags in help).
    pub(crate) fn add_inherited(&mut self, flag: &Flag) {
        if !self.flags.contains_key(&flag.name) {
            let mut inherited = flag.clone();
            inherited.inherited = true;
            self.add(inherited);
        }
    }

    /// Look up a flag definition by name.
    pub fn get_flag(&self, name: &str) -> Option<&Flag> {
        self.flags.get(name)
    }

    /// Look up the flag definition registered for a short character (for subcommand routing dry-run).
    pub(crate) fn short_flag(&self, c: char) -> Option<&Flag> {
        let name = self.short_map.get(&c)?;
        self.flags.get(name.as_str())
    }

    /// Get a flag value. Returns the default value if not set.
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

    /// Get a `u64` value (a negative `Int` yields `None`).
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

    /// Iterate over all flags in insertion order (for help output).
    pub fn flags_iter(&self) -> impl Iterator<Item = &Flag> {
        self.flags.values()
    }

    /// Iterate over only persistent flags (for propagating to subcommands).
    pub fn persistent_flags(&self) -> impl Iterator<Item = &Flag> {
        self.flags.values().filter(|f| f.persistent)
    }

    /// Iterate over only values the user entered explicitly (excluding defaults/config seeds).
    /// Used by the dispatch engine when binding Config layer 4.
    pub(crate) fn values_iter(&self) -> impl Iterator<Item = (&str, &FlagValue)> {
        self.user_set
            .iter()
            .filter_map(|k| self.values.get(k).map(|v| (k.as_str(), v)))
    }

    /// Whether the user explicitly specified this flag on argv.
    ///
    /// Returns `false` for values seeded from config or for defaults.
    pub fn is_set(&self, name: &str) -> bool {
        self.user_set.contains(name)
    }

    /// Seed flags that were not set explicitly with values from the config store.
    ///
    /// Injects only config values matching the flag's default value type.
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

    /// Parse argv tokens. Returns the remaining non-flag tokens as positional arguments.
    ///
    /// Supported forms:
    /// - `--name=value`, `--name value`
    /// - `-c value` (only the last short flag in a group may take a value)
    /// - `-abc` (a group of short flags that are all bool)
    /// - `--` sentinel (everything after is a positional argument)
    /// - A bool flag present without a value is treated as `true`
    pub fn parse(&mut self, args: Vec<String>) -> Result<Vec<String>> {
        self.parse_inner(args, true)
    }

    /// Variant for consuming parent flags that precede a subcommand.
    ///
    /// Required-flag validation runs only in the leaf command, so it is skipped here.
    pub(crate) fn parse_partial(&mut self, args: Vec<String>) -> Result<Vec<String>> {
        self.parse_inner(args, false)
    }

    fn parse_inner(&mut self, args: Vec<String>, validate_required: bool) -> Result<Vec<String>> {
        log::trace!("flag parsing start: {:?}", args);
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

    /// Copy flag values consumed in the parent into the child FlagSet.
    ///
    /// If the child redefines the same name, child parsing overwrites it. Definitions
    /// (help/completion) are not added, so parent local flags are not exposed in child help.
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
                            got: redact(flag_ref, &part),
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
            FlagValue::String(_) => {
                if flag.optional_value && s.eq_ignore_ascii_case("none") {
                    Ok(FlagValue::String(String::new()))
                } else {
                    Ok(FlagValue::String(s.to_owned()))
                }
            }
            FlagValue::Int(_) => {
                s.parse::<i64>()
                    .map(FlagValue::Int)
                    .map_err(|_| WrCliError::InvalidFlagValue {
                        flag: flag.name.clone(),
                        expected: "integer",
                        got: redact(flag, s),
                    })
            }
            FlagValue::Float(_) => {
                s.parse::<f64>()
                    .map(FlagValue::Float)
                    .map_err(|_| WrCliError::InvalidFlagValue {
                        flag: flag.name.clone(),
                        expected: "float",
                        got: redact(flag, s),
                    })
            }
            FlagValue::Bool(_) => match s {
                "true" | "1" | "yes" => Ok(FlagValue::Bool(true)),
                "false" | "0" | "no" => Ok(FlagValue::Bool(false)),
                _ => Err(WrCliError::InvalidFlagValue {
                    flag: flag.name.clone(),
                    expected: "bool (true/false/1/0/yes/no)",
                    got: redact(flag, s),
                }),
            },
            FlagValue::StringVec(_) | FlagValue::IntVec(_) => {
                // StringVec/IntVec are handled directly in parse_long, so reaching here is a bug
                Ok(FlagValue::String(s.to_owned()))
            }
        }
    }
}

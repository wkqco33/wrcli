use indexmap::IndexMap;
use std::collections::HashMap;
use crate::error::{Result, WrCliError};

/// The typed value of a flag — also serves as the type tag via the default.
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
        if let FlagValue::Bool(v) = self { Some(*v) } else { None }
    }

    pub fn as_str(&self) -> Option<&str> {
        if let FlagValue::String(v) = self { Some(v) } else { None }
    }

    pub fn as_int(&self) -> Option<i64> {
        if let FlagValue::Int(v) = self { Some(*v) } else { None }
    }

    pub fn as_float(&self) -> Option<f64> {
        if let FlagValue::Float(v) = self { Some(*v) } else { None }
    }

    pub fn as_string_vec(&self) -> Option<&[String]> {
        if let FlagValue::StringVec(v) = self { Some(v) } else { None }
    }

    pub fn as_int_vec(&self) -> Option<&[i64]> {
        if let FlagValue::IntVec(v) = self { Some(v) } else { None }
    }
}

/// A single flag definition (both local and persistent flags are stored as `Flag`).
#[derive(Debug, Clone)]
pub struct Flag {
    pub name: String,
    pub short: Option<char>,
    pub usage: String,
    pub default: FlagValue,
    pub required: bool,
    pub persistent: bool,
}

impl Flag {
    pub fn new(name: &str, default: FlagValue, usage: &str) -> Self {
        Flag {
            name: name.to_owned(),
            short: None,
            usage: usage.to_owned(),
            default,
            required: false,
            persistent: false,
        }
    }

    pub fn short(mut self, c: char) -> Self {
        self.short = Some(c);
        self
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    pub fn persistent(mut self) -> Self {
        self.persistent = true;
        self
    }
}

/// Container for all flags on a single command. Preserves insertion order for help output.
#[derive(Debug, Default, Clone)]
pub struct FlagSet {
    flags: IndexMap<String, Flag>,
    short_map: HashMap<char, String>,
    values: HashMap<String, FlagValue>,
}

impl FlagSet {
    pub fn new() -> Self {
        Default::default()
    }

    /// Add a flag definition. If a flag with the same name already exists, it is replaced.
    pub fn add(&mut self, flag: Flag) {
        if let Some(c) = flag.short {
            self.short_map.insert(c, flag.name.clone());
        }
        self.flags.insert(flag.name.clone(), flag);
    }

    /// Add a flag only if no flag with that name is already registered (used for persistent flag injection).
    /// Accepts a reference and clones only when the flag is actually absent.
    pub fn add_if_absent(&mut self, flag: &Flag) {
        if !self.flags.contains_key(&flag.name) {
            self.add(flag.clone());
        }
    }

    /// Get the definition of a flag by name.
    pub fn get_flag(&self, name: &str) -> Option<&Flag> {
        self.flags.get(name)
    }

    /// Get the parsed value for a flag, falling back to the flag's default.
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

    /// Iterate all flags in insertion order (for help output).
    pub fn flags_iter(&self) -> impl Iterator<Item = &Flag> {
        self.flags.values()
    }

    /// Iterate only persistent flags (for propagation to subcommands).
    pub fn persistent_flags(&self) -> impl Iterator<Item = &Flag> {
        self.flags.values().filter(|f| f.persistent)
    }

    /// Iterate over explicitly-set flag values — skips defaults.
    /// Used by the dispatch engine to bind CLI-provided values into Config.
    pub(crate) fn values_iter(&self) -> impl Iterator<Item = (&str, &FlagValue)> {
        self.values.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// Check whether a flag value was explicitly set (not just default).
    pub fn is_set(&self, name: &str) -> bool {
        self.values.contains_key(name)
    }

    /// Parse argv tokens. Consumes `args`, returns positional (non-flag) arguments.
    ///
    /// Supports:
    /// - `--name=value`, `--name value`
    /// - `-c value` (last short flag in a group may take a value)
    /// - `-abc` (all-bool short flag groups)
    /// - `--` sentinel (everything after is positional)
    /// - Bool flags default to `true` when present without a value
    pub fn parse(&mut self, args: Vec<String>) -> Result<Vec<String>> {
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

        // Enforce required flags
        for flag in self.flags.values() {
            if flag.required && !self.values.contains_key(&flag.name) {
                return Err(WrCliError::MissingRequiredFlag(flag.name.clone()));
            }
        }

        Ok(positional)
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

        // Split-borrow the two independent fields so we can hold a &Flag reference
        // (from `flags`) while also mutating `values` — no full Flag clone needed.
        let flags  = &self.flags;
        let values = &mut self.values;

        let flag_ref = flags.get(name).ok_or_else(|| WrCliError::UnknownFlag {
            flag: format!("--{}", name),
            command: String::new(),
        })?;

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
                    .ok_or_else(|| WrCliError::MissingRequiredFlag(flag_ref.name.clone()))?;
                // Append to existing vec
                let entry = values
                    .entry(flag_ref.name.clone())
                    .or_insert(FlagValue::StringVec(vec![]));
                if let FlagValue::StringVec(v) = entry {
                    v.push(s);
                }
                return Ok(());
            }
            FlagValue::IntVec(_) => {
                let s = value_opt
                    .or_else(|| iter.next())
                    .ok_or_else(|| WrCliError::MissingRequiredFlag(flag_ref.name.clone()))?;
                let n = s.parse::<i64>().map_err(|_| WrCliError::InvalidFlagValue {
                    flag: flag_ref.name.clone(),
                    expected: "integer",
                    got: s.clone(),
                })?;
                let entry = values
                    .entry(flag_ref.name.clone())
                    .or_insert(FlagValue::IntVec(vec![]));
                if let FlagValue::IntVec(v) = entry {
                    v.push(n);
                }
                return Ok(());
            }
            _ => {
                let s = value_opt
                    .or_else(|| iter.next())
                    .ok_or_else(|| WrCliError::MissingRequiredFlag(flag_ref.name.clone()))?;
                let parsed = Self::coerce(flag_ref, &s)?;
                values.insert(flag_ref.name.clone(), parsed);
            }
        }
        Ok(())
    }

    fn parse_short(
        &mut self,
        chars_str: &str,
        iter: &mut impl Iterator<Item = String>,
    ) -> Result<()> {
        // Split-borrow the three independent fields.
        let short_map = &self.short_map;
        let flags     = &self.flags;
        let values    = &mut self.values;

        // Use peekable to detect the last char without collecting into a Vec<char>.
        let mut chars = chars_str.chars().peekable();
        while let Some(c) = chars.next() {
            let is_last = chars.peek().is_none();

            let flag_name = short_map
                .get(&c)
                .ok_or_else(|| WrCliError::UnknownFlag {
                    flag: format!("-{}", c),
                    command: String::new(),
                })?;
            let flag_ref = flags.get(flag_name.as_str()).unwrap();

            let parsed = match &flag_ref.default {
                FlagValue::Bool(_) => FlagValue::Bool(true),
                _ => {
                    if is_last {
                        let s = iter.next().ok_or_else(|| {
                            WrCliError::MissingRequiredFlag(flag_ref.name.clone())
                        })?;
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
        }

        Ok(())
    }

    fn coerce(flag: &Flag, s: &str) -> Result<FlagValue> {
        match &flag.default {
            FlagValue::String(_) => Ok(FlagValue::String(s.to_owned())),
            FlagValue::Int(_) => s.parse::<i64>().map(FlagValue::Int).map_err(|_| {
                WrCliError::InvalidFlagValue {
                    flag: flag.name.clone(),
                    expected: "integer",
                    got: s.to_owned(),
                }
            }),
            FlagValue::Float(_) => s.parse::<f64>().map(FlagValue::Float).map_err(|_| {
                WrCliError::InvalidFlagValue {
                    flag: flag.name.clone(),
                    expected: "float",
                    got: s.to_owned(),
                }
            }),
            FlagValue::Bool(_) => match s {
                "true" | "1" | "yes" => Ok(FlagValue::Bool(true)),
                "false" | "0" | "no" => Ok(FlagValue::Bool(false)),
                _ => Err(WrCliError::InvalidFlagValue {
                    flag: flag.name.clone(),
                    expected: "bool (true/false/1/0/yes/no)",
                    got: s.to_owned(),
                }),
            },
            _ => Ok(FlagValue::String(s.to_owned())),
        }
    }
}

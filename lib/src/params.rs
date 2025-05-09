// Copyright 2024-2025 David Drysdale

//! Functionality for parsing and formatting parameters.

use crate::parser;
use log::{error, trace};
use serde::Serialize;
use std::borrow::Cow;
use std::fmt::{self, Display, Formatter};

/// Populate a [`MoveParam`].
#[macro_export]
macro_rules! param {
    { $self:ident.$pname:ident } => {
        $crate::MoveParam {
            name: stringify!($pname),
            value: $self.$pname.clone().into(),
        }
    };
    { $name:ident=$value:expr } => {
        $crate::MoveParam {
            name: stringify!($name),
            value: $value.into(),
        }
    };
    { $name:literal=$value:expr } => {
        $crate::MoveParam {
            name: $name,
            value: $value.into(),
        }
    }
}

/// A parameter for a move, with a compile-time name.
pub type MoveParam = MoveParamRef<'static>;

/// A parameter for a move, where the parameter name has a lifetime.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoveParamRef<'a> {
    /// Name of the parameter.
    pub name: &'a str,
    /// Value for the parameter.
    pub value: Value,
}

/// A parameter value may be either a number or a `String`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum Value {
    /// Numeric value.
    Number(i32),
    /// Text value, either owned or a static string.
    Text(Cow<'static, str>),
    /// Boolean value.
    Boolean(bool),
}

impl Value {
    /// Extract the numeric value.
    pub fn as_i32<'a>(&'_ self, input: &'a str) -> Result<i32, parser::Error<'a>> {
        match self {
            Value::Number(v) => Ok(*v),
            Value::Boolean(v) => {
                error!("Trying to extract number from boolean parameter '{v}'");
                Err(parser::fail(input))
            }
            Value::Text(v) => {
                error!("Trying to extract number from text parameter '{v}'");
                Err(parser::fail(input))
            }
        }
    }
    /// Extract the text value.
    pub fn as_str<'a, 'b>(&'b self, input: &'a str) -> Result<&'b str, parser::Error<'a>> {
        match self {
            Value::Number(v) => {
                error!("Trying to extract number from text parameter '{v}'!");
                Err(parser::fail(input))
            }
            Value::Boolean(v) => {
                error!("Trying to extract number from boolean parameter '{v}'!");
                Err(parser::fail(input))
            }
            Value::Text(v) => Ok(v),
        }
    }
    /// Extract the boolean value.
    pub fn as_bool<'a>(&'_ self, input: &'a str) -> Result<bool, parser::Error<'a>> {
        match self {
            Value::Number(v) => {
                error!("Trying to extract bool from text parameter '{v}'!");
                Err(parser::fail(input))
            }
            Value::Boolean(v) => Ok(*v),
            Value::Text(v) => {
                error!("Trying to extract number from text parameter '{v}'!");
                Err(parser::fail(input))
            }
        }
    }
}

impl From<i32> for Value {
    fn from(val: i32) -> Self {
        Self::Number(val)
    }
}

impl From<String> for Value {
    fn from(val: String) -> Self {
        Self::Text(val.into())
    }
}

impl From<&'static str> for Value {
    fn from(val: &'static str) -> Self {
        Self::Text(val.into())
    }
}

impl From<bool> for Value {
    fn from(val: bool) -> Self {
        Self::Boolean(val)
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(v) => write!(f, "{v}",),
            Value::Boolean(false) => write!(f, "false",),
            Value::Boolean(true) => write!(f, "true",),
            Value::Text(v) => write!(f, "\"{v}\"",),
        }
    }
}

/// Set of predefined values for numeric short codes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Detents {
    /// Value to use for one detent more than default.
    pub add1: i32,
    /// Value to use for two detents more than default.
    pub add2: i32,
    /// Value to use for three detents more than default.
    pub add3: i32,
    /// Value to use for one detent less than default.
    pub less1: i32,
    /// Value to use for two detents less than default.
    pub less2: i32,
    /// Value to use for three detents less than default.
    pub less3: i32,
}

impl Detents {
    /// Panics if count is not 1,2,3,-1,-2,-3.
    fn value(&self, count: i32) -> Value {
        match count {
            1 => self.add1.into(),
            2 => self.add2.into(),
            3 => self.add3.into(),
            -1 => self.less1.into(),
            -2 => self.less2.into(),
            -3 => self.less3.into(),
            _ => unreachable!(),
        }
    }
}

/// Abbreviated form for a parameter.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Abbrev {
    /// Numeric parameter can be specified as "+", "+++", "--" etc.
    PlusMinus(Detents),
    /// Numeric parameter can be specified  as "<", "<<<", ">>" etc.
    GreaterLess(Detents),
}

impl Abbrev {
    /// Extract the [`Detents`] for an abbreviation.
    pub fn detents(&self) -> &Detents {
        match self {
            Abbrev::PlusMinus(d) => d,
            Abbrev::GreaterLess(d) => d,
        }
    }
    /// Return the short chars for an abbreviation.
    pub fn chars(&self) -> (char, char) {
        match self {
            Abbrev::PlusMinus(_) => ('+', '-'),
            Abbrev::GreaterLess(_) => ('>', '<'),
        }
    }
}

/// Valid range for parameter values.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Range {
    /// Only text strings are allowed (resulting in `Value::Text`).
    Text,
    /// Any numeric value is allowed (resulting in `Value::Number`).
    Any,
    /// Numeric value restricted to [0, ∞).
    Positive,
    /// Numeric value restricted to (0, ∞).
    StrictlyPositive,
    /// Only boolean values are allowed (resulting in `Value::Boolean`).
    Boolean,
}

impl Range {
    /// Indicate whether the given value is valid for this range.
    pub fn valid<'a>(&'_ self, input: &'a str, val: &'_ Value) -> Result<(), parser::Error<'a>> {
        match (val, self) {
            (Value::Number(_v), Range::Any) => Ok(()),
            (Value::Number(v), Range::Text) => Err(format!("{v} unexpected, want \"string\"")),
            (Value::Number(v), Range::Positive) if *v >= 0 => Ok(()),
            (Value::Number(v), Range::Positive) => Err(format!("{v} out of range, must be >= 0")),
            (Value::Number(v), Range::StrictlyPositive) if *v > 0 => Ok(()),
            (Value::Number(v), Range::StrictlyPositive) => {
                Err(format!("{v} out of range, must be > 0"))
            }
            (Value::Number(v), Range::Boolean) => Err(format!("{v} out of range, expect boolean")),

            (Value::Text(_v), Range::Text) => Ok(()),
            (Value::Text(v), range) => Err(format!("'{v}' unexpected, want {range:?}")),

            (Value::Boolean(_v), Range::Boolean) => Ok(()),
            (Value::Boolean(v), range) => Err(format!("'{v}' unexpected, want {range:?}")),
        }
        .map_err(|e| {
            error!("{e}");
            parser::fail(input)
        })
    }
}

/// Information about a parameter for a move.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Info {
    /// Name of the parameter.
    pub name: &'static str,
    /// Documentation for the parameter.
    pub doc: &'static str,
    /// Whether the parameter can be specified in an abbreviated form.
    pub short: Option<Abbrev>,
    /// Valid range for parameter values.
    pub range: Range,
    /// Default value.
    pub default: Value,
}

/// Check that a set of move parameters is compatible with the move information.
pub fn compatible(info: &[Info], params: &[MoveParam]) -> bool {
    if info.len() != params.len() {
        error!("len mismatch: info {} params {}", info.len(), params.len());
        return false;
    }
    for idx in 0..info.len() {
        if params[idx].name != info[idx].name {
            error!(
                "[{idx}] name mismatch: info '{}' params '{}'",
                info[idx].name, params[idx].name
            );
            return false;
        }
    }
    true
}

/// Generate a minimal string describing a set of [`MoveParam`]s.
/// Assumes that the length of the two inputs is the same.
pub fn to_string(params_info: &[Info], params: &[MoveParam]) -> String {
    // Invariant: `params_info` and `params` are in sync.
    assert_eq!(params_info.len(), params.len());
    // Only allow one short code parameter of each style.
    assert!(
        params_info
            .iter()
            .filter(|info| matches!(info.short, Some(Abbrev::PlusMinus(_))))
            .count()
            <= 1
    );
    assert!(
        params_info
            .iter()
            .filter(|info| matches!(info.short, Some(Abbrev::GreaterLess(_))))
            .count()
            <= 1
    );

    let mut s = String::new();

    let mut done = vec![false; params.len()];
    for (idx, param) in params.iter().enumerate() {
        let info = &params_info[idx];
        if param.value == info.default {
            // A default value can be assumed.
            done[idx] = true;
        } else if let Some(abbrev) = info.short {
            let detents = abbrev.detents();
            let (u, d) = abbrev.chars();
            let value = param.value.as_i32("<internal>").unwrap();
            let short = if value == detents.add1 {
                format!("{u}")
            } else if value == detents.add2 {
                format!("{u}{u}")
            } else if value == detents.add3 {
                format!("{u}{u}{u}")
            } else if value == detents.less1 {
                format!("{d}")
            } else if value == detents.less2 {
                format!("{d}{d}")
            } else if value == detents.less3 {
                format!("{d}{d}{d}")
            } else {
                "".to_string()
            };
            if !short.is_empty() {
                s += &short;
                done[idx] = true;
            }
        }
    }

    if done.iter().all(|v| *v) {
        // No need for explicit params, everything is already covered.
        return s;
    }

    s += "[";
    let mut first = true;
    for (idx, param) in params.iter().enumerate() {
        if done[idx] {
            continue;
        }
        if !first {
            s += ",";
        }
        s += param.name;
        s += "=";
        s += &param.value.to_string();
        first = false;
    }
    s += "]";
    s
}

/// Populate a collection of [`MoveParam`]s from the given `input`.  Any values that are not mentioned in the input will
/// get default values.
pub fn populate<'a>(
    params_info: &'_ [Info],
    input: &'a str,
) -> Result<Vec<MoveParam>, parser::Error<'a>> {
    let (rest, (plus_minus, more_less, vals)) = crate::parser::params::parse(input)?;
    if !rest.is_empty() {
        return Err(parser::fail(input));
    }
    populate_from(params_info, input, plus_minus, more_less, vals)
}

/// Populate a collection of [`MoveParam`]s that match `params_info` from the given parsed values.
pub fn populate_from<'a>(
    params_info: &'_ [Info],
    input: &'a str,
    plus_minus: i32,
    more_less: i32,
    vals: Vec<MoveParamRef>,
) -> Result<Vec<MoveParam>, parser::Error<'a>> {
    // Begin with default values.
    // Invariant: entries in `params_info` and `params` are in sync.
    let mut params: Vec<MoveParam> = params_info
        .iter()
        .map(|info| MoveParam {
            name: info.name,
            value: info.default.clone(),
        })
        .collect();
    trace!("  start with defaults {params:?}");

    if plus_minus != 0 {
        let (idx, detents) = params_info
            .iter()
            .enumerate()
            .find_map(|(idx, info)| {
                if let Some(Abbrev::PlusMinus(detents)) = info.short {
                    Some((idx, detents))
                } else {
                    None
                }
            })
            .ok_or_else(|| parser::fail(input))?;
        params[idx].value = detents.value(plus_minus);
    }
    if more_less != 0 {
        let (idx, detents) = params_info
            .iter()
            .enumerate()
            .find_map(|(idx, info)| {
                if let Some(Abbrev::GreaterLess(detents)) = info.short {
                    Some((idx, detents))
                } else {
                    None
                }
            })
            .ok_or_else(|| parser::fail(input))?;
        params[idx].value = detents.value(more_less);
    }

    // Work through the explicitly specified parameters, transcribing valid values (overriding any value already set by
    // a short code) and rejecting invalid parameter names.
    for val in vals {
        if let Some(idx) = params.iter().enumerate().find_map(|(idx, param)| {
            if param.name == val.name {
                Some(idx)
            } else {
                None
            }
        }) {
            params_info[idx].range.valid(input, &val.value)?;
            params[idx].value = val.value;
        } else {
            return Err(parser::fail(input));
        }
    }

    trace!("  end with {params:?}");
    Ok(params)
}

#[cfg(test)]
mod tests {
    use super::*;
    const TEST_PARAMS_INFO: &[Info] = &[
        Info {
            name: "len1",
            doc: "test",
            default: Value::Number(100),
            range: Range::StrictlyPositive,
            short: Some(Abbrev::PlusMinus(Detents {
                add1: 125,
                add2: 150,
                add3: 200,
                less1: 75,
                less2: 50,
                less3: 25,
            })),
        },
        Info {
            name: "len2",
            doc: "test",
            default: Value::Number(10),
            range: Range::Positive,
            short: None,
        },
        Info {
            name: "curve",
            doc: "test",
            default: Value::Number(45),
            range: Range::Any,
            short: Some(Abbrev::GreaterLess(Detents {
                add1: 60,
                add2: 90,
                add3: 120,
                less1: 30,
                less2: 20,
                less3: 10,
            })),
        },
        Info {
            name: "boolean",
            doc: "test",
            default: Value::Boolean(false),
            range: Range::Boolean,
            short: None,
        },
    ];

    #[test]
    fn test_to_string() {
        let tests = [
            (
                vec![
                    param!(len1 = 12),
                    param!(len2 = 10),
                    param!(curve = 45),
                    param!(boolean = false),
                ],
                "[len1=12]",
            ),
            (
                vec![
                    param!(len1 = 100),
                    param!(len2 = 10),
                    param!(curve = -10),
                    param!(boolean = false),
                ],
                "[curve=-10]",
            ),
            (
                vec![
                    param!(len1 = 1),
                    param!(len2 = 2),
                    param!(curve = 3),
                    param!(boolean = true),
                ],
                "[len1=1,len2=2,curve=3,boolean=true]",
            ),
            (
                vec![
                    param!(len1 = 1),
                    param!(len2 = 10),
                    param!(curve = 46),
                    param!(boolean = false),
                ],
                "[len1=1,curve=46]",
            ),
            (
                vec![
                    param!(len1 = 125),
                    param!(len2 = 10),
                    param!(curve = 60),
                    param!(boolean = false),
                ],
                "+>",
            ),
            (
                vec![
                    param!(len1 = 200),
                    param!(len2 = 11),
                    param!(curve = 10),
                    param!(boolean = false),
                ],
                "+++<<<[len2=11]",
            ),
            (
                vec![
                    param!(len1 = 200),
                    param!(len2 = 50),
                    param!(curve = 10),
                    param!(boolean = true),
                ],
                "+++<<<[len2=50,boolean=true]",
            ),
        ];
        for (input, want) in tests {
            let got = to_string(TEST_PARAMS_INFO, &input);
            assert_eq!(got, want, "for input {input:?}");

            let recovered = populate(TEST_PARAMS_INFO, &got).unwrap();
            assert_eq!(
                recovered, input,
                "for input {input:?} round-trip via '{got}'"
            );
        }
    }

    #[test]
    fn test_populate() {
        let got = populate(TEST_PARAMS_INFO, " [len1=42]").unwrap();
        assert_eq!(
            got,
            vec![
                param!(len1 = 42),
                param!(len2 = 10),
                param!(curve = 45),
                param!(boolean = false)
            ]
        );

        let got = populate(TEST_PARAMS_INFO, " [len2=42]").unwrap();
        assert_eq!(
            got,
            vec![
                param!(len1 = 100),
                param!(len2 = 42),
                param!(curve = 45),
                param!(boolean = false)
            ]
        );
    }

    #[test]
    fn test_populate_err() {
        assert!(populate(TEST_PARAMS_INFO, " [len2=42,other=99]").is_err());
        assert!(populate(TEST_PARAMS_INFO, " [len2=-1]").is_err());
        assert!(populate(TEST_PARAMS_INFO, " [len1=-1]").is_err());
        assert!(populate(TEST_PARAMS_INFO, " [len1=0]").is_err());
        assert!(populate(TEST_PARAMS_INFO, " ++>>--").is_err());
    }
}

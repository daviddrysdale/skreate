//! Functionality for parsing and formatting parameters.

use crate::MoveParam;
use regex::Regex;
use std::collections::HashMap;

/// Populate a [`MoveParam`] from a field in `self`.
#[macro_export]
macro_rules! param {
    { $self:ident.$pname:ident } => {
        $crate::MoveParam {
            name: stringify!($pname),
            value: $self.$pname,
        }
    };
    { $name:ident=$value:expr } => {
        $crate::MoveParam {
            name: stringify!($name),
            value: $value,
        }
    };
}

/// Set of predefined values for short codes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

/// Abbreviated form for a parameter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Abbrev {
    /// No abbreviated form.
    None,
    /// Specify as "+", "+++", "--" etc.
    PlusMinus(Detents),
    /// Specify as "<", "<<<", ">>" etc.
    GreaterLess(Detents),
}

impl Abbrev {
    fn detents(&self) -> Option<&Detents> {
        match self {
            Abbrev::None => None,
            Abbrev::PlusMinus(d) => Some(d),
            Abbrev::GreaterLess(d) => Some(d),
        }
    }
    fn chars(&self) -> (char, char) {
        match self {
            Abbrev::None => ('?', '?'),
            Abbrev::PlusMinus(_) => ('+', '-'),
            Abbrev::GreaterLess(_) => ('>', '<'),
        }
    }
}

/// Valid range for parameter values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Range {
    /// Any value is allowed.
    Any,
    /// Restrict to [0, ∞)
    Positive,
    /// Restrict to (0, ∞)
    StrictlyPositive,
}

impl Range {
    /// Indicate whether the given value is valid for this range.
    pub fn valid(&self, val: i32) -> Result<i32, String> {
        match self {
            Range::Any => Ok(val),
            Range::Positive if val >= 0 => Ok(val),
            Range::Positive => Err(format!("{val} out of range, must be >= 0")),
            Range::StrictlyPositive if val > 0 => Ok(val),
            Range::StrictlyPositive => Err(format!("{val} out of range, must be > 0")),
        }
    }
}

/// Information about a parameter for a move.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Info {
    /// Name of the parameter.
    pub name: &'static str,
    /// Whether the parameter can be specified in an abbreviated form.
    pub short: Abbrev,
    /// Valid range for parameter values.
    pub range: Range,
    /// Default value.
    pub default: i32,
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
            .filter(|info| matches!(info.short, Abbrev::PlusMinus(_)))
            .count()
            <= 1
    );
    assert!(
        params_info
            .iter()
            .filter(|info| matches!(info.short, Abbrev::GreaterLess(_)))
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
        } else if let Some(detents) = info.short.detents() {
            let (u, d) = info.short.chars();
            let short = if param.value == detents.add1 {
                format!("{u}")
            } else if param.value == detents.add2 {
                format!("{u}{u}")
            } else if param.value == detents.add3 {
                format!("{u}{u}{u}")
            } else if param.value == detents.less1 {
                format!("{d}")
            } else if param.value == detents.less2 {
                format!("{d}{d}")
            } else if param.value == detents.less3 {
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

/// Parse an explicit 'name=value' parameter string.
fn param_from_string(input: &str) -> Result<(&str, i32), String> {
    let inner_re =
        Regex::new(r#"^(?P<name>[a-zA-Z_][a-zA-Z_0-9]*)\s*=\s*(?P<value>-?[0-9]+)$"#).unwrap();
    let Some(captures) = inner_re.captures(input) else {
        return Err(format!("failed to find parameter in '{input}'"));
    };
    Ok((
        captures.name("name").unwrap().as_str(),
        captures
            .name("value")
            .unwrap()
            .as_str()
            .parse::<i32>()
            .map_err(|e| format!("value not an integer: {e:?}"))?,
    ))
}

/// Populate a collection of [`MoveParam`]s from the given `input`.  Any values that are not mentioned in the input will
/// get default values.
pub fn populate(params_info: &[Info], input: &str) -> Result<Vec<MoveParam>, String> {
    // Begin with default values.
    // Invariant: entries in `params_info` and `params` are in sync.
    let mut params: Vec<MoveParam> = params_info
        .iter()
        .map(|info| MoveParam {
            name: info.name,
            value: info.default,
        })
        .collect();

    // First look for short codes, until we encounter '[' or end of string.
    let in_chars = input.trim().chars().collect::<Vec<_>>();
    struct ShortCode {
        c: char,
        count: usize,
    }
    let mut codes: Vec<ShortCode> = Vec::new();
    let mut idx = 0;
    loop {
        if idx >= in_chars.len() {
            break;
        }
        let c = in_chars[idx];
        if c == '[' {
            break;
        }
        idx += 1;
        if !matches!(c, '+' | '-' | '<' | '>') {
            return Err(format!("unexpected char '{c}' used in short code location"));
        }
        if let Some(&mut ref mut current) = codes.last_mut() {
            if c == current.c {
                current.count += 1;
                continue;
            }
        }
        codes.push(ShortCode { c, count: 1 });
    }
    if codes.len() > 2 {
        return Err(format!("{} short codes found, max is 2", codes.len()));
    }
    for code in codes {
        let (idx, info) = params_info
            .iter()
            .enumerate()
            .filter_map(|(idx, info)| {
                let (u, d) = info.short.chars();
                if code.c == u || code.c == d {
                    Some((idx, info))
                } else {
                    None
                }
            })
            .next()
            .ok_or_else(|| format!("No parameter with short abbreviation using '{}'", code.c))?;
        let (u, d) = info.short.chars();
        let detents = info.short.detents().unwrap();
        params[idx].value = match (code.c, code.count) {
            (c, 1) if c == u => detents.add1,
            (c, 2) if c == u => detents.add2,
            (c, 3) if c == u => detents.add3,
            (c, 1) if c == d => detents.less1,
            (c, 2) if c == d => detents.less2,
            (c, 3) if c == d => detents.less3,
            _ => unreachable!(),
        };
    }

    let rest: String = in_chars[idx..].iter().collect();
    if rest.is_empty() {
        return Ok(params);
    }

    // Now look for explicitly specified parameters.
    let outer_re = Regex::new(r#"^\s*\[(?P<inner>.*)\]\s*$"#).unwrap();
    let Some(captures) = outer_re.captures(&rest) else {
        return Err(format!("failed to find params in '{input}'"));
    };
    let inner = captures.name("inner").unwrap().as_str();
    let result: Result<Vec<(&str, i32)>, String> =
        inner.split(',').map(param_from_string).collect();
    let vals: HashMap<&str, i32> = result?.into_iter().collect();

    // Work through the explicitly specified parameters, transcribing valid values (overriding any value already set by
    // a short code) and rejecting invalid parameter names.
    for (name, value) in vals {
        if let Some(idx) =
            params
                .iter()
                .enumerate()
                .find_map(|(idx, param)| if param.name == name { Some(idx) } else { None })
        {
            params[idx].value = params_info[idx].range.valid(value)?;
        } else {
            return Err(format!("'{name}' is not a valid parameter name"));
        }
    }
    Ok(params)
}

#[cfg(test)]
mod tests {
    use super::*;
    const TEST_PARAMS_INFO: &[Info] = &[
        Info {
            name: "len1",
            default: 100,
            range: Range::StrictlyPositive,
            short: Abbrev::PlusMinus(Detents {
                add1: 125,
                add2: 150,
                add3: 200,
                less1: 75,
                less2: 50,
                less3: 25,
            }),
        },
        Info {
            name: "len2",
            default: 10,
            range: Range::Positive,
            short: Abbrev::None,
        },
        Info {
            name: "curve",
            default: 45,
            range: Range::Any,
            short: Abbrev::GreaterLess(Detents {
                add1: 60,
                add2: 90,
                add3: 120,
                less1: 30,
                less2: 20,
                less3: 10,
            }),
        },
    ];

    #[test]
    fn test_param_from_string() {
        let tests = [
            ("a=1", param!(a = 1)),
            ("a =1", param!(a = 1)),
            ("a = 1", param!(a = 1)),
            ("a= 1", param!(a = 1)),
            ("a=-1", param!(a = -1)),
            ("a=0", param!(a = 0)),
            ("B=10", param!(B = 10)),
            ("a1bcDEF123=123", param!(a1bcDEF123 = 123)),
        ];
        for (input, want) in tests {
            let got = param_from_string(input).unwrap();
            assert_eq!(got, (want.name, want.value), "for input '{input}'");
        }
    }

    #[test]
    fn test_param_from_string_err() {
        let tests = ["1a=123", "x=1.2", "β=1", "a_1"];
        for input in tests {
            assert!(
                param_from_string(input).is_err(),
                "unexpected success for '{input}'"
            );
        }
    }

    #[test]
    fn test_to_string() {
        let tests = [
            (
                vec![param!(len1 = 12), param!(len2 = 10), param!(curve = 45)],
                "[len1=12]",
            ),
            (
                vec![param!(len1 = 100), param!(len2 = 10), param!(curve = -10)],
                "[curve=-10]",
            ),
            (
                vec![param!(len1 = 1), param!(len2 = 2), param!(curve = 3)],
                "[len1=1,len2=2,curve=3]",
            ),
            (
                vec![param!(len1 = 1), param!(len2 = 10), param!(curve = 46)],
                "[len1=1,curve=46]",
            ),
            (
                vec![param!(len1 = 125), param!(len2 = 10), param!(curve = 60)],
                "+>",
            ),
            (
                vec![param!(len1 = 200), param!(len2 = 11), param!(curve = 10)],
                "+++<<<[len2=11]",
            ),
            (
                vec![param!(len1 = 200), param!(len2 = 50), param!(curve = 10)],
                "+++<<<[len2=50]",
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
            vec![param!(len1 = 42), param!(len2 = 10), param!(curve = 45)]
        );

        let got = populate(TEST_PARAMS_INFO, " [len2=42]").unwrap();
        assert_eq!(
            got,
            vec![param!(len1 = 100), param!(len2 = 42), param!(curve = 45)]
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

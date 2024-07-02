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
    { $name:ident } => {
        $crate::MoveParam {
            name: stringify!($name),
            value: $crate::DEFAULT_PARAM,
        }
    };
}

/// Generate a string describing a set of [`MoveParam`]s.
pub fn params_to_string(params: &[MoveParam]) -> String {
    let mut s = "[".to_string();
    let mut first = true;
    for param in params {
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
/// be left as-is.
pub fn populate_params(params: &mut Vec<MoveParam>, input: &str) -> Result<(), String> {
    let outer_re = Regex::new(r#"^\s*\[(?P<inner>.*)\]\s*$"#).unwrap();
    let Some(captures) = outer_re.captures(input) else {
        return Err(format!("failed to find params in '{input}'"));
    };
    let inner = captures.name("inner").unwrap().as_str();
    let result: Result<Vec<(&str, i32)>, String> =
        inner.split(',').map(param_from_string).collect();
    let vals: HashMap<&str, i32> = result?.into_iter().collect();
    for param in params {
        if let Some(val) = vals.get(param.name) {
            param.value = *val;
        }
    }
    // TODO: should this error if there's an unknown parameter?
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_params_to_string() {
        let tests = [
            (vec![param!(len = 12)], "[len=12]"),
            (vec![param!(len = -1)], "[len=-1]"),
            (
                vec![param!(a = 1), param!(b = 2), param!(c = 3)],
                "[a=1,b=2,c=3]",
            ),
            (
                vec![param!(len = 1), param!(curve = 45)],
                "[len=1,curve=45]",
            ),
        ];
        for (input, want) in tests {
            let got = params_to_string(&input);
            assert_eq!(got, want, "for input {input:?}");
        }
    }

    #[test]
    fn test_populate_params() {
        let mut params = vec![param!(len)];
        populate_params(&mut params, " [len=100]").unwrap();
        assert_eq!(params[0], param!(len = 100));
        populate_params(&mut params, " [len=100,other=200]").unwrap();
        assert_eq!(params[0], param!(len = 100));
    }
}

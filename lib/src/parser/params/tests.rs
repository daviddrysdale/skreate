// Copyright 2024-2025 David Drysdale

use super::*;
use crate::param;

#[test]
fn test_parse_bool() {
    let tests = [
        ("true,xy", true, ",xy"),
        ("y,xy", true, ",xy"),
        ("Y,xy", true, ",xy"),
        ("falsexy", false, "xy"),
        ("nxy", false, "xy"),
        ("Nxy", false, "xy"),
    ];

    for (input, want, want_rest) in tests {
        let (got_rest, got) =
            parse_bool(input).unwrap_or_else(|e| panic!("parse failed for input: {input}, {e:?}"));
        assert_eq!(got, want, "for input: {input}");
        assert_eq!(got_rest, want_rest, "for input: {input}");
    }
}

#[test]
fn test_parse_i32() {
    let tests = [
        ("0xy", 0, "xy"),
        ("42xy", 42, "xy"),
        ("+42xy", 42, "xy"),
        ("12345xy", 12345, "xy"),
        ("-42xy", -42, "xy"),
    ];

    for (input, want, want_rest) in tests {
        let (got_rest, got) =
            parse_i32(input).unwrap_or_else(|e| panic!("parse failed for input: {input}, {e:?}"));
        assert_eq!(got, want, "for input: {input}");
        assert_eq!(got_rest, want_rest, "for input: {input}");
    }
}

#[test]
fn test_parse_value_ok() {
    let tests = [
        ("true,xy", Value::Boolean(true), ",xy"),
        ("falsexy", Value::Boolean(false), "xy"),
        ("42xy", Value::Number(42), "xy"),
        ("-42xy", Value::Number(-42), "xy"),
        (r#""test"xy"#, Value::Text(Cow::Borrowed("test")), "xy"),
    ];

    for (input, want, want_rest) in tests {
        let (got_rest, got) =
            parse_value(input).unwrap_or_else(|e| panic!("parse failed for input: {input}, {e:?}"));
        assert_eq!(got, want, "for input: {input}");
        assert_eq!(got_rest, want_rest, "for input: {input}");
    }
}

#[test]
fn test_parse_name() {
    let tests = [
        ("abc,xy", "abc", ",xy"),
        ("A=y", "A", "=y"),
        ("abc1234=true", "abc1234", "=true"),
        ("abc_1234=true", "abc_1234", "=true"),
        ("abc-1234=true", "abc-1234", "=true"),
    ];

    for (input, want, want_rest) in tests {
        let (got_rest, got) =
            parse_name(input).unwrap_or_else(|e| panic!("parse failed for input: {input}, {e:?}"));
        assert_eq!(got, want, "for input: {input}");
        assert_eq!(got_rest, want_rest, "for input: {input}");
    }
}

#[test]
fn test_parse_name_value() {
    let tests = [
        ("a=1", param!(a = 1)),
        ("a =1", param!(a = 1)),
        ("a = 1", param!(a = 1)),
        ("a= 1", param!(a = 1)),
        ("a=-1", param!(a = -1)),
        ("a=0", param!(a = 0)),
        ("B=10", param!(B = 10)),
        ("a1bcDEF123=123", param!(a1bcDEF123 = 123)),
        ("t = \"text\"", param!(t = "text")),
        ("t= \"text\"", param!(t = "text")),
        ("t =\"text\"", param!(t = "text")),
        ("t=\"text\"", param!(t = "text")),
        ("t=\"\"", param!(t = "")),
        ("b=true", param!(b = true)),
        ("b_c_d=true", param!("b_c_d" = true)),
        ("b-c-d=true", param!("b-c-d" = true)),
        ("b=Y", param!(b = true)),
        ("b=y", param!(b = true)),
        ("b=false", param!(b = false)),
        ("b=n", param!(b = false)),
        ("b=N", param!(b = false)),
    ];
    for (input, want) in tests {
        let (_got_rest, got) = parse_name_value(input)
            .unwrap_or_else(|e| panic!("parse failed for input '{input}': {e:?}"));
        assert_eq!(got, want, "for input '{input}'");
    }
}

#[test]
fn test_parse_name_values() {
    let tests = [
        ("[a=1]", vec![param!(a = 1)]),
        ("[a=1 , b2=123]", vec![param!(a = 1), param!(b2 = 123)]),
        ("[a=1,b2=123]", vec![param!(a = 1), param!(b2 = 123)]),
        ("[a=1, b2=123]", vec![param!(a = 1), param!(b2 = 123)]),
        ("[a=1 ,b2=123]", vec![param!(a = 1), param!(b2 = 123)]),
        ("[a=1 ,b2 = y]", vec![param!(a = 1), param!(b2 = true)]),
        ("[a=1 ,b2 = y ]", vec![param!(a = 1), param!(b2 = true)]),
    ];
    for (input, want) in tests {
        let (_got_rest, got) = parse_name_values(input)
            .unwrap_or_else(|e| panic!("parse failed for input '{input}': {e:?}"));
        assert_eq!(got, want, "for input '{input}'");
    }
}

#[test]
fn test_parse_name_value_err() {
    let tests = [
        "1a=123",  // Name must start with a letter
        "β=1",     // ASCII only
        "a_1",     // Missing value
        "b=flase", // Invalid boolean
    ];
    for input in tests {
        assert!(
            parse_name_value(input).is_err(),
            "unexpected success for '{input}'"
        );
    }
}

#[test]
fn test_parse_plus_minus() {
    let tests = [
        ("+", 1, ""),
        ("+A", 1, "A"),
        ("++A", 2, "A"),
        ("+++A", 3, "A"),
        ("---A", -3, "A"),
        ("abcxy", 0, "abcxy"),
    ];
    for (input, want, want_rest) in tests {
        let (got_rest, got) = parse_plus_minus(input)
            .unwrap_or_else(|e| panic!("parse failed for input: {input}, {e:?}"));
        assert_eq!(got, want, "for input: {input}");
        assert_eq!(got_rest, want_rest, "for input: {input}");
    }
}

#[test]
fn test_parse_short_codes() {
    let tests = [
        ("abcxy", (0, 0), "abcxy"),
        ("+A", (1, 0), "A"),
        ("++A", (2, 0), "A"),
        ("+++A", (3, 0), "A"),
        ("---A", (-3, 0), "A"),
        ("++>>A", (2, 2), "A"),
        ("++<<A", (2, -2), "A"),
        (">>A", (0, 2), "A"),
        (">>++A", (2, 2), "A"),
        ("<<++A", (2, -2), "A"),
    ];
    for (input, want, want_rest) in tests {
        let (got_rest, got) = parse_short_codes(input)
            .unwrap_or_else(|e| panic!("parse failed for input: {input}, {e:?}"));
        assert_eq!(got, want, "for input: {input}");
        assert_eq!(got_rest, want_rest, "for input: {input}");
    }
}

#[test]
fn test_parse() {
    let tests = [
        ("abcxy", (0, 0, vec![]), "abcxy"),
        ("+A", (1, 0, vec![]), "A"),
        ("<<++A", (2, -2, vec![]), "A"),
        (
            "<<++ [c=2,y=false] A",
            (2, -2, vec![param!(c = 2), param!(y = false)]),
            " A",
        ),
    ];
    for (input, want, want_rest) in tests {
        let (got_rest, got) =
            parse(input).unwrap_or_else(|e| panic!("parse failed for input: {input}, {e:?}"));
        assert_eq!(got, want, "for input: {input}");
        assert_eq!(got_rest, want_rest, "for input: {input}");
    }
}

#[test]
fn test_parse_turn_count() {
    let tests = [
        ("1xy", 2, "xy"),
        ("1.5xy", 3, "xy"),
        ("2.5xy", 5, "xy"),
        ("9xy", 18, "xy"),
    ];

    for (input, want, want_rest) in tests {
        let (got_rest, got) = parse_turn_count(input)
            .unwrap_or_else(|e| panic!("parse failed for input: {input}, {e:?}"));
        assert_eq!(got, want, "for input: {input}");
        assert_eq!(got_rest, want_rest, "for input: {input}");
    }
}

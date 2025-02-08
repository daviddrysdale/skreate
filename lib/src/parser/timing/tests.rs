use super::*;

#[test]
fn test_parse_count() {
    let tests = [
        ("0)xy", Count(0), "xy"),
        ("42)xy", Count(42), "xy"),
        ("+42)xy", Count(42), "xy"),
        ("12345)xy", Count(12345), "xy"),
        ("-42)xy", Count(-42), "xy"),
    ];

    for (input, want, want_rest) in tests {
        let (got_rest, got) =
            parse_count(input).unwrap_or_else(|e| panic!("parse failed for input: {input}, {e:?}"));
        assert_eq!(got, want, "for input: {input}");
        assert_eq!(got_rest, want_rest, "for input: {input}");
    }
}

#[test]
fn test_parse_duration() {
    let tests = [
        ("/0xy", Duration(0), "xy"),
        ("/42xy", Duration(42), "xy"),
        ("/+42xy", Duration(42), "xy"),
        ("/12345xy", Duration(12345), "xy"),
        ("/-42xy", Duration(-42), "xy"),
    ];

    for (input, want, want_rest) in tests {
        let (got_rest, got) = parse_duration(input)
            .unwrap_or_else(|e| panic!("parse failed for input: {input}, {e:?}"));
        assert_eq!(got, want, "for input: {input}");
        assert_eq!(got_rest, want_rest, "for input: {input}");
    }
}

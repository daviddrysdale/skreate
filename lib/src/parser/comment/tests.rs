// Copyright 2024-2025 David Drysdale

use super::parse;

#[test]
fn test_parse() {
    let tests = [
        ("#comment\nNext Line", "Next Line"),
        ("# comment with space \rNext Line", "Next Line"),
        ("#\nNext Line", "Next Line"),
        ("# at end of text", ""),
    ];
    for (input, want_rest) in tests {
        let (got_rest, _got) =
            parse(input).unwrap_or_else(|e| panic!("parse failed for input: {input}, {e:?}"));
        assert_eq!(got_rest, want_rest, "for input: {input}");
    }
}

use super::parse;

#[test]
fn test_parse() {
    let tests = [
        (r#""abc"xyz"#, "abc", "xyz"),
        (r#""a\"b\"c"xyz"#, "a\"b\"c", "xyz"),
    ];
    for (input, want, want_rest) in tests {
        let (got_rest, got) = parse(input).expect(&format!("parse failed for input: {input}"));
        assert_eq!(got, want, "for input: {input}");
        assert_eq!(got_rest, want_rest, "for input: {input}");
    }
}

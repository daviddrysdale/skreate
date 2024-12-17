use super::parse;

#[test]
fn test_parse() {
    let tests = [
        ("#comment\nNext Line", "\nNext Line"),
        ("# comment with space \rNext Line", "\rNext Line"),
    ];
    for (input, want_rest) in tests {
        let (got_rest, ()) =
            parse(input).unwrap_or_else(|e| panic!("parse failed for input: {input}, {e:?}"));
        assert_eq!(got_rest, want_rest, "for input: {input}");
    }
}

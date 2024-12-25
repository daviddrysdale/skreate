use super::*;

macro_rules! text_pos {
    { $row:expr, $col:expr, $count:expr } => {
        TextPosition {
            row: $row,
            col: $col,
            count: $count,
        }
    }
}

#[test]
fn test_move_locations() {
    let tests = [
        ("LFO  ", 0, text_pos!(0, 0, 3)),
        ("  LFO-Br  ", 2, text_pos!(0, 2, 6)),
        ("  \n  LFO  ", 5, text_pos!(1, 2, 3)),
    ];
    for (input, offset, want_pos) in tests {
        let (_rest, mv) = parse_move(input, &input[offset..]).unwrap();
        let got_pos = mv.text_pos().unwrap();
        assert_eq!(got_pos, want_pos, "for input '{input}'");
    }
}

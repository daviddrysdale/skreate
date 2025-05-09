// Copyright 2024-2025 David Drysdale

use super::*;

macro_rules! text_pos {
    { $row:expr, $col:expr, $count:expr } => {
        TextPosition {
            row: $row,
            col: $col,
            count: $count,
            repeat: None,
        }
    }
}

#[test]
fn test_move_locations() {
    let tests = [
        ("LFO  ", 0, text_pos!(0, 0, 3), "r_0_c_0_3"),
        ("  LFO-Br  ", 2, text_pos!(0, 2, 6), "r_0_c_2_8"),
        ("  \n  LFO  ", 5, text_pos!(1, 2, 3), "r_1_c_2_5"),
    ];
    for (input, offset, want_pos, want_id) in tests {
        let (_rest, mv) = parse_move(input, &input[offset..]).unwrap();
        let got_pos = mv.mv.text_pos().unwrap();
        assert_eq!(got_pos, want_pos, "for input '{input}'");
        let got_id = got_pos.unique_id();
        assert_eq!(got_id, want_id, "for input '{input}'");
    }
}

#[test]
fn test_valid_text() {
    let _ = env_logger::try_init();
    // All of the following should parse OK.
    let tests = [
        "|:",
        ":|",
        "!|",
        "!| x 2",
        ":|x4",
        "LBI-2S",
        "LFO",
        "LFO+",
        "LFO+>>",
        "LFO+>> # comment",
        "LFO+>> # comment\nRFI",
        "LFO [len=600]",
        "LFO [len=600] ",
        "LFO [len=600] # end of line comment after explicit params",
        "LFO [len=600] # end of line comment after explicit params\nLFO",
        "LFO [len=600] # end of line comment after explicit params\nLFO;RFI",
        "LFO\n\n# comment at end",
        "LFO\n\n# comment at end\n",
        "LFO [len=300] # comment\n",
        "Warp [x=200,y=4400,dir=270] # extend bounds\nWarp [x=200,y=4200,dir=270]",
    ];
    for input in tests {
        let result = crate::parser::parse(input);
        assert!(result.is_ok(), "for input '{input}'");
        let (rest, _moves) = result.unwrap();
        assert!(
            rest.is_empty(),
            "got '{rest}' left over after parsing '{input}'"
        );
    }
}

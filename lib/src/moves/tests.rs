//! Unit tests.

use super::*;
use crate::code;

fn check_consistent(mv: &dyn Move, input: &str) {
    assert_eq!(
        mv.pre_transition(code!(BF)).code,
        mv.start(),
        "for '{}'",
        input
    );
    assert_eq!(mv.transition().code, mv.end(), "for '{}'", input);
    assert_eq!(mv.text(), input);
}

#[test]
fn test_examples() {
    for info in INFO {
        let (_rest, mv) = crate::parser::mv::parse_move(info.example, info.example)
            .unwrap_or_else(|e| panic!("example for {} doesn't construct!: {e:?}", info.name));
        check_consistent(&*mv.mv, info.example);
    }
}

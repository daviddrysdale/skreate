//! Unit tests.

use super::*;
use crate::code;

fn check_consistent(mv: &dyn Move, input: &Input) {
    assert_eq!(
        mv.pre_transition(code!(BF)).code,
        mv.start(),
        "for '{}'",
        input.text
    );
    assert_eq!(mv.transition().code, mv.end(), "for '{}'", input.text);
    assert_eq!(mv.input(), Some(input.owned()));
    assert_eq!(mv.text(), input.text);
}

#[test]
fn test_examples() {
    for info in info() {
        let input = Input {
            pos: Default::default(),
            text: info.example,
        };
        let mv = factory(&input)
            .unwrap_or_else(|e| panic!("example for {} doesn't construct!: {e:?}", info.name));
        check_consistent(&*mv, &input);
    }
}

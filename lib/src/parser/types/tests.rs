use super::*;
use crate::code;

#[test]
fn test_parse() {
    assert_eq!(Ok(("xx", Foot::Left)), parse_foot("Lxx"));
    assert_eq!(Ok(("yy", Foot::Right)), parse_foot("Ryy"));
    assert_eq!(Ok(("xx", code!(LFO))), parse_code("LFOxx"));
    assert_eq!(Ok(("xx", code!(LF))), parse_code("LFxx"));
    assert_eq!(
        Ok(("xx", PreTransition::Wide)),
        parse_pre_transition("wd-xx")
    );
    assert_eq!(
        Ok(("xx", PreTransition::Normal)),
        parse_pre_transition("xx")
    );
}

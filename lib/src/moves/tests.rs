// Copyright 2024-2025 David Drysdale

//! Unit tests.

use super::*;
use crate::{code, params::Value};

fn check_consistent(mv: &dyn Move, input: &str) {
    assert_eq!(
        mv.pre_transition(code!(BF)).code,
        mv.start(),
        "for '{}'",
        input
    );
    assert_eq!(mv.transition().code, mv.end(), "for '{input}'");
    assert_eq!(mv.text(), input);
    let opposite = mv.opposite(None);
    let recovered = opposite.opposite(None);
    assert_eq!(
        recovered.text(),
        mv.text(),
        "for '{input}' <-> '{}'",
        opposite.text()
    );
}

#[test]
fn test_examples() {
    for info in INFO {
        let (_rest, mv) = crate::parser::mv::parse_move(info.example, info.example)
            .unwrap_or_else(|e| panic!("example for {} doesn't parse!: {e:?}", info.name));
        let mv = mv
            .construct()
            .unwrap_or_else(|e| panic!("example for {} doesn't construct!: {e:?}", info.name));
        check_consistent(&*mv.mv, info.example);
    }
}

#[test]
fn test_examples_all_params() {
    for info in INFO {
        // Build a set of parameters that has a non-default value for every param.
        let params: Vec<MoveParam> = info
            .params
            .iter()
            .map(|info| MoveParam {
                name: info.name,
                value: make_non_default(&info.default),
            })
            .collect();

        // Use the example to get a valid entry code.
        let (_rest, eg) = crate::parser::mv::parse_move(info.example, info.example)
            .unwrap_or_else(|e| panic!("example for {} doesn't parse!: {e:?}", info.name));
        let eg = eg
            .construct()
            .unwrap_or_else(|e| panic!("example for {} doesn't construct!: {e:?}", info.name));

        // Should be able to create a move from these parameter values...
        let mv = info
            .id
            .construct(
                "test",
                Default::default(),
                PreTransition::Normal,
                eg.mv.start().unwrap_or(code!(LFO)),
                params.clone(),
            )
            .unwrap_or_else(|e| {
                panic!(
                    "constructing move {:?} with {params:?} failed: {e:?}",
                    info.name
                )
            });
        // ... and the move's `text()` should parse...
        let text = mv.text();
        let (_rest, regen_mv) = crate::parser::mv::parse_move(&text, &text)
            .unwrap_or_else(|e| panic!("generated text '{text}' doesn't parse!: {e:?}",));
        let regen_mv = regen_mv
            .construct()
            .unwrap_or_else(|e| panic!("generated text '{text}' doesn't construct!: {e:?}",));
        // ... into something that re-emits the same `text()`.
        assert_eq!(text, regen_mv.mv.text());
    }
}

fn make_non_default(value: &Value) -> Value {
    match value {
        Value::Number(v) => Value::Number(v + 1),
        Value::Boolean(b) => Value::Boolean(!b),
        Value::Text(t) => Value::Text(format!("{t}LFO").into()),
    }
}

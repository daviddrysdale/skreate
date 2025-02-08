//! Compound move definition.
use crate::{
    moves::{MoveId, SkatingMoveId},
    params,
    params::Value,
    Bounds, Code, Label, Move, MoveParam, RenderOptions, Rotation, Skater, SpatialTransition,
    SvgId, TextPosition, Transition,
};
use std::borrow::Cow;
use svg::node::element::Group;
use svg::Document;

pub struct Compound {
    // Invariant: `moves` is assumed non-empty throughout.
    // Invariant: only `moves[0]` can have a pre-transition.
    moves: Vec<Box<dyn Move>>,
    start_code: Code,

    id: MoveId,
    params: Vec<MoveParam>,
    text: String,
    text_pos: TextPosition,
    move_for_count: Option<usize>,
}

impl Compound {
    /// Create a compound move.
    ///
    /// Ignores any pre-transitions other than for first constituent move.
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - `moves` has fewer than 2 entries
    /// - any move has an absolute transition
    pub fn new(
        input: &str,
        text_pos: TextPosition,
        id: SkatingMoveId,
        moves: Vec<Box<dyn Move>>,
        params: Vec<MoveParam>,
        text: String,
    ) -> Self {
        Compound::new_with_count_idx(input, text_pos, id, moves, params, text, Some(0))
    }

    /// Create a compound move, identifying which one gets count labels.
    ///
    /// Ignores any pre-transitions other than for first constituent move.
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - `moves` has fewer than 2 entries
    /// - any move has an absolute transition
    pub fn new_with_count_idx(
        _input: &str,
        text_pos: TextPosition,
        id: SkatingMoveId,
        moves: Vec<Box<dyn Move>>,
        params: Vec<MoveParam>,
        text: String,
        move_for_count: Option<usize>,
    ) -> Self {
        assert!(moves.len() >= 2);
        let mut start_code = None;
        for (idx, mv) in moves.iter().enumerate() {
            let trans = mv.transition();
            assert!(matches!(
                trans.spatial,
                SpatialTransition::Relative {
                    delta: _,
                    rotate: _
                }
            ));
            if idx == 0 {
                start_code = trans.code;
                continue;
            }
        }

        Self {
            text_pos,
            moves,
            start_code: start_code.expect("first move must have code"),
            id: MoveId::Skating(id),
            params,
            text,
            move_for_count,
        }
    }

    fn for_each_move<F>(&self, op: F)
    where
        F: FnMut(&Skater, usize, &Box<dyn Move>),
    {
        self.for_each_move_from(&Skater::at_zero(self.start_code), op)
    }

    fn for_each_move_from<F>(&self, start: &Skater, mut op: F)
    where
        F: FnMut(&Skater, usize, &Box<dyn Move>),
    {
        let mut skater = *start;
        for (idx, mv) in self.moves.iter().enumerate() {
            op(&skater, idx, mv);
            skater = skater + mv.transition();
        }
    }
}

impl Move for Compound {
    fn id(&self) -> MoveId {
        self.id
    }
    fn params(&self) -> Vec<MoveParam> {
        self.params.clone()
    }
    fn start(&self) -> Option<Code> {
        self.moves[0].start()
    }
    fn end(&self) -> Option<Code> {
        self.moves[self.moves.len() - 1].end()
    }
    fn text(&self) -> String {
        self.text.clone()
    }
    fn text_pos(&self) -> Option<TextPosition> {
        Some(self.text_pos)
    }
    fn pre_transition(&self, from: Code) -> Transition {
        self.moves[0].pre_transition(from)
    }
    fn transition(&self) -> Transition {
        let mut skater = Skater::at_zero(self.start_code);
        for mv in &self.moves {
            // This assumes no pre-transitions other than `move[0]`.
            skater = skater + mv.transition();
        }
        Transition {
            spatial: SpatialTransition::Relative {
                delta: skater.pos,
                rotate: Rotation(skater.dir.0 as i32),
            },
            code: Some(skater.code),
        }
    }
    fn bounds(&self, before: &Skater) -> Option<Bounds> {
        let mut bounds = Bounds::default();
        self.for_each_move_from(before, |skater, _idx, mv| {
            if let Some(mv_bounds) = mv.bounds(skater) {
                bounds.encompass_bounds(&mv_bounds);
            }
        });
        Some(bounds)
    }
    fn defs(&self, opts: &mut RenderOptions) -> Vec<(SvgId, Group)> {
        // Definitions are each relative to (0,0) at 0° so no need for translation.
        let id = self.text();
        let mut result = Vec::new();
        for (mv_idx, mv) in self.moves.iter().enumerate() {
            let mv_ns = SvgId(format!("{id}_{mv_idx}"));
            for (grp_id, grp) in mv.defs(opts) {
                // The ID for the inner definition is the original ID prefixed by
                // the compound move's ID and inner move index.
                let id = grp_id.in_ns(&mv_ns);
                result.push((id, grp));
            }
        }
        result
    }
    fn labels(&self, opts: &RenderOptions) -> Vec<Label> {
        let mut result = Vec::new();
        self.for_each_move(|skater, _idx, mv| {
            let labels = mv.labels(opts);
            for mut label in labels {
                // Labels have positions relative to (0,0) at 0°, so need to be
                // translated relative to accumulated position and direction.
                let fake_skater = *skater
                    + Transition {
                        spatial: SpatialTransition::Relative {
                            delta: label.pos,
                            rotate: Rotation(0),
                        },
                        code: None,
                    };
                label.pos = fake_skater.pos;

                result.push(label);
            }
        });
        result
    }
    fn render(
        &self,
        mut doc: Document,
        start: &Skater,
        opts: &mut RenderOptions,
        ns: Option<&SvgId>,
    ) -> Document {
        let count_marker = opts.count;
        let duration_marker = opts.duration;
        let id = self.text();
        let mut skater = *start;
        for (idx, mv) in self.moves.iter().enumerate() {
            let mv_ns = SvgId(format!("{id}_{idx}"));
            let ns = match ns {
                Some(outer) => mv_ns.in_ns(outer),
                None => mv_ns,
            };

            // Only render timing information on one component.
            if Some(idx) == self.move_for_count {
                opts.count = count_marker;
                opts.duration = duration_marker;
            } else {
                opts.count = None;
                opts.duration = None;
            }

            doc = mv.render(doc, &skater, opts, Some(&ns));
            skater = skater + mv.transition();
        }
        doc
    }
}

/// Generate move parameters for a two-part compound move.
#[allow(clippy::too_many_arguments)]
pub const fn params(
    aless3: i32,
    aless2: i32,
    aless1: i32,
    adflt: i32,
    aadd1: i32,
    aadd2: i32,
    aadd3: i32,
    lless3: i32,
    lless2: i32,
    lless1: i32,
    ldflt: i32,
    ladd1: i32,
    ladd2: i32,
    ladd3: i32,
) -> [params::Info; 6] {
    [
        params::Info {
            name: "angle",
            doc: "Angle of rotation for each curved part, in degrees",
            default: Value::Number(adflt),
            range: params::Range::StrictlyPositive,
            short: Some(params::Abbrev::GreaterLess(params::Detents {
                add1: aadd1,
                add2: aadd2,
                add3: aadd3,
                less1: aless1,
                less2: aless2,
                less3: aless3,
            })),
        },
        params::Info {
            name: "len",
            doc: "Length of each curved part in centimetres",
            default: Value::Number(ldflt),
            range: params::Range::StrictlyPositive,
            short: Some(params::Abbrev::PlusMinus(params::Detents {
                add1: ladd1,
                add2: ladd2,
                add3: ladd3,
                less1: lless1,
                less2: lless2,
                less3: lless3,
            })),
        },
        params::Info {
            name: "delta-angle",
            doc: "Difference in angle for second curved part, in degrees",
            default: Value::Number(0),
            range: params::Range::Any,
            short: None,
        },
        params::Info {
            name: "delta-len",
            doc: "Difference in length for second curved part, in centimetres",
            default: Value::Number(0),
            range: params::Range::Any,
            short: None,
        },
        params::Info {
            name: "style",
            doc: "Style of line",
            default: Value::Text(Cow::Borrowed("")),
            range: params::Range::Text,
            short: None,
        },
        params::Info {
            name: "transition-label",
            doc: "Replacement transition label, used if non-empty",
            default: Value::Text(Cow::Borrowed("")),
            range: params::Range::Text,
            short: None,
        },
    ]
}

/// Generate move parameters for a two-part compound move with a flat join.
#[allow(clippy::too_many_arguments)]
pub const fn params_flat(
    aless3: i32,
    aless2: i32,
    aless1: i32,
    adflt: i32,
    aadd1: i32,
    aadd2: i32,
    aadd3: i32,
    lless3: i32,
    lless2: i32,
    lless1: i32,
    ldflt: i32,
    ladd1: i32,
    ladd2: i32,
    ladd3: i32,
) -> [params::Info; 7] {
    [
        params::Info {
            name: "angle",
            doc: "Angle of rotation for each curved part, in degrees",
            default: Value::Number(adflt),
            range: params::Range::StrictlyPositive,
            short: Some(params::Abbrev::GreaterLess(params::Detents {
                add1: aadd1,
                add2: aadd2,
                add3: aadd3,
                less1: aless1,
                less2: aless2,
                less3: aless3,
            })),
        },
        params::Info {
            name: "len",
            doc: "Length of each curved part in centimetres",
            default: Value::Number(ldflt),
            range: params::Range::StrictlyPositive,
            short: Some(params::Abbrev::PlusMinus(params::Detents {
                add1: ladd1,
                add2: ladd2,
                add3: ladd3,
                less1: lless1,
                less2: lless2,
                less3: lless3,
            })),
        },
        params::Info {
            name: "delta-angle",
            doc: "Difference in angle for second curved part, in degrees",
            default: Value::Number(0),
            range: params::Range::Any,
            short: None,
        },
        params::Info {
            name: "delta-len",
            doc: "Difference in length for second curved part, in centimetres",
            default: Value::Number(0),
            range: params::Range::Any,
            short: None,
        },
        params::Info {
            name: "style",
            doc: "Style of line",
            default: Value::Text(Cow::Borrowed("")),
            range: params::Range::Text,
            short: None,
        },
        params::Info {
            name: "transition-label",
            doc: "Replacement transition label, used if non-empty",
            default: Value::Text(Cow::Borrowed("")),
            range: params::Range::Text,
            short: None,
        },
        params::Info {
            name: "flat-len",
            doc: "Length between edges in centimetres",
            default: Value::Number(50),
            range: params::Range::StrictlyPositive,
            short: None,
        },
    ]
}

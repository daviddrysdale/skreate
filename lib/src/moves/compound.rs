//! Compound move definition.
use crate::{
    use_at, Bounds, Code, Input, Label, Move, MoveParam, OwnedInput, RenderOptions, Rotation,
    Skater, SpatialTransition, Transition,
};
use svg::node::element::Group;
use svg::Document;

pub struct Compound {
    input: OwnedInput,
    // Invariant: `moves` is assumed non-empty throughout.
    // Invariant: only `moves[0]` can have a pre-transition.
    moves: Vec<Box<dyn Move>>,
    start_code: Code,

    params: Vec<MoveParam>,
    text: String,
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
        input: &Input,
        moves: Vec<Box<dyn Move>>,
        params: Vec<MoveParam>,
        text: String,
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
            input: input.owned(),
            moves,
            start_code: start_code.expect("first move must have code"),
            params,
            text,
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
    fn input(&self) -> Option<OwnedInput> {
        Some(self.input.clone())
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
    fn defs(&self, opts: &mut RenderOptions) -> Vec<Group> {
        // Definitions are each relative to (0,0) at 0° so no need for translation.
        self.moves.iter().flat_map(|mv| mv.defs(opts)).collect()
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
    fn render(&self, mut doc: Document, start: &Skater, opts: &mut RenderOptions) -> Document {
        let base_id = self.text();

        let mut links = Vec::new();
        self.for_each_move_from(start, |skater, idx, _mv| {
            let id = format!("{base_id}_{idx}");
            links.push(use_at(skater, &id, opts));
        });
        for link in links {
            doc = doc.add(link);
        }
        self.render_labels(doc, start, opts)
    }
}

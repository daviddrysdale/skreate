//! Pseudo-move definition for diagram info.

use crate::{
    param, params, params::Value, Bounds, Code, Edge, Foot, Input, Label, Move, MoveParam,
    OwnedInput, ParseError, RenderOptions, Skater, SkatingDirection, Transition,
};
use svg::node::element::Group;

pub struct Info {
    input: OwnedInput,
    title: String,
    extra: String,
    debug: bool,
}

const NAME: &str = "Info";

impl Info {
    const PARAMS_INFO: &'static [params::Info] = &[
        params::Info {
            name: "title",
            default: Value::Text("".to_string()),
            range: params::Range::Text,
            short: params::Abbrev::None,
        },
        params::Info {
            name: "extra",
            default: Value::Text("".to_string()),
            range: params::Range::Text,
            short: params::Abbrev::None,
        },
        params::Info {
            name: "debug",
            default: Value::Boolean(false),
            range: params::Range::Boolean,
            short: params::Abbrev::None,
        },
    ];
    pub fn construct(input: &Input) -> Result<Box<dyn Move>, ParseError> {
        let Some(rest) = input.text.strip_prefix(NAME) else {
            return Err(ParseError {
                pos: input.pos,
                msg: format!("No {NAME} prefix"),
            });
        };
        let params = params::populate(Self::PARAMS_INFO, rest).map_err(|msg| ParseError {
            pos: input.pos,
            msg,
        })?;

        Ok(Box::new(Self {
            input: input.owned(),
            title: params[0].value.as_str().unwrap().to_string(),
            extra: params[1].value.as_str().unwrap().to_string(),
            debug: params[2].value.as_bool().unwrap(),
        }))
    }
}

impl Move for Info {
    fn params(&self) -> Vec<MoveParam> {
        vec![param!(self.title), param!(self.extra), param!(self.debug)]
    }
    fn start(&self) -> Code {
        Code {
            foot: Foot::Both,
            dir: SkatingDirection::Forward,
            edge: Edge::Flat,
        }
    }
    fn end(&self) -> Code {
        self.start()
    }
    fn text(&self) -> String {
        let params = params::to_string(Self::PARAMS_INFO, &self.params());
        format!("{NAME} {params}")
    }
    fn input(&self) -> Option<OwnedInput> {
        Some(self.input.clone())
    }
    fn pre_transition(&self, _from: Code) -> Transition {
        Transition {
            delta: Default::default(),
            rotate: Default::default(),
            code: self.start(),
        }
    }
    fn transition(&self) -> Transition {
        Transition {
            delta: self.start,
            rotate: crate::Rotation(self.start_dir.0 as i32),
            code: self.start(),
        }
    }
    fn encompass_bounds(
        &self,
        skater: &Skater,
        _include_pre: bool,
        _bounds: &mut Bounds,
    ) -> Skater {
        *skater
    }
    fn def(&self, _opts: &RenderOptions) -> Group {
        Group::new()
    }
    fn labels(&self, _opts: &RenderOptions) -> Vec<Label> {
        Vec::new()
    }
}

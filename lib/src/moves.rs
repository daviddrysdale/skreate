use crate::{Input, Move, RenderOptions, Skater};
use svg::{node::element::Path, Document};

pub struct Lf {
    input: String,
}

impl Lf {
    pub fn new(input: &Input) -> Self {
        Self {
            input: input.text.to_string(),
        }
    }
}
impl Move for Lf {
    fn transition(&self, start: &Skater) -> Skater {
        let mut end = start.clone();
        end.pos.y += 100;
        end
    }

    fn render(&self, doc: Document, start: &Skater, _opts: &RenderOptions) -> Document {
        // TODO: don't want to deal with translation and rotation on every move, need
        // to make it so that each move renders as if it's at (0, 0) with direction 0.
        // Either with code helpers or with SVG translation/transformation.
        doc.add(
            Path::new()
                .set("d", format!("M {} {} l 0 100", start.pos.x, start.pos.y))
                .set("stroke", "black"), // TODO: from opts, persist?
        )
    }

    fn text(&self) -> String {
        "LF".to_string()
    }

    fn input_text(&self) -> Option<String> {
        Some(self.input.clone())
    }
}

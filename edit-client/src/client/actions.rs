mod styles;
mod caret;
mod modify;
mod identify;

pub use self::styles::*;
pub use self::caret::*;
pub use self::modify::*;
pub use self::identify::*;
use crate::walkers::*;
use failure::Error;
use oatie::doc::*;
use oatie::OT;

fn is_boundary_char(c: char) -> bool {
    c.is_whitespace() || c == '-' || c == '_'
}

fn caret_attrs(client_id: &str, focus: bool) -> Attrs {
    Attrs::Caret {
        client_id: client_id.to_string(),
        focus,
    }
}

#[derive(Clone)]
pub struct ActionContext {
    pub doc: Doc,
    pub client_id: String,
    op_result: Op,
}

impl ActionContext {
    pub fn new(doc: Doc, client_id: String) -> ActionContext {
        ActionContext {
            doc,
            client_id,
            op_result: Op::empty(),
        }
    }

    pub fn apply(mut self, op: &Op) -> Result<ActionContext, Error> {
        // update self with the op, update self doc, return new self
        self.doc = Op::apply(&self.doc, op);
        self.op_result = Op::compose(&self.op_result, op);
        Ok(self)
    }

    pub fn get_walker<'a>(&'a self, pos: Pos) -> Result<Walker<'a>, Error> {
        Walker::to_caret(&self.doc, &self.client_id, pos)
    }

    pub fn result(self) -> Op {
        self.op_result
    }
}

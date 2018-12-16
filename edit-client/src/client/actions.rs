mod styles;
mod caret;
mod modify;

pub use self::styles::*;
pub use self::caret::*;
pub use self::modify::*;
use crate::walkers::*;
use failure::Error;
use oatie::doc::*;
use oatie::OT;
use std::collections::HashSet;

fn is_boundary_char(c: char) -> bool {
    c.is_whitespace() || c == '-' || c == '_'
}

fn caret_attrs(client_id: &str, focus: bool) -> Attrs {
    hashmap! {
        "tag".to_string() => "caret".to_string(),
        "client".to_string() => client_id.to_string(),
        "focus".to_string() => if focus { "true".to_string() } else { "false".to_string() }
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

#[derive(Debug, Clone)]
pub struct CaretState {
    pub block: String,
    pub in_list: bool,
    pub styles: HashSet<Style>,
}

// Return a "caret state".
pub fn identify_block(ctx: ActionContext) -> Result<CaretState, Error> {
    // Identify selection styles.
    let styles = identify_styles(&ctx)?;

    let mut walker = ctx.get_walker(Pos::Focus)?;
    assert!(walker.back_block());
    if let Some(DocGroup(ref attrs, _)) = walker.doc().head() {
        let tag = attrs["tag"].clone();
        let mut in_list = false;
        if walker.parent() {
            if let Some(DocGroup(ref attrs_2, _)) = walker.doc().head() {
                in_list = attrs_2["tag"] == "bullet";
            }
        }
        Ok(CaretState {
            block: tag,
            in_list,
            styles,
        })
    } else {
        bail!("Expected a DocGroup from back_block");
    }
}

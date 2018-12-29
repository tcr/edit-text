use crate::walkers::*;
use failure::Error;
use oatie::doc::*;
use oatie::OT;
use oatie::rtf::*;
use oatie::stepper::DocStepper;
use super::*;

// For function reuse
#[derive(Debug)]
pub enum StyleOp {
    AddStyle(RtfStyle, Option<String>),
    RemoveStyle(RtfStyle),
}

// TODO consider removing this and just use restyle
pub fn apply_style(ctx: ActionContext, style: RtfStyle, value: Option<String>) -> Result<Op<RtfSchema>, Error> {
    restyle(ctx, vec![StyleOp::AddStyle(style, value)])
}

// TODO consider removing this and just use restyle
pub fn remove_styles(ctx: ActionContext, styles: StyleSet) -> Result<Op<RtfSchema>, Error> {
    restyle(
        ctx,
        styles
            .styles()
            .into_iter()
            .map(|style| StyleOp::RemoveStyle(style))
            .collect(),
    )
}

pub fn restyle(ctx: ActionContext, ops: Vec<StyleOp>) -> Result<Op<RtfSchema>, Error> {
    // Find start and end carets, or return if either are missing.
    let (walker_start, walker_end) = match (
        ctx.get_walker(Pos::Start),
        ctx.get_walker(Pos::End),
    ) {
        (Ok(walker_start), Ok(walker_end)) => (walker_start, walker_end),
        _ => {
            return Ok(Op::empty());
        }
    };

    // Calculate delta.
    let delta = walker_end.delta(&walker_start).unwrap_or(0);
    if delta == 0 {
        return Ok(Op::empty());
    }

    // Create style sets for adding or removing styles.
    let mut add_styles = hashset![];
    for op in &ops {
        if let &StyleOp::AddStyle(ref style, _) = op {
            add_styles.insert(style.to_owned());
        }
    }
    let mut remove_styles = hashset![];
    for op in &ops {
        if let &StyleOp::RemoveStyle(ref style) = op {
            remove_styles.insert(style.to_owned());
        }
    }

    let mut writer = walker_start.to_writer();

    // Remove styles.
    if !remove_styles.is_empty() {
        let mut doc1: DocStepper<RtfSchema> = walker_start.doc().to_owned();
        let doc2: DocStepper<RtfSchema> = walker_end.doc().to_owned();
        while doc1 != doc2 {
            match doc1.head() {
                Some(DocGroup(..)) => {
                    writer.del.begin();
                    doc1.enter();
                }
                Some(DocChars(_, ref text)) => {
                    writer
                        .del
                        .place(&DelStyles(text.char_len(), StyleSet::from(remove_styles.clone())));
                    doc1.skip(text.char_len());
                }
                None => {
                    writer.del.exit();
                    doc1.exit();
                }
            }
        }
    }

    // Add styles.
    if !add_styles.is_empty() {
        let mut doc1 = walker_start.doc().to_owned();
        let doc2 = walker_end.doc().to_owned();
        while doc1 != doc2 {
            match doc1.head() {
                Some(DocGroup(..)) => {
                    writer.add.begin();
                    doc1.enter();
                }
                Some(DocChars(_, ref text)) => {
                    writer
                        .add
                        .place(&AddStyles(text.char_len(), StyleSet::from(add_styles.clone())));
                    doc1.skip(text.char_len());
                }
                None => {
                    writer.add.exit();
                    doc1.exit();
                }
            }
        }
    }

    Ok(writer.exit_result())
}

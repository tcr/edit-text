use crate::walkers::*;
use failure::Error;
use oatie::doc::*;
use oatie::OT;
use oatie::stepper::DocStepper;
use super::*;
use std::collections::HashSet;

// For function reuse
#[derive(Debug)]
pub enum StyleOp {
    AddStyle(Style, Option<String>),
    RemoveStyle(Style),
    // ToggleStyle(Style, Option<String>),
}

// TODO consider removing this and just use restyle
pub fn apply_style(ctx: ActionContext, style: Style, value: Option<String>) -> Result<Op, Error> {
    restyle(ctx, vec![StyleOp::AddStyle(style, value)])
}

// TODO consider removing this and just use restyle
pub fn remove_styles(ctx: ActionContext, mut styles: StyleSet) -> Result<Op, Error> {
    restyle(
        ctx,
        styles
            .drain()
            .map(|style| StyleOp::RemoveStyle(style))
            .collect(),
    )
}

pub fn identify_styles(ctx: &ActionContext) -> Result<StyleSet, Error> {
    // Find start and end carets, or return if either are missing.
    let (mut walker_start, walker_end) = match (
        ctx.get_walker(Pos::Start),
        ctx.get_walker(Pos::End),
    ) {
        (Ok(walker_start), Ok(walker_end)) => (walker_start, walker_end),
        _ => {
            return Ok(hashset![]);
        }
    };

    // If we have a collapsed selection (delta of 0), identify the style of the
    // previous text element.
    let delta = walker_end.delta(&walker_start).unwrap_or(0);
    if delta == 0 {
        // Skip past adjacent carets in between cursor and the next char.
        // TODO is there a more elegant way to do this:
        loop {
            match walker_start.doc().unhead() {
                Some(DocGroup(ref attrs, _)) => {
                    // Skip over inline carets.
                    if attrs["tag"] == "caret" {
                        walker_start.stepper.doc.prev();
                    } else {
                        break;
                    }
                }
                Some(DocChars(_, ref styles)) => {
                    return Ok(styles.styles().clone());
                }
                _ => break,
            }
        }

        // Fallback.
        return Ok(hashset![]);
    }

    // Identify existing styles from selection.
    let mut existing_styles: HashSet<Style> = hashset![];
    let mut doc1: DocStepper = walker_start.doc().to_owned();
    let doc2: DocStepper = walker_end.doc().to_owned();
    while doc1 != doc2 {
        match doc1.head() {
            Some(DocGroup(..)) => {
                doc1.enter();
            }
            Some(DocChars(ref text, ref styles)) => {
                existing_styles.extend(&styles.styles());
                doc1.skip(text.char_len());
            }
            None => {
                doc1.exit();
            }
        }
    }
    return Ok(existing_styles);
}

pub fn restyle(ctx: ActionContext, ops: Vec<StyleOp>) -> Result<Op, Error> {
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
    let mut add_styles = hashmap![];
    for op in &ops {
        if let &StyleOp::AddStyle(ref style, ref value) = op {
            add_styles.insert(style.to_owned(), value.clone());
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
        let mut doc1: DocStepper = walker_start.doc().to_owned();
        let doc2: DocStepper = walker_end.doc().to_owned();
        while doc1 != doc2 {
            match doc1.head() {
                Some(DocGroup(..)) => {
                    writer.del.begin();
                    doc1.enter();
                }
                Some(DocChars(ref text, _)) => {
                    writer
                        .del
                        .place(&DelStyles(text.char_len(), remove_styles.clone()));
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
                Some(DocChars(ref text, _)) => {
                    writer
                        .add
                        .place(&AddStyles(text.char_len(), add_styles.clone()));
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

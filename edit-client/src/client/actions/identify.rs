use crate::walkers::*;
use failure::Error;
use oatie::doc::*;
use oatie::stepper::DocStepper;
use super::*;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct CaretState {
    pub block: String,
    pub in_list: bool,
    pub styles: HashSet<Style>,
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
                    if let Attrs::Caret { .. } = attrs {
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

// Return a "caret state".
pub fn identify_block(ctx: ActionContext) -> Result<CaretState, Error> {
    // Identify selection styles.
    let styles = identify_styles(&ctx)?;

    let mut walker = ctx.get_walker(Pos::Focus)?;
    assert!(walker.back_block());
    if let Some(DocGroup(ref attrs, _)) = walker.doc().head() {
        let tag = match attrs {
            Attrs::Header(level) => format!("h{}", level),
            Attrs::Html => format!("html"),
            Attrs::Code => format!("pre"),
            Attrs::Rule => format!("hr"),
            Attrs::Caret { .. } => format!("caret"),
            Attrs::Text => format!("p"),
            Attrs::ListItem => format!("bullet"),
        };
        let mut in_list = false;
        if walker.parent() {
            if let Some(DocGroup(ref attrs_2, _)) = walker.doc().head() {
                in_list = *attrs_2 == Attrs::ListItem
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

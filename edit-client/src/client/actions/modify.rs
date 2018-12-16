use super::*;
use crate::walkers::*;
use failure::Error;
use oatie::doc::*;
use oatie::OT;
use oatie::style::OpaqueStyleMap;

// Insert a string at the user's caret position.
pub fn add_string(ctx: ActionContext, input: &str) -> Result<ActionContext, Error> {
    Ok(ctx)
        .and_then(delete_selection)
        .and_then(|(_success, ctx)| {
            // Insert before start caret (given the carets are now collapsed).
            let walker = ctx.get_walker(Pos::Start)?;

            // Clone styles of hte previous text node, or use default styles.
            let mut styles = hashmap!{ Style::Normie => None };
            let mut char_walker = walker.clone();
            char_walker.back_char();
            if let Some(DocChars(_, ref prefix_styles)) = char_walker.doc().head() {
                styles.extend(
                    prefix_styles
                        .iter()
                        .map(|(a, b)| (a.to_owned(), b.to_owned())),
                );
            }

            // Insert new character.
            let mut writer = walker.to_writer();
            let step = AddChars(DocString::from_str(input), OpaqueStyleMap::from(styles));
            writer.add.place(&step);
            ctx.apply(&writer.exit_result())
        })
}

pub fn toggle_list(ctx: ActionContext) -> Result<Op, Error> {
    // Create a walker that points to the beginning of the block the caret
    // is currently in.
    let mut walker = ctx.get_walker(Pos::Focus).expect("Expected a Focus caret");
    assert!(walker.back_block());

    // If the parent of our current block is a list item, delete it.
    let mut parent_walker = walker.clone();
    if parent_walker.parent() {
        if let Some(DocGroup(ref attrs, ref span)) = parent_walker.doc().head() {
            if let Attrs::ListItem = attrs {
                // Delete the bullet group.
                let mut writer = parent_walker.to_writer();
                writer
                    .del
                    .place(&DelGroup(del_span![DelSkip(span.skip_len())]));
                return Ok(writer.exit_result());
            }
        }
    }

    // Wrap current block with a bullet group.
    Ok({
        let mut writer = walker.to_writer();
        writer.add.place(&AddGroup(
            Attrs::ListItem,
            add_span![AddSkip(1)],
        ));
        writer.exit_result()
    })
}

/// Replaces the current block with a new block.
pub fn replace_block(ctx: ActionContext, attrs: Attrs) -> Result<Op, Error> {
    // Create a walker that points to the beginning of the block the caret
    // is currently in.
    let mut walker = ctx.get_walker(Pos::Focus).expect("Expected a Focus caret");
    assert!(walker.back_block());

    // Get the skip length of the contents of the current block.
    let len = match walker.doc().head() {
        Some(DocGroup(_, ref span)) => span.skip_len(),
        _ => unreachable!(),
    };

    // Delete the current block, and the wrap its contents with a new block.
    Ok({
        let mut writer = walker.to_writer();
        writer.del.place(&DelGroup(
            del_span![DelSkip(len)],
        ));
        writer.add.place_all(&add_span![AddGroup(
            attrs,
            [AddSkip(len)],
        )]);
        writer.exit_result()
    })
}

/// Hit backspace at the beginning of a block.
fn combine_with_previous_block(
    walker: Walker<'_>,
) -> Result<Op, Error> {
    // Check for first block in a list item.
    let mut parent_walker = walker.clone();
    assert!(parent_walker.back_block());

    // Check if we are in a block inside of a list item. Also get the length
    // of the contents of the parent list item, or otherwise just use "1"
    // (indicating the current block).
    let mut is_list_item = false;
    let mut list_item_skip_len = 1;
    if parent_walker.doc().unhead() == None && parent_walker.parent() {
        if let Some(DocGroup(ref attrs_2, ref span_2)) = parent_walker.doc().head() {
            if let Attrs::ListItem = attrs_2 {
                // We are at the start of a block inside of a list item.
                is_list_item = true;
                list_item_skip_len = span_2.skip_len();
            }
        }
    }

    // Check if parent is preceded by a list item.
    // 1. If we are in a list item also, delete both and join together as one
    //    list item.
    // 2. If we are not in a list item, delete previous group and create a new
    //    group spanning its contents and our block.
    // contents of both list items.
    if let Some(DocGroup(ref attrs, ref span)) = parent_walker.doc().unhead() {
        if let Attrs::ListItem = attrs {
            // Create local copies of attributes and span length of the previous
            // bullet group.
            let attrs = attrs.to_owned();
            let skip_len = span.skip_len();

            // Move to preceding bullet.
            parent_walker.stepper.doc.prev();

            return Ok({
                let mut writer = parent_walker.to_writer();

                writer.del.begin();
                if skip_len > 0 {
                    writer.del.place(&DelSkip(skip_len));
                }
                writer.del.close();
                if is_list_item {
                    writer.del.begin();
                }
                if list_item_skip_len > 0 {
                    writer.del.place(&DelSkip(list_item_skip_len));
                }
                if is_list_item {
                    writer.del.close();
                }

                writer.add.begin();
                if skip_len + list_item_skip_len > 0 {
                    writer
                        .add
                        .place(&AddSkip(skip_len + list_item_skip_len));
                }
                writer.add.close(attrs);

                writer.exit_result()
            });
        }
    } else {
        // We think we're at the start of the document, so do nothing.
        return Ok(Op::empty());
    }

    // If we are in a list item but there is no preceding list item, unindent
    // the current list item by deleting it and preserving its contents.
    if is_list_item {
        return Ok({
            let mut writer = parent_walker.to_writer();
            writer.del.begin();
            if list_item_skip_len > 0 {
                writer.del.place(&DelSkip(list_item_skip_len));
            }
            writer.del.close();
            writer.exit_result()
        });
    }

    // Get length of current block.
    let mut block_walker = walker.clone();
    assert!(block_walker.back_block());
    let span_2 = match block_walker.stepper().head() {
        Some(DocGroup(.., span)) => span.skip_len(),
        _ => unreachable!(),
    };

    // Move to previous block to join it (or bail if we can't find it).
    if !block_walker.back_block_or_block_object() {
        return Ok(op_span!([], []));
    }

    // If previous block is an "hr", delete it.
    if let Some(DocGroup(ref attrs, _)) = block_walker.doc().head() {
        if let Attrs::Rule = attrs {
            // Remove horizontal rule.
            return Ok({
                let mut writer = block_walker.to_writer();
                writer.del.begin();
                writer.del.close();
                writer.exit_result()
            });
        }
    } else {
        unreachable!();
    }

    // Get the length and attributes of the previous block.
    let (attrs, span_1) = match block_walker.stepper().head() {
        Some(DocGroup(attrs, span)) => (attrs, span.skip_len()),
        _ => unreachable!(),
    };

    Ok({
        let mut writer = block_walker.to_writer();

        // Delete both blocks.
        writer.del.begin();
        if span_1 > 0 {
            writer.del.place(&DelSkip(span_1));
        }
        writer.del.close();
        writer.del.begin();
        if span_2 > 0 {
            writer.del.place(&DelSkip(span_2));
        }
        writer.del.close();

        // Surround both block contents with a single block.
        writer.add.begin();
        if span_1 + span_2 > 0 {
            writer.add.place(&AddSkip(span_1 + span_2));
        }
        writer.add.close(attrs.to_owned());

        writer.exit_result()
    })
}

// Deletes backward once from a provided walker position.
fn delete_char_inner(mut walker: Walker<'_>) -> Result<Op, Error> {
    // See if we can collapse this and the previous block or list item.
    if walker.at_start_of_block() {
        return combine_with_previous_block(walker);
    }

    walker.back_char();

    // Skip past adjacent carets in between cursor and the next char.
    // TODO is there a more elegant way to do this? could build this into
    // the walker itself. but really when you do back_char there should be a
    // known normalization about which position the caret will be in respect
    // to other carets.
    while let Some(DocGroup(ref attrs, _)) = walker.doc().head() {
        if let Attrs::Caret { .. } = attrs {
            walker.stepper.doc.next();
        }
    }

    // Check that we precede a character.
    if let Some(DocChars(..)) = walker.doc().head() {
        // fallthrough
    } else {
        unreachable!();
    }

    // Delete the character we skipped over.
    let mut writer = walker.to_writer();
    writer.del.place(&DelChars(1));
    Ok(writer.exit_result())
}

/// Deletes the contents of the current selection. Returns a modified context
/// and a boolean indicating if a selection existed to delete.
fn delete_selection(ctx: ActionContext) -> Result<(bool, ActionContext), Error> {
    Ok(ctx)
        .and_then(|ctx| {
            let start = ctx.get_walker(Pos::Start)?;
            let end = ctx.get_walker(Pos::End)?;
            let delta = end.delta(&start).unwrap_or(0) as usize;

            // If we found a selection, delete every character in the selection.
            // We implement this by looping until the caret distance between our
            // cursors is 0.
            // TODO: This is incredibly inefficient.
            //  1. Dont' recurse infinitely, do this in a loop.
            //  2. Skip entire DocChars components instead of one character at a time.
            Ok(
                if delta != 0 {
                    // Get real weird with it.
                    let op = delete_char_inner(end)?;
                    let ctx = ctx.apply(&op)?;
                    if delta > 1 {
                        delete_selection(ctx)?
                    } else {
                        (true, ctx)
                    }
                } else {
                    (false, ctx)
                }
            )
        })
}

/// Backspace.
pub fn delete_char(ctx: ActionContext) -> Result<Op, Error> {
    // Bail early if we delete a selection.
    let (success, ctx) = delete_selection(ctx)?;
    if success {
        return Ok(ctx.result());
    }

    // Fallback; delete backward from start caret (given the carets are collapsed).
    let walker = ctx.get_walker(Pos::Start)?;
    delete_char_inner(walker)
}

// Splits the current block at the position of the user's caret.
pub fn split_block(ctx: ActionContext, add_hr: bool) -> Result<Op, Error> {
    let walker = ctx.get_walker(Pos::Start)?;
    let skip = walker.doc().skip_len();

    // Identify the tag of the block we're splitting.
    let mut prev_walker = walker.clone();
    assert!(prev_walker.back_block());
    let previous_block_attrs = if let Some(DocGroup(attrs, _)) = prev_walker.doc().head() {
        attrs.clone()
    } else {
        unreachable!();
    };

    // Identify if we're nested inside of a bullet.
    let mut parent_walker = prev_walker.clone();
    let mut nested_bullet = false;
    if parent_walker.parent() {
        if let Some(DocGroup(ref attrs, _)) = parent_walker.doc().head() {
            if let Attrs::ListItem = attrs {
                nested_bullet = true;
            }
        }
    }

    Ok({
        let mut writer = walker.to_writer();

        if skip > 0 {
            writer.del.place(&DelSkip(skip));
        }
        writer.del.close();
        if nested_bullet {
            writer.del.close();
        }

        writer.add.close(previous_block_attrs);
        if nested_bullet {
            writer.add.close(Attrs::ListItem);
            writer.add.begin();
        }
        if add_hr {
            writer.add.begin();
            writer.add.close(Attrs::Rule);
        }
        writer.add.begin();
        if skip > 0 {
            writer.add.place(&AddSkip(skip));
        }
        writer.add.close(Attrs::Text);
        if nested_bullet {
            writer.add.close(Attrs::ListItem);
        }

        writer.exit_result()
    })
}

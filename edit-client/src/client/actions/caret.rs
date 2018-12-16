use super::*;
use crate::walkers::*;
use failure::Error;
use oatie::doc::*;
use oatie::schema::RtfSchema;
use oatie::OT;

pub fn init_caret(ctx: ActionContext) -> Result<Op, Error> {
    let mut walker = Walker::new(&ctx.doc);
    if !walker.goto_pos(0) {
        bail!("Could not insert first caret");
    }

    let mut writer = walker.to_writer();
    writer.add.begin();
    writer.add.close(caret_attrs(&ctx.client_id, true));
    writer.add.begin();
    writer.add.close(caret_attrs(&ctx.client_id, false));
    Ok(writer.exit_result())
}

/// Arrow keys move the caret.
pub fn caret_move(
    ctx: ActionContext,
    increase: bool,
    preserve_select: bool,
) -> Result<Op, Error> {
    Ok(ctx)
        .and_then(|ctx| {
            // If we aren't preserving the selection, collapse the anchor caret
            // to where the focus caret is.
            if !preserve_select {
                let op = caret_clear_optional(&ctx, Pos::Anchor);
                ctx.apply(&op)
            } else {
                Ok(ctx)
            }
        })
        .and_then(|ctx| {
            let mut walker = ctx.get_walker(Pos::Focus)?;

            // Remove focus caret and move it to next position.
            let op = Op::transform_advance::<RtfSchema>(&{
                // First operation removes the caret.
                let mut writer = walker.to_writer();
                writer.del.begin();
                writer.del.close();
                writer.exit_result()
            }, &{
                // Move the walker to the new position.
                if increase {
                    walker.next_char();
                } else {
                    walker.back_char();
                }

                // Insert the carets.
                let mut writer = walker.to_writer();
                if !preserve_select {
                    writer.add.begin();
                    writer.add.close(caret_attrs(&ctx.client_id, false));
                }
                writer.add.begin();
                writer.add.close(caret_attrs(&ctx.client_id, true));
                writer.exit_result()
            });

            ctx.apply(&op)
        })
        .map(|ctx| ctx.result())
}

pub fn caret_word_move(ctx: ActionContext, increase: bool) -> Result<Op, Error> {
    Ok(ctx)
        .and_then(|ctx| {
            let op = caret_clear_optional(&ctx, Pos::Anchor);
            ctx.apply(&op)
        })
        .and_then(|ctx| {
            let mut walker = ctx.get_walker(Pos::Focus).expect("Expected a Focus caret");

            // First operation removes the caret.
            let mut writer = walker.to_writer();
            writer.del.begin();
            writer.del.close();
            let op_1 = writer.exit_result();

            // Find the next walker position after the current word.
            if increase {
                walker.next_char();
                loop {
                    match walker.doc().head() {
                        Some(DocChars(ref text, _)) => {
                            if is_boundary_char(text.as_str().chars().next().unwrap()) {
                                break;
                            } else {
                                walker.next_char();
                            }
                        }
                        Some(DocGroup(ref attrs, _)) => {
                            if attrs["tag"] == "caret" {
                                // guess we'll stop
                                break;
                            }
                        }
                        None => {
                            // guess we'll stop
                            break;
                        }
                    }
                }
            } else {
                // Move backward.
                walker.back_char();
                loop {
                    match walker.doc().unhead() {
                        Some(DocChars(ref text, _)) => {
                            if is_boundary_char(text.as_str().chars().rev().next().unwrap()) {
                                break;
                            } else {
                                walker.back_char();
                            }
                        }
                        Some(DocGroup(ref attrs, _)) => {
                            if attrs["tag"] == "caret" {
                                // guess we'll stop
                                break;
                            }
                        }
                        None => {
                            // guess we'll stop
                            break;
                        }
                    }
                }
            }

            // Second operation inserts the new caret.
            let mut writer = walker.to_writer();
            writer.add.begin();
            writer.add.close(caret_attrs(&ctx.client_id, true));
            writer.add.begin();
            writer.add.close(caret_attrs(&ctx.client_id, false));
            let op_2 = writer.exit_result();

            // Return composed operations. Select proper order or otherwise composition
            // will be invalid.
            ctx.apply(&Op::transform_advance::<RtfSchema>(&op_1, &op_2))
        })
        .map(|ctx| ctx.result())
}

pub fn caret_select_all(ctx: ActionContext) -> Result<Op, Error> {
    Ok(Op::transform_advance::<RtfSchema>(&{
        Op::transform_advance::<RtfSchema>(&{
            // Delete focus caret.
            caret_clear(&ctx, Pos::Focus).unwrap_or_else(|_| Op::empty())
        }, &{
            // Delete anchor caret.
            caret_clear(&ctx, Pos::Anchor).unwrap_or_else(|_| Op::empty())
        })
    }, &{
        Op::transform_advance::<RtfSchema>(&{
            // Insert anchor caret at start.
            let mut start = Walker::new(&ctx.doc);
            start.goto_pos(0);

            let mut writer = start.to_writer();
            writer.add.begin();
            writer.add.close(caret_attrs(&ctx.client_id, false));
            writer.exit_result()
        }, &{
            // Insert focus caret at end.
            let mut end = Walker::new(&ctx.doc);
            end.goto_end();

            let mut writer = end.to_writer();
            writer.add.begin();
            writer.add.close(caret_attrs(&ctx.client_id, true));
            writer.exit_result()
        })
    }))
}

pub fn caret_block_move(ctx: ActionContext, increase: bool) -> Result<Op, Error> {
    let mut walker = ctx.get_walker(Pos::Focus).expect("Expected a Focus caret");

    // First operation removes the caret.
    let mut writer = walker.to_writer();
    writer.del.begin();
    writer.del.close();
    let op_1 = writer.exit_result();

    // Second operation inserts the new caret.
    if increase {
        if !walker.next_block() {
            return Ok(op_span!([], []));
        }
    } else {
        assert!(walker.back_block());
        let _ = walker.back_block(); // don't care
    }

    let mut writer = walker.to_writer();
    writer.add.begin();
    writer.add.close(caret_attrs(&ctx.client_id, false));
    writer.add.begin();
    writer.add.close(caret_attrs(&ctx.client_id, true));
    let op_2 = writer.exit_result();

    // Return composed operations. Select proper order or otherwise composition
    // will be invalid.
    Ok(Op::transform_advance::<RtfSchema>(&op_1, &op_2))
}

// Delete a caret, return its position.
// TODO delete this?
pub fn caret_clear_inner(walker: Walker<'_>) -> Result<Op, Error> {
    let mut writer = walker.to_writer();
    writer.del.begin();
    writer.del.close();
    Ok(writer.exit_result())
}

// Deletes a caret, returning its position.
pub fn caret_clear(ctx: &ActionContext, position: Pos) -> Result<Op, Error> {
    let walker = ctx.get_walker(position)?;
    caret_clear_inner(walker)
}

// TODO delete this?
fn caret_clear_optional(ctx: &ActionContext, pos: Pos) -> Op {
    caret_clear(ctx, pos).unwrap_or(Op::empty())
}

pub fn cur_to_caret(ctx: ActionContext, cur: &CurSpan, pos: Pos) -> Result<Op, Error> {
    Ok(Op::transform_advance::<RtfSchema>(&{
        // First operation removes the caret.
        caret_clear_optional(&ctx, pos)
    }, &{
        // Second operation inserts a new caret.
        let walker = Walker::to_cursor(&ctx.doc, cur);
        let mut writer = walker.to_writer();
        writer.add.begin();
        writer.add.close(caret_attrs(&ctx.client_id, if pos == Pos::Focus { true } else { false }));
        writer.exit_result()
    }))
}

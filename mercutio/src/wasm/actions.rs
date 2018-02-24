use oatie::doc::*;
use oatie::OT;
use failure::Error;
use std::char::from_u32;
use super::walkers::*;

pub struct ActionContext {
    pub doc: Doc,
    pub client_id: String,
}

pub fn toggle_list(ctx: ActionContext) -> Result<Op, Error> {
    let mut walker = Walker::to_caret(&ctx.doc, &ctx.client_id);
    assert!(walker.back_block());

    let mut parent_walker = walker.clone();
    if parent_walker.parent() {
        if let Some(DocGroup(ref attrs, ref span)) = parent_walker.doc().head() {
            if attrs["tag"] == "bullet" {
                // Do the list destructuring here
                let mut writer = parent_walker.to_writer();

                writer.del.place(&DelGroup(del_span![DelSkip(span.skip_len())]));
                writer.del.exit_all();

                writer.add.exit_all();

                return Ok(writer.result());

                // let op_1 = writer.result();

                // assert!(parent_walker.parent());

                // if let Some(DocGroup(ref attrs, ref span)) = parent_walker.doc().head() {
                //     assert_eq!(attrs["tag"], "ul");

                //     let mut writer = parent_walker.to_writer();

                //     writer.del.group(&del_span![DelSkip(1)]);
                //     writer.del.exit_all();

                //     writer.add.exit_all();

                //     let op_2 = writer.result();

                //     return Ok(Operation::compose(&op_1, &op_2));

                // } else {
                //     unreachable!();
                // }
            }
        }
    }

    let mut writer = walker.to_writer();

    writer.del.exit_all();

    writer.add.place(&AddGroup(
        hashmap! { "tag".to_string() => "bullet".to_string() },
        add_span![AddSkip(1)],
    ));
    writer.add.exit_all();

    Ok(writer.result())
}

pub fn replace_block(ctx: ActionContext, tag: &str) -> Result<Op, Error> {
    let mut walker = Walker::to_caret(&ctx.doc, &ctx.client_id);
    assert!(walker.back_block());

    let len = if let Some(DocGroup(_, ref span)) = walker.doc().head() {
        span.skip_len()
    } else {
        unreachable!()
    };

    let mut writer = walker.to_writer();

    writer.del.place(&DelGroup(del_span![DelSkip(len)]));
    writer.del.exit_all();

    writer.add.place(&AddGroup(
        hashmap! { "tag".to_string() => tag.to_string() },
        add_span![AddSkip(len)],
    ));
    writer.add.exit_all();

    Ok(writer.result())
}

pub fn delete_char(ctx: ActionContext) -> Result<Op, Error> {
    let mut walker = Walker::to_caret(&ctx.doc, &ctx.client_id);

    // Check if caret is at the start of a block.
    let caret_pos = walker.caret_pos();
    let mut block_walker = walker.clone();
    assert!(block_walker.back_block());
    block_walker.stepper.next(); // re-enter the block to first caret position
    let at_start_of_block = caret_pos == block_walker.caret_pos();

    // See if we can collapse this and the previous block or list item.
    if at_start_of_block {
        // Check for first block in a list item.
        let mut parent_walker = walker.clone();
        assert!(parent_walker.back_block());
        if parent_walker.doc().unhead() == None && parent_walker.parent() {
            if let Some(DocGroup(ref attrs_2, ref span_2)) = parent_walker.doc().head() {
                if attrs_2["tag"] == "bullet" {
                    // Do the list destructuring here
                    println!("BEGINNING OF LIST");


                    // Check if previous sibling is a list item too.
                    if let Some(DocGroup(ref attrs_1, ref span_1)) = parent_walker.doc().unhead() {
                        if attrs_1["tag"] == "bullet" {
                            parent_walker.stepper.doc.prev();
                            let mut writer = parent_walker.to_writer();

                            writer.del.begin();
                            if span_1.skip_len() > 0 {
                                writer.del.place(&DelSkip(span_1.skip_len()));
                            }
                            writer.del.close();
                            writer.del.begin();
                            if span_2.skip_len() > 0 {
                                writer.del.place(&DelSkip(span_2.skip_len()));
                            }
                            writer.del.close();
                            writer.del.exit_all();

                            writer.add.begin();
                            if span_1.skip_len() + span_2.skip_len() > 0 {
                                writer.add.place(&AddSkip(span_1.skip_len() + span_2.skip_len()));
                            }
                            writer.add.close(attrs_1.clone());
                            writer.add.exit_all();

                            let res = writer.result();

                            return Ok(res);
                        }
                    }

                    // Unindent
                    println!("REMOVING SELF BULLET");

                    let mut writer = parent_walker.to_writer();

                    writer.del.begin();
                    if span_2.skip_len() > 0 {
                        writer.del.place(&DelSkip(span_2.skip_len()));
                    }
                    writer.del.close();
                    writer.del.exit_all();

                    let res = writer.result();
                    
                    return Ok(res);
                }
            }
        }

        // Return to block parent.
        assert!(block_walker.back_block());
        let span_2 = match block_walker.stepper().head() {
            Some(DocGroup(.., span)) => span.skip_len(),
            _ => unreachable!(),
        };

        let last_doc_stack = block_walker.doc().stack.clone();

        // Move to prior block to join it, or abort.
        if !block_walker.back_block() {
            return Ok(op_span!([], []));
        }

        let next_doc_stack = block_walker.doc().stack.clone();

        if last_doc_stack != next_doc_stack {
            return Ok(op_span!([], []));
        }

        // Surround block.
        let (attrs, span_1) = match block_walker.stepper().head() {
            Some(DocGroup(attrs, span)) => (attrs, span.skip_len()),
            _ => unreachable!(),
        };

        let mut writer = block_walker.to_writer();

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
        writer.del.exit_all();

        writer.add.begin();
        if span_1 + span_2 > 0 {
            writer.add.place(&AddSkip(span_1 + span_2));
        }
        writer.add.close(attrs);
        writer.add.exit_all();

        let res = writer.result();

        return Ok(res);
    }

    walker.back_char();

    // Check that we precede a character.
    if let Some(DocChars(..)) = walker.doc().head() {
        // fallthrough
    } else {
        // Check if parent is span, if so move outside span
        // TODO check that the parent is actually a span
        walker.stepper.next();
        if let Some(DocChars(..)) = walker.doc().head() {
            // fallthrough
        } else {
            return Ok(op_span!([], []));
        }
    }

    let mut writer = walker.to_writer();

    // Delete the character.
    writer.del.place(&DelChars(1));
    writer.del.exit_all();

    writer.add.exit_all();

    Ok(writer.result())
}

pub fn add_char(ctx: ActionContext, key: u32) -> Result<Op, Error> {
    let mut writer = Walker::to_caret(&ctx.doc, &ctx.client_id).to_writer();

    writer.del.exit_all();

    // Insert new character.
    let c: char = from_u32(key).unwrap_or('?');
    writer.add.place(&AddChars(format!("{}", c)));
    writer.add.exit_all();

    Ok(writer.result())
}

pub fn split_block(ctx: ActionContext) -> Result<Op, Error> {
    let walker = Walker::to_caret(&ctx.doc, &ctx.client_id);
    let skip = walker.doc().skip_len();

    // Identify the tag of the block we're splitting.
    let mut prev_walker = walker.clone();
    assert!(prev_walker.back_block());
    let previous_block = if let Some(DocGroup(attrs, _)) = prev_walker.doc().head() {
        attrs["tag"].to_string()
    } else {
        // Fill in default value.
        // TODO this should be a unreachable!
        "p".to_string()
    };

    // Identify if we're nested inside of a bullet.
    let mut parent_walker = prev_walker.clone();
    let nested_bullet = loop {
        //TODO re-enable once DocGroup aborts when has too few items
        if parent_walker.parent() {
            if let Some(DocGroup(ref attrs, _)) = parent_walker.doc().head() {
                if attrs["tag"] == "bullet" {
                    break true;
                }
            }
        }
        break false;
    };

    let mut writer = walker.to_writer();

    if skip > 0 {
        writer.del.place(&DelSkip(skip));
    }
    writer.del.close();
    if nested_bullet {
        writer.del.close();
    }
    writer.del.exit_all();

    writer
        .add
        .close(hashmap! { "tag".into() => previous_block });
    if nested_bullet {
        writer
            .add
            .close(hashmap! { "tag".into() => "bullet".into() });
        writer.add.begin();
    }
    writer.add.begin();
    if skip > 0 {
        writer.add.place(&AddSkip(skip));
    }
    writer.add.close(hashmap! { "tag".into() => "p".into() });
    if nested_bullet {
        writer.add.close(hashmap! { "tag".into() => "bullet".into() });
    }
    writer.add.exit_all();

    Ok(writer.result())
}

pub fn caret_move(ctx: ActionContext, increase: bool) -> Result<Op, Error> {
    let mut walker = Walker::to_caret(&ctx.doc, &ctx.client_id);

    // First operation removes the caret.
    let mut writer = walker.to_writer();

    writer.del.begin();
    writer.del.close();
    writer.del.exit_all();

    writer.add.exit_all();

    let op_1 = writer.result();

    // Second operation inserts the new caret.
    if increase {
        walker.next_char();
    } else {
        walker.back_char();
    }

    let mut writer = walker.to_writer();

    writer.del.exit_all();

    writer.add.begin();
    writer.add.close(hashmap! {
        "tag".to_string() => "caret".to_string(),
        "client".to_string() => ctx.client_id.clone(),
    });
    writer.add.exit_all();

    let op_2 = writer.result();

    // Return composed operations. Select proper order or otherwise composition
    // will be invalid.
    if increase {
        Ok(OT::compose(&op_2, &op_1))
    } else {
        Ok(OT::compose(&op_1, &op_2))
    }
}

pub fn has_caret(ctx: ActionContext) -> bool {
    Walker::to_caret_safe(&ctx.doc, &ctx.client_id).is_some()
}

pub fn init_caret(ctx: ActionContext) -> Result<Op, Error> {
    let mut walker = Walker::new(&ctx.doc);
    if !walker.goto_pos(0) {
        bail!("Could not insert first caret");
    }

    let mut writer = walker.to_writer();

    writer.del.exit_all();

    writer.add.begin();
    writer.add.close(hashmap! {
        "tag".to_string() => "caret".to_string(),
        "client".to_string() => ctx.client_id.clone(),
    });
    writer.add.exit_all();

    Ok(writer.result())
}

pub fn caret_block_move(ctx: ActionContext, increase: bool) -> Result<Op, Error> {
    let mut walker = Walker::to_caret(&ctx.doc, &ctx.client_id);

    // First operation removes the caret.
    let mut writer = walker.to_writer();

    writer.del.begin();
    writer.del.close();
    writer.del.exit_all();

    writer.add.exit_all();

    let op_1 = writer.result();

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

    writer.del.exit_all();

    writer.add.begin();
    writer.add.begin();
    writer.add.close(hashmap! {
        "tag".to_string() => "caret".to_string(),
        "client".to_string() => ctx.client_id.clone(),
    });
    writer.add.exit_all();

    let op_2 = writer.result();

    // Return composed operations. Select proper order or otherwise composition
    // will be invalid.
    if increase {
        Ok(OT::compose(&op_2, &op_1))
    } else {
        Ok(OT::compose(&op_1, &op_2))
    }
}

pub fn cur_to_caret(ctx: ActionContext, cur: &CurSpan) -> Result<Op, Error> {
    // First operation removes the caret.
    let walker = Walker::to_caret(&ctx.doc, &ctx.client_id);
    let pos_1 = walker.caret_pos();
    let mut writer = walker.to_writer();

    writer.del.begin();
    writer.del.close();
    writer.del.exit_all();

    writer.add.exit_all();

    let op_1 = writer.result();

    // Second operation inserts a new caret.

    let walker = Walker::to_cursor(&ctx.doc, cur);
    let pos_2 = walker.caret_pos();
    if pos_1 == pos_2 {
        // Redundant
        return Ok(op_span!([], []));
    }

    let mut writer = walker.to_writer();

    writer.del.exit_all();

    writer.add.begin();
    writer.add.close(hashmap! { "tag".to_string() => "caret".to_string(), "client".to_string() => ctx.client_id.clone() });
    writer.add.exit_all();

    let op_2 = writer.result();

    // Return composed operations. Select proper order or otherwise composition
    // will be invalid.
    if pos_1 < pos_2 {
        Ok(OT::compose(&op_2, &op_1))
    } else {
        Ok(OT::compose(&op_1, &op_2))
    }
}

use crate::walkers::*;
use failure::Error;
use oatie::doc::*;
use oatie::schema::RtfSchema;
use oatie::OT;
use oatie::style::OpaqueStyleMap;

fn is_boundary_char(c: char) -> bool {
    c.is_whitespace() || c == '-' || c == '_'
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

    fn apply(mut self, op: &Op) -> Result<ActionContext, Error> {
        // update self with the op, update self doc, return new self
        self.doc = Op::apply(&self.doc, op);
        self.op_result = Op::compose(&self.op_result, op);
        Ok(self)
    }

    fn get_walker<'a>(&'a self, pos: Pos) -> Result<Walker<'a>, Error> {
        Walker::to_caret(&self.doc, &self.client_id, pos)
    }

    pub fn result(self) -> Op {
        self.op_result
    }
}

pub fn add_string(mut ctx: ActionContext, input: &str) -> Result<ActionContext, Error> {
    Ok(ctx)
        .and_then(delete_selection)
        .and_then(|(_success, ctx)| {
            // Insert before start caret (given the carets are now collapsed).
            let walker = ctx.get_walker(Pos::Start)?;

            // Style map.
            let mut styles = hashmap!{ Style::Normie => None };

            // Identify previous styles.
            let mut char_walker = walker.clone();
            char_walker.back_char();
            if let Some(DocChars(_, ref prefix_styles)) = char_walker.doc().head() {
                styles.extend(
                    prefix_styles
                        .iter()
                        .map(|(a, b)| (a.to_owned(), b.to_owned())),
                );
            }

            let mut writer = walker.to_writer();

            writer.del.exit_all(); // ANCHOR next up is to remove need for exit_all

            // Insert new character.
            writer
                .add
                .place(&AddChars(DocString::from_str(input), OpaqueStyleMap::from(styles)));
            writer.add.exit_all();

            ctx.apply(&writer.result())
        })
}

pub fn toggle_list(ctx: ActionContext) -> Result<Op, Error> {
    let mut walker =
        Walker::to_caret(&ctx.doc, &ctx.client_id, Pos::Focus).expect("Expected a Focus caret");
    assert!(walker.back_block());

    let mut parent_walker = walker.clone();
    if parent_walker.parent() {
        if let Some(DocGroup(ref attrs, ref span)) = parent_walker.doc().head() {
            if attrs["tag"] == "bullet" {
                // Do the list destructuring here
                let mut writer = parent_walker.to_writer();

                writer
                    .del
                    .place(&DelGroup(del_span![DelSkip(span.skip_len())]));
                writer.del.exit_all();

                writer.add.exit_all();

                return Ok(writer.result());
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

// Return a "caret state"
pub fn identify_block(ctx: ActionContext) -> Result<(String, bool), Error> {
    let mut walker = Walker::to_caret(&ctx.doc, &ctx.client_id, Pos::Focus)?;
    assert!(walker.back_block());
    if let Some(DocGroup(ref attrs, _)) = walker.doc().head() {
        let tag = attrs["tag"].clone();
        let mut in_list = false;
        if walker.parent() {
            if let Some(DocGroup(ref attrs_2, _)) = walker.doc().head() {
                in_list = attrs_2["tag"] == "bullet";
            }
        }
        Ok((tag, in_list))
    } else {
        bail!("Expected a DocGroup from back_block");
    }
}

pub fn replace_block(ctx: ActionContext, tag: &str) -> Result<Op, Error> {
    let mut walker =
        Walker::to_caret(&ctx.doc, &ctx.client_id, Pos::Focus).expect("Expected a Focus caret");
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

fn delete_char_inner(mut walker: Walker<'_>) -> Result<Op, Error> {
    // Check if caret is at the start of a block.
    let mut block_walker = walker.clone();
    assert!(block_walker.back_block());
    // console_log!("before at start of block {:?} vs {:?}", walker.caret_pos(), block_walker.caret_pos());
    block_walker.stepper.doc.enter();
    // block_walker.stepper.next(); // re-enter the block to first caret position
    let at_start_of_block = walker.caret_pos() == block_walker.caret_pos();
    // console_log!("at start of block {:?} vs {:?}", walker.caret_pos(), block_walker.caret_pos());

    // See if we can collapse this and the previous block or list item.
    if at_start_of_block {
        // console_log!("at_start {:?} {:?}", caret_pos, block_walker.caret_pos());
        // console_log!("1");

        // Check for first block in a list item.
        let mut parent_walker = walker.clone();
        assert!(parent_walker.back_block());

        let mut is_list_item = false;
        let mut list_item_skip_len = 1;
        if parent_walker.doc().unhead() == None && parent_walker.parent() {
            if let Some(DocGroup(ref attrs_2, ref span_2)) = parent_walker.doc().head() {
                if attrs_2["tag"] == "bullet" {
                    // We are at the start of a block inside of a list item.
                    is_list_item = true;
                    list_item_skip_len = span_2.skip_len();
                }
            }
        }

        // console_log!("2");

        // Check if previous sibling is a list item too.
        if let Some(DocGroup(ref attrs_1, ref span_1)) = parent_walker.doc().unhead() {
            if attrs_1["tag"] == "bullet" {
                // The previous sibling is a list item.

                // TODO IDK wer'e working around ownership issues here
                let attrs_1 = attrs_1.to_owned();
                let span_1 = span_1.to_owned();

                parent_walker.stepper.doc.prev();
                let mut writer = parent_walker.to_writer();

                writer.del.begin();
                if span_1.skip_len() > 0 {
                    writer.del.place(&DelSkip(span_1.skip_len()));
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
                writer.del.exit_all();

                writer.add.begin();
                if span_1.skip_len() + list_item_skip_len > 0 {
                    writer
                        .add
                        .place(&AddSkip(span_1.skip_len() + list_item_skip_len));
                }
                writer.add.close(attrs_1.clone());
                writer.add.exit_all();

                let res = writer.result();

                return Ok(res);
            }
        } else {
            // console_log!("3");
            // We think we're at the start of the document, so do nothing.
            return Ok(Op::empty());
        }

        // console_log!("4");

        if is_list_item {
            // We are a list item, but we want to unindent ourselves.
            let mut writer = parent_walker.to_writer();

            writer.del.begin();
            if list_item_skip_len > 0 {
                writer.del.place(&DelSkip(list_item_skip_len));
            }
            writer.del.close();
            writer.del.exit_all();

            let res = writer.result();

            return Ok(res);
        }

        // console_log!("5");

        // Return to block parent.
        assert!(block_walker.back_block());
        let span_2 = match block_walker.stepper().head() {
            Some(DocGroup(.., span)) => span.skip_len(),
            _ => unreachable!(),
        };

        // console_log!("6");

        // Move to prior block to join it, or abort.
        if !block_walker.back_block_or_block_object() {
            return Ok(op_span!([], []));
        }

        // If block is an "hr", delete it.
        if let Some(DocGroup(ref attrs, _)) = block_walker.doc().head() {
            if attrs["tag"] == "hr" {
                let mut writer = block_walker.to_writer();

                writer.del.begin();
                writer.del.close();
                writer.del.exit_all();

                writer.add.exit_all();

                return Ok(writer.result());
            }
        } else {
            unreachable!();
        }

        // console_log!("8");

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
        writer.add.close(attrs.to_owned());
        writer.add.exit_all();

        let res = writer.result();

        // console_log!("9");

        return Ok(res);
    }

    walker.back_char();

    // console_log!("back char {:?}", walker);

    // Skip past adjacent carets in between cursor and the next char.
    // TODO is there a more elegant way to do this:
    while let Some(DocGroup(ref attrs, _)) = walker.doc().head() {
        if attrs["tag"] == "caret" {
            walker.stepper.doc.next();
        }
    }

    // Check that we precede a character.
    if let Some(DocChars(..)) = walker.doc().head() {
        // fallthrough
    } else {
        // Check if parent is span, if so move outside span
        // TODO check that the parent is actually a span
        // TODO this might not be possible anymore without spans.
        walker.stepper.next();
        if let Some(DocChars(..)) = walker.doc().head() {
            // fallthrough
        } else {
            return Ok(op_span!([], []));
        }
    }

    // console_log!("precede char");

    let mut writer = walker.to_writer();

    // Delete the character.
    writer.del.place(&DelChars(1));
    writer.del.exit_all();

    writer.add.exit_all();

    // console_log!("delITE {:?}", walker);

    Ok(writer.result())
}

/// Backspace.
pub fn delete_char(ctx: ActionContext) -> Result<Op, Error> {
    // Bail early if we delete a selection.
    let (success, ctx) = delete_selection(ctx)?;
    if success {
        return Ok(ctx.result());
    }

    // Fallback; delete backward from start caret (given the carets are collapsed).
    let walker = Walker::to_caret(&ctx.doc, &ctx.client_id, Pos::Start)?;
    delete_char_inner(walker)
}

fn delete_selection(ctx: ActionContext) -> Result<(bool, ActionContext), Error> {
    Ok(ctx)
        .and_then(|mut ctx| {
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

// For function reuse
pub enum StyleOp {
    AddStyle(Style, Option<String>),
    RemoveStyle(Style),
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

pub fn restyle(ctx: ActionContext, ops: Vec<StyleOp>) -> Result<Op, Error> {
    let walker1 = Walker::to_caret(&ctx.doc, &ctx.client_id, Pos::Anchor);
    let walker2 = Walker::to_caret(&ctx.doc, &ctx.client_id, Pos::Focus);

    let (walker1, walker2) = if let (Ok(walker1), Ok(walker2)) = (walker1, walker2) {
        if walker1.caret_pos() == walker2.caret_pos() {
            return Ok(Op::empty());
        } else if walker1.caret_pos() <= walker2.caret_pos() {
            (walker1, walker2)
        } else {
            (walker2, walker1)
        }
    } else {
        return Ok(Op::empty());
    };

    // Style map.
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

    let mut writer = walker1.to_writer();

    // Place all styles.
    if !remove_styles.is_empty() {
        let mut doc1 = walker1.doc().to_owned();
        let doc2 = walker2.doc().to_owned();
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
    writer.del.exit_all();

    // Place all styles.
    if !add_styles.is_empty() {
        let mut doc1 = walker1.doc().to_owned();
        let doc2 = walker2.doc().to_owned();
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
    writer.add.exit_all();

    let r = writer.result();
    println!("(r) {:?}", r);

    Ok(r)
}

pub fn split_block(ctx: ActionContext, add_hr: bool) -> Result<Op, Error> {
    let walker =
        Walker::to_caret(&ctx.doc, &ctx.client_id, Pos::Focus).expect("Expected a Focus caret");
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
    if add_hr {
        writer.add.begin();
        writer.add.close(hashmap! { "tag".into() => "hr".into() });
    }
    writer.add.begin();
    if skip > 0 {
        writer.add.place(&AddSkip(skip));
    }
    writer.add.close(hashmap! { "tag".into() => "p".into() });
    if nested_bullet {
        writer
            .add
            .close(hashmap! { "tag".into() => "bullet".into() });
    }
    writer.add.exit_all();

    Ok(writer.result())
}

pub fn has_bounding_carets(ctx: ActionContext) -> bool {
    // At the moment, having an Anchor caret indicates that both carets exist
    has_caret(ctx, Pos::Anchor)
}

pub fn caret_move(
    mut ctx: ActionContext,
    increase: bool,
    preserve_select: bool,
) -> Result<Op, Error> {
    let op_1 = if !preserve_select && has_bounding_carets(ctx.clone()) {
        let (_pos, op) = caret_clear(ctx.clone(), Pos::Anchor)?;
        ctx.doc = Op::apply(&ctx.doc.clone(), &op);
        op
    } else {
        Op::empty()
    };

    let mut walker = Walker::to_caret(&ctx.doc, &ctx.client_id, Pos::Focus)?;

    // First operation removes the caret.
    let mut writer = walker.to_writer();

    writer.del.begin();
    writer.del.close();
    writer.del.exit_all();

    writer.add.exit_all();

    let op_2 = writer.result();

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
        "focus".to_string() => "true".to_string(),
    });
    writer.add.begin();
    writer.add.close(hashmap! {
        "tag".to_string() => "caret".to_string(),
        "client".to_string() => ctx.client_id.clone(),
        "focus".to_string() => "false".to_string(),
    });
    writer.add.exit_all();

    let op_3 = writer.result();

    // Return composed operations. Select proper order or otherwise composition
    // will be invalid.

    Ok(Op::compose(
        &op_1,
        &Op::transform_advance::<RtfSchema>(&op_2, &op_3),
    ))
}

pub fn caret_word_move(ctx: ActionContext, increase: bool) -> Result<Op, Error> {
    let mut walker =
        Walker::to_caret(&ctx.doc, &ctx.client_id, Pos::Focus).expect("Expected a Focus caret");

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
        println!("skipping WORD");
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

    let mut writer = walker.to_writer();

    writer.del.exit_all();

    writer.add.begin();
    writer.add.close(hashmap! {
        "tag".to_string() => "caret".to_string(),
        "client".to_string() => ctx.client_id.clone(),
        "focus".to_string() => "true".to_string(),
    });
    writer.add.begin();
    writer.add.close(hashmap! {
        "tag".to_string() => "caret".to_string(),
        "client".to_string() => ctx.client_id.clone(),
        "focus".to_string() => "false".to_string(),
    });
    writer.add.exit_all();

    let op_2 = writer.result();

    // Return composed operations. Select proper order or otherwise composition
    // will be invalid.
    Ok(Op::transform_advance::<RtfSchema>(&op_1, &op_2))
}

pub fn caret_select_all(ctx: ActionContext) -> Result<Op, Error> {
    let mut start = Walker::new(&ctx.doc);
    start.goto_pos(0);
    let mut end = Walker::new(&ctx.doc);
    end.goto_end();

    // Delete both carets.
    let op_1_2 = Op::transform_advance::<RtfSchema>(&{
        // First operation removes the caret.
        caret_clear(ctx.clone(), Pos::Focus)
            .map(|(_pos_1, op_1)| op_1)
            .unwrap_or_else(|_| Op::empty())
    }, &{
        // Second operation removes the focus caret if needed.
        caret_clear(ctx.clone(), Pos::Anchor)
            .map(|(_pos_1, op_1)| op_1)
            .unwrap_or_else(|_| Op::empty())
    });

    // Second operation inserts a new caret.

    let mut writer = start.to_writer();

    writer.del.exit_all();

    writer.add.begin();
    writer.add.close(hashmap! {
        "tag".to_string() => "caret".to_string(),
        "client".to_string() => ctx.client_id.clone(),
        "focus".to_string() => "false".to_string(),
    });
    writer.add.exit_all();

    let op_3 = writer.result();

    let mut writer = end.to_writer();

    writer.del.exit_all();

    writer.add.begin();
    writer.add.close(hashmap! {
        "tag".to_string() => "caret".to_string(),
        "client".to_string() => ctx.client_id.clone(),
        "focus".to_string() => "true".to_string(),
    });
    writer.add.exit_all();

    let op_4 = writer.result();

    // println!("------------->\n{:?}\n\n\nAAAAAA\n-------->", op_2);

    let op_1_2_3 = Op::transform_advance::<RtfSchema>(&op_1_2, &op_3);
    let op_1_2_3_4 = Op::transform_advance::<RtfSchema>(&op_1_2_3, &op_4);

    Ok(op_1_2_3_4)
}

pub fn has_caret(ctx: ActionContext, pos: Pos) -> bool {
    Walker::to_caret(&ctx.doc, &ctx.client_id, pos).is_ok()
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
        "focus".to_string() => "true".to_string(),
    });
    writer.add.begin();
    writer.add.close(hashmap! {
        "tag".to_string() => "caret".to_string(),
        "client".to_string() => ctx.client_id.clone(),
        "focus".to_string() => "false".to_string(),
    });
    writer.add.exit_all();

    Ok(writer.result())
}

pub fn caret_block_move(ctx: ActionContext, increase: bool) -> Result<Op, Error> {
    let mut walker =
        Walker::to_caret(&ctx.doc, &ctx.client_id, Pos::Focus).expect("Expected a Focus caret");

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
        "focus".to_string() => "true".to_string(),
    });
    writer.add.exit_all();

    let op_2 = writer.result();

    // Return composed operations. Select proper order or otherwise composition
    // will be invalid.
    Ok(Op::transform_advance::<RtfSchema>(&op_1, &op_2))
}

// Returns new caret position
pub fn caret_clear(ctx: ActionContext, position: Pos) -> Result<(isize, Op), Error> {
    let walker = Walker::to_caret(&ctx.doc, &ctx.client_id, position)?;
    caret_clear_inner(walker)
}

pub fn caret_clear_inner(walker: Walker<'_>) -> Result<(isize, Op), Error> {
    let pos = walker.caret_pos();
    let mut writer = walker.to_writer();

    writer.del.begin();
    writer.del.close();
    writer.del.exit_all();

    writer.add.exit_all();

    let op = writer.result();

    Ok((pos, op))
}

pub fn cur_to_caret(ctx: ActionContext, cur: &CurSpan, focus: bool) -> Result<Op, Error> {
    // First operation removes the caret.
    let (_pos_1, op_1) = caret_clear(ctx.clone(), if focus { Pos::Focus } else { Pos::Anchor })
        .map(|(pos_1, op_1)| (Some(pos_1), op_1))
        .unwrap_or_else(|_| (None, Op::empty()));

    // Second operation removes the focus caret if needed.
    // let (_, op_2) = (if !focus {
    //     caret_clear(ctx.clone(), Pos::Anchor)
    //         .map(|(pos_1, op_1)| (Some(pos_1), op_1))
    //         .ok()
    // } else {
    //     None
    // }).unwrap_or_else(|| (None, Op::empty()));
    // TODO might just remove this op combo
    let op_2 = Op::empty();

    // Combine two starting ops.
    let op_1_2 = Op::transform_advance::<RtfSchema>(&op_1, &op_2);

    // Second operation inserts a new caret.

    // console_log!("----@@ {:?}", op_1_2);
    // console_log!("-----> {:?}", cur);
    let walker = Walker::to_cursor(&ctx.doc, cur);
    let _pos_3 = Some(walker.caret_pos());
    // console_log!("---@@@@@@@@@@ {:?}", pos_3);
    // if pos_1 == pos_3 {
    //     // Redundant
    //     return Ok(op_span!([], []));
    // }
    let mut writer = walker.to_writer();
    // console_log!("-[[[\n{:?}\n\n]]]", writer.add);

    writer.del.exit_all();

    writer.add.begin();
    writer.add.close(hashmap! {
        "tag".to_string() => "caret".to_string(),
        "client".to_string() => ctx.client_id.clone(),
        "focus".to_string() => if focus { format!("true") } else { format!("false") },
    });
    writer.add.exit_all();

    let op_3 = writer.result();

    // console_log!("-----op_3: {:?}", op_3);

    // println!("------------->\n{:?}\n\n\nAAAAAA\n-------->", op_2);

    let res = Op::transform_advance::<RtfSchema>(&op_1_2, &op_3);
    // console_log!("------< {:?}", res);
    Ok(res)
}

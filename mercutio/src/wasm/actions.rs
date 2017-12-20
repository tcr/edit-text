use oatie::doc::*;
use oatie::Operation;
use failure::Error;
use std::char::from_u32;
use super::walkers::*;

pub fn replace_block(doc: &Doc, tag: &str) -> Result<Op, Error> {
    let mut walker = Walker::to_caret(&*doc);
    walker.back_block();

    let len = if let Some(DocGroup(_, ref span)) = walker.doc.head() {
        span.skip_len()
    } else {
        println!("uhg {:?}", walker);
        unreachable!()
    };

    let mut writer = walker.to_writer();

    writer.del.group(&del_span![DelSkip(len)]);

    writer.add.group(&hashmap! { "tag".to_string() => tag.to_string() }, &add_span![AddSkip(len)]);

    Ok(writer.result())
}

pub fn delete_char(doc: &Doc) -> Result<Op, Error> {
    let mut walker = Walker::to_caret(&*doc);

    // Check if we lead the block.
    let caret_pos = walker.caret_pos;
    let mut block_walker = walker.clone();
    block_walker.back_block();
    if caret_pos == block_walker.caret_pos {
        println!("TODO: merge blocks!");
        return Ok(op_span!([], []));
    }

    walker.back_char();

    // check if we are in a character group.
    if let Some(DocChars(..)) = walker.doc.head() {
        // fallthrough
    } else {
        return Ok(op_span!([], []));
    }

    let mut writer = walker.to_writer();

    writer.del.chars(1);
    writer.del.exit_all();

    writer.add.exit_all();

    Ok(writer.result())
}

pub fn add_char(doc: &Doc, key: u32) -> Result<Op, Error> {
    let mut writer = Walker::to_caret(&*doc).to_writer();

    writer.del.exit_all();

    // Insert new character.
    let c: char = from_u32(key).unwrap_or('?');
    writer.add.chars(&format!("{}", c));
    writer.add.exit_all();

    Ok(writer.result())
}

pub fn split_block(doc: &Doc) -> Result<Op, Error> {
    let walker = Walker::to_caret(&*doc);
    let skip = walker.doc.skip_len();

    let previous_block = if let Some(DocGroup(attrs, _)) = walker.clone().back_block().doc.head() {
        attrs["tag"].to_string()
    } else {
        // Fill in default value.
        // TODO this should be a panic!
        "p".to_string()
    };

    let mut writer = walker.to_writer();

    writer.del.skip(skip);
    writer.del.close();
    writer.del.exit_all();

    writer.add.close(hashmap! { "tag".to_string() => previous_block });
    writer.add.begin();
    writer.add.skip(skip);
    writer.add.close(hashmap! { "tag".to_string() => "p".to_string() });
    writer.add.exit_all();

    Ok(writer.result())
}

pub fn caret_move(doc: &Doc, increase: bool) -> Result<Op, Error> {
    let mut walker = Walker::to_caret(&*doc);

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
    writer.add.close(hashmap! { "tag".to_string() => "caret".to_string() });
    writer.add.exit_all();
    
    let op_2 = writer.result();

    // Return composed operations. Select proper order or otherwise composition
    // will be invalid.
    if increase {
        Ok(Operation::compose(&op_2, &op_1))
    } else {
        Ok(Operation::compose(&op_1, &op_2))
    }
}

pub fn cur_to_caret(doc: &Doc, cur: &CurSpan) -> Result<Op, Error> {
    // First operation removes the caret.
    let mut writer = Walker::to_caret(&*doc).to_writer();

    writer.del.begin();
    writer.del.close();
    writer.del.exit_all();

    writer.add.exit_all();

    let (doc, op_1) = writer.apply_result(doc);

    // Second operation inserts a new caret.
    let mut walker = Walker::to_cursor(&doc, cur);
    walker.snap_char();

    let mut writer = walker.to_writer();

    writer.del.exit_all();

    writer.add.begin();
    writer.add.close(hashmap! { "tag".to_string() => "caret".to_string() });
    writer.add.exit_all();
    
    let op_2 = writer.result();

    // Return composed operations.
    Ok(Operation::compose(&op_1, &op_2))
}
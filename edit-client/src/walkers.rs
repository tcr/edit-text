mod caretstepper;

use failure::Error;
use oatie::doc::*;
use oatie::stepper::*;
use oatie::writer::*;
use take_mut;
pub use self::caretstepper::*;

#[derive(Debug, Clone)]
pub struct Walker<'a> {
    original_doc: Doc,
    pub stepper: CaretStepper<'a>,
}

impl<'a> Walker<'a> {
    pub fn new(doc: &'a Doc) -> Walker<'a> {
        Walker {
            original_doc: doc.clone(),
            stepper: CaretStepper::new(DocStepper::new(&doc.0)),
        }
    }

    pub fn doc(&self) -> &'a DocStepper<'_> {
        &self.stepper.doc
    }

    pub fn caret_pos(&self) -> isize {
        self.stepper.caret_pos
    }

    pub fn goto_pos(&mut self, target_pos: isize) -> bool {
        let mut matched = false;
        take_mut::take(&mut self.stepper, |prev_stepper| {
            let mut stepper = prev_stepper.clone();

            // Iterate until we match the cursor.
            matched = loop {
                if stepper.caret_pos == target_pos && stepper.is_valid_caret_pos() {
                    break true;
                }
                if stepper.next().is_none() {
                    break false;
                }
            };

            if matched {
                stepper
            } else {
                prev_stepper
            }
        });

        matched
    }

    pub fn goto_end(&mut self) {
        take_mut::take(&mut self.stepper, |prev_stepper| {
            let mut stepper = prev_stepper.clone();
            let mut last_stepper = stepper.clone();

            // Iterate until we match the cursor.
            loop {
                if stepper.is_valid_caret_pos() {
                    last_stepper = stepper.clone();
                }
                if stepper.next().is_none() {
                    break;
                }
            }

            last_stepper
        });
    }

    pub fn to_caret(doc: &'a Doc, client_id: &str, position: Pos) -> Result<Walker<'a>, Error> {
        let mut stepper = CaretStepper::new(DocStepper::new(&doc.0));

        // Iterate until we match the cursor.
        let mut result_stepper = None;
        loop {
            if let Some(DocGroup(attrs, _)) = stepper.doc.head() {
                match position {
                    Pos::Focus => {
                        if is_caret(&attrs, Some(client_id), true) {
                            result_stepper = Some(stepper);
                            break;
                        }
                    }
                    Pos::Anchor => {
                        if is_caret(&attrs, Some(client_id), false) {
                            result_stepper = Some(stepper);
                            break;
                        }
                    }
                    Pos::Start => {
                        if is_caret(&attrs, Some(client_id), false)
                            || is_caret(&attrs, Some(client_id), true)
                        {
                            result_stepper = Some(stepper);
                            break;
                        }
                    }
                    // Continue until last match
                    Pos::End => {
                        if is_caret(&attrs, Some(client_id), false)
                            || is_caret(&attrs, Some(client_id), true)
                        {
                            result_stepper = Some(stepper.clone());
                        }
                    }
                }
            }
            if stepper.skip_element().is_none() {
                break;
            }
        }

        if let Some(mut result_stepper) = result_stepper {
            result_stepper.caret_pos = 0;
            Ok(Walker {
                original_doc: doc.clone(),
                stepper: result_stepper,
            })
        } else {
            Err(format_err!(
                "Could not find cursor at {:?} position.",
                position
            ))
        }
    }

    pub fn to_cursor(doc: &'a Doc, cur: &CurSpan) -> Walker<'a> {
        let mut stepper = CaretStepper::new(DocStepper::new(&doc.0));

        let mut match_cur = CurStepper::new(cur);
        let mut match_doc = DocStepper::new(&doc.0);

        let mut matched = false;
        loop {
            match match_cur.head() {
                // End of cursor iterator
                Some(CurGroup) | Some(CurChar) => {
                    matched = true;
                    break;
                }

                Some(CurSkip(n)) => {
                    match_cur.next();
                    match_doc.skip(n);
                }
                Some(CurWithGroup(..)) => {
                    match_cur.enter();
                    match_doc.enter();
                }
                None => {
                    if match_cur.is_done() {
                        break;
                    } else {
                        match_cur.exit();
                        match_doc.exit();
                    }
                }
            }

            while match_doc != stepper.doc {
                if stepper.next().is_none() {
                    break;
                }
            }
        }
        if !matched {
            panic!("Didn't find the cursor.");
        }

        // console_log!("(^^^) (A) {:?}", stepper.doc);

        // Snap to leftmost character boundary. It's possible the
        // cursor points to a character following a span or caret, but
        // we want our stepper to be on the immediate right of its character.
        let mut rstepper = stepper.rev();
        loop {
            // console_log!("(^^^) (uu) {:?}", rstepper.doc);
            if rstepper.next().is_none() {
                // console_log!("none?");
                break;
            }
            if rstepper.is_valid_caret_pos() {
                // console_log!("valid");
                break;
            }
        }

        // console_log!("(^^^) (C) {:?}", rstepper.doc);

        // Next, increment by one full char (so cursor is always on right).
        let mut stepper = rstepper.clone().rev();
        loop {
            if stepper.next().is_none() {
                break;
            }
            if stepper.is_valid_caret_pos() {
                break;
            }
        }
        // console_log!("(^^^) (D) {:?}", stepper.doc);
        // ...or else restore the stepper again.
        if !stepper.is_valid_caret_pos() {
            stepper = rstepper.rev();
        }

        // console_log!("(^^^) (E) {:?}", stepper.doc);

        Walker {
            original_doc: doc.clone(),
            stepper,
        }
    }

    pub fn parent(&mut self) -> bool {
        let mut matched = false;
        take_mut::take(&mut self.stepper, |stepper| {
            let mut rstepper = stepper.rev();

            // Iterate until we reach a block.
            let mut depth = 1;
            while depth > 0 {
                if rstepper.next().is_none() {
                    break;
                }
                if let Some(DocGroup(_, _)) = rstepper.doc.head() {
                    depth -= 1;
                } else if let None = rstepper.doc.head() {
                    depth += 1;
                }
            }
            matched = depth == 0;

            rstepper.rev()
        });

        matched
    }

    // TODO this might be worth a better name
    pub fn back_block_or_block_object(&mut self) -> bool {
        let mut matched = false;
        take_mut::take(&mut self.stepper, |prev_stepper| {
            let mut rstepper = prev_stepper.clone().rev();

            // Iterate until we reach a block.
            matched = loop {
                if rstepper.next().is_none() {
                    break false;
                }
                if let Some(DocGroup(attrs, _)) = rstepper.doc.head() {
                    if is_block(&attrs) || is_block_object(&attrs) {
                        break true;
                    }
                }
            };

            if matched {
                rstepper.rev()
            } else {
                prev_stepper
            }
        });

        matched
    }

    pub fn at_start_of_block(&mut self) -> bool {
        let mut matched = false;
        take_mut::take(&mut self.stepper, |prev_stepper| {
            let mut rstepper = prev_stepper.clone().rev();

            // Iterate until we reach a block.
            matched = loop {
                if rstepper.next().is_none() {
                    break false;
                }
                match rstepper.doc.head() {
                    Some(DocGroup(attrs, _)) => {
                        if is_any_caret(&attrs) {
                            continue;
                        }
                        if is_block(&attrs) {
                            break true;
                        }
                    }
                    _ => {}
                }
                break false;
            };

            prev_stepper
        });

        matched
    }

    pub fn back_block(&mut self) -> bool {
        let mut matched = false;
        take_mut::take(&mut self.stepper, |prev_stepper| {
            let mut rstepper = prev_stepper.clone().rev();

            // Iterate until we reach a block.
            matched = loop {
                if rstepper.next().is_none() {
                    break false;
                }
                if let Some(DocGroup(attrs, _)) = rstepper.doc.head() {
                    if is_block(&attrs) {
                        break true;
                    }
                }
            };

            if matched {
                rstepper.rev()
            } else {
                prev_stepper
            }
        });

        matched
    }

    pub fn next_block(&mut self) -> bool {
        let mut matched = false;
        take_mut::take(&mut self.stepper, |prev_stepper| {
            let mut stepper = prev_stepper.clone();

            // Iterate until we match the cursor.
            matched = loop {
                if stepper.next().is_none() {
                    break false;
                }
                if let Some(DocGroup(attrs, _)) = stepper.doc.head() {
                    if is_block(&attrs) {
                        break true;
                    }
                }
            };

            if matched {
                stepper
            } else {
                prev_stepper
            }
        });

        matched
    }

    pub fn next_char(&mut self) {
        take_mut::take(&mut self.stepper, |prev_stepper| {
            let mut stepper = prev_stepper.clone();
            let target_pos = stepper.caret_pos + 1;

            // Iterate until we match the cursor.
            let matched = loop {
                if stepper.caret_pos == target_pos && stepper.is_valid_caret_pos() {
                    break true;
                }
                if stepper.next().is_none() {
                    break false;
                }
            };

            if matched {
                stepper
            } else {
                prev_stepper
            }
        });
    }

    pub fn back_char(&mut self) {
        let _ = take_mut::take(&mut self.stepper, |prev_stepper| {
            let mut rstepper = prev_stepper.clone().rev();

            let target_pos = rstepper.caret_pos - 1;

            // Iterate until we match the cursor.
            let matched = loop {
                if rstepper.caret_pos == target_pos && rstepper.is_valid_caret_pos() {
                    break true;
                }
                if rstepper.next().is_none() {
                    break false;
                }
                // println!("----> step {:#?}", rstepper.doc);
            };

            if matched {
                rstepper.rev()
            } else {
                prev_stepper
            }
        });
    }

    pub fn delta(&self, previous: &Self) -> Option<isize> {
        let mut stepper = previous.stepper.clone();

        // Iterate until we match the cursor.
        loop {
            if stepper.is_valid_caret_pos() {
                if stepper.doc == self.stepper.doc {
                    return Some(stepper.caret_pos);
                }
            }
            if stepper.next().is_none() {
                return None;
            }
        };
    }

    pub fn to_writer(&self) -> OpWriter {
        let mut del = DelWriter::new();
        let mut add = AddWriter::new();

        // Walk the doc until we reach our current doc position.
        let mut current_stepper = self.stepper.doc.clone();

        let char_index = current_stepper.char_index();
        current_stepper.char_cursor_update(); // why? because we might have a reversed stepper?

        let mut doc_stepper = DocStepper::new(&self.original_doc.0);
        // TODO added raw_index since raw partialeq operation breaks. see DocStepper:raw_index
        while unsafe { current_stepper.raw_index() != doc_stepper.raw_index() } {
            // console_log!("head ----> {:?}", doc_stepper.head());
            // console_log!("head stack len ---> {:?}", doc_stepper.stack().len());
            // console_log!("head stack ---> {:?}", doc_stepper.stack());
            match doc_stepper.head() {
                Some(DocChars(ref text, _)) => {
                    let text_len = text.char_len();
                    del.place(&DelSkip(text_len));
                    add.place(&AddSkip(text_len));
                    doc_stepper.next();
                }
                Some(DocGroup(..)) => {
                    del.begin();
                    add.begin();
                    doc_stepper.enter();
                }
                None => {
                    if doc_stepper.is_done() {
                        // TODO is it possible end of document could actually be target?
                        panic!("Reached end of document via to_writer");
                    } else {
                        del.exit();
                        add.exit();
                        doc_stepper.exit();
                    }
                }
            }
        }

        if let Some(index) = char_index {
            if index > 0 {
                del.place(&DelSkip(index));
                add.place(&AddSkip(index));
            }
        }

        OpWriter { del, add }
    }

    pub fn stepper(&self) -> &'a DocStepper<'_> {
        &self.stepper.doc
    }
}

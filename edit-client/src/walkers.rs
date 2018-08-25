use oatie::doc::*;
use oatie::stepper::*;
use oatie::transform::Schema;
use oatie::writer::*;
use take_mut;
use failure::Error;

fn is_block(attrs: &Attrs) -> bool {
    use oatie::schema::*;
    RtfSchema::track_type_from_attrs(attrs) == Some(RtfTrack::Blocks)
}

// TODO what does this refer to?
fn is_block_object(attrs: &Attrs) -> bool {
    use oatie::schema::*;
    RtfSchema::track_type_from_attrs(attrs) == Some(RtfTrack::BlockObjects)
}

fn is_caret(attrs: &Attrs, client_id: Option<&str>, focus: bool) -> bool {
    attrs["tag"] == "caret" && client_id.map(|id| attrs.get("client") == Some(&id.to_string())).unwrap_or(false)
        && attrs
            .get("focus")
            .map(|x| x == "true")
            .unwrap_or(false) == focus
}

// Is any caret
pub fn is_any_caret(attrs: &Attrs) -> bool {
    attrs["tag"] == "caret"
}

#[derive(Clone, Debug)]
pub struct CaretStepper {
    pub doc: DocStepper,
    caret_pos: isize,
}

#[derive(Clone, Debug)]
pub enum Pos {
    Start,
    Anchor,
    Focus,
    End,
}

impl CaretStepper {
    pub fn new(doc: DocStepper) -> CaretStepper {
        CaretStepper { doc, caret_pos: -1 }
    }

    pub fn rev(self) -> ReverseCaretStepper {
        ReverseCaretStepper {
            doc: self.doc,
            caret_pos: self.caret_pos,
        }
    }

    pub fn is_valid_caret_pos(&self) -> bool {
        if let Some(DocChars(..)) = self.doc.unhead() {
            return true;
        } else if self.doc.unhead().is_none() && !self.doc.is_back_done() {
            if let Some(DocGroup(ref attrs, _)) = self.doc.clone().unenter().head() {
                if is_block(attrs) {
                    return true;
                }
            }
        }
        return false;
    }

    // TODO this is an easier alternative to .next() for skipping strings of chars,
    // but is it the best name or interface
    fn skip_element(&mut self) -> Option<()> {
        let len = match self.doc.head() {
            Some(DocChars(val)) => {
                let len = val.char_len();
                self.doc.skip(len);
                len
            }
            Some(DocGroup(..)) => {
                self.doc.enter();
                1
            }
            None => if self.doc.is_done() {
                return None;
            } else {
                self.doc.exit();
                1
            },
        };

        if self.is_valid_caret_pos() {
            self.caret_pos += len as isize;
        }

        Some(())
    }
}

impl Iterator for CaretStepper {
    type Item = ();

    fn next(&mut self) -> Option<()> {
        match self.doc.head() {
            Some(DocChars(..)) => {
                self.doc.skip(1);
            }
            Some(DocGroup(..)) => {
                self.doc.enter();
            }
            None => if self.doc.is_done() {
                return None;
            } else {
                self.doc.exit();
            },
        }

        if self.is_valid_caret_pos() {
            self.caret_pos += 1;
        }

        Some(())
    }
}

#[derive(Clone, Debug)]
pub struct ReverseCaretStepper {
    doc: DocStepper,
    caret_pos: isize,
}

impl ReverseCaretStepper {
    pub fn rev(self) -> CaretStepper {
        CaretStepper {
            doc: self.doc,
            caret_pos: self.caret_pos,
        }
    }

    pub fn is_valid_caret_pos(&self) -> bool {
        // Skip over all preceding carets so we can identify the previous node
        // more easily.
        // TODO can this clone be avoided?
        let mut doc2 = self.doc.clone();
        while let Some(DocGroup(ref attrs, _)) = doc2.unhead() {
            if is_any_caret(attrs) {
                doc2.unskip(1);
            } else {
                break;
            }
        }

        if let Some(DocChars(..)) = doc2.unhead() {
            return true;
        } else if doc2.unhead().is_none() {
            if doc2.stack.is_empty() {
                return false;
            }
            if let Some(DocGroup(ref attrs, _)) = doc2.clone().unenter().head() {
                if is_block(attrs) {
                    return true;
                }
            }
        }
        return false;
    }
}

impl Iterator for ReverseCaretStepper {
    type Item = ();

    fn next(&mut self) -> Option<()> {
        // Skip over all preceding carets so we don't confuse the previous
        // char with a new position.
        while let Some(DocGroup(ref attrs, _)) = self.doc.unhead() {
            if is_any_caret(attrs) {
                self.doc.unskip(1);
            } else {
                break;
            }
        }

        // console_log!("what {:?}", self.doc);

        match self.doc.unhead() {
            Some(DocChars(..)) => {
                self.doc.unskip(1);
            }
            Some(DocGroup(..)) => {
                self.doc.unexit();
            }
            None => {
                if self.doc.stack.is_empty() {
                    return None;
                } else {
                    self.doc.unenter();
                }
            }
        }

        if self.is_valid_caret_pos() {
            self.caret_pos -= 1;
        }

        Some(())
    }
}

#[derive(Debug, Clone)]
pub struct Walker {
    original_doc: Doc,
    pub stepper: CaretStepper,
}

impl Walker {
    pub fn new(doc: &Doc) -> Walker {
        Walker {
            original_doc: doc.clone(),
            stepper: CaretStepper::new(DocStepper::new(&doc.0)),
        }
    }

    pub fn doc(&self) -> &DocStepper {
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
            };

            last_stepper
        });
    }

    pub fn to_caret(doc: &Doc, client_id: &str, focus: bool) -> Walker {
        let mut stepper = CaretStepper::new(DocStepper::new(&doc.0));

        // Iterate until we match the cursor.
        let matched = loop {
            if let Some(DocGroup(attrs, _)) = stepper.doc.head() {
                if is_caret(&attrs, Some(client_id), focus) {
                    break true;
                }
            }
            if stepper.skip_element().is_none() {
                break false;
            }
        };
        if !matched {
            panic!("Didn't find a (focus={:?}) caret.", focus);
        }

        Walker {
            original_doc: doc.clone(),
            stepper,
        }
    }

    // TODO Have this be replaced with to_caret_position also
    // Only difference is that above consumers
    // haven't had an .unwrap() call added yet for this:
    pub fn to_caret_safe(doc: &Doc, client_id: &str, focus: bool) -> Option<Walker> {
        let mut stepper = CaretStepper::new(DocStepper::new(&doc.0));

        // Iterate until we match the cursor.
        let matched = loop {
            if let Some(DocGroup(attrs, _)) = stepper.doc.head() {
                if is_caret(&attrs, Some(client_id), focus) {
                    break true;
                }
            }
            if stepper.next().is_none() {
                break false;
            }
        };

        if !matched {
            None
        } else {
            Some(Walker {
                original_doc: doc.clone(),
                stepper,
            })
        }
    }

    // TODO Have this replace the above and take its name.
    // Only difference is that above consumers
    // haven't had an .unwrap() call added yet for this:
    pub fn to_caret_position(doc: &Doc, client_id: &str, position: Pos) -> Result<Walker, Error> {
        let mut stepper = CaretStepper::new(DocStepper::new(&doc.0));

        // Iterate until we match the cursor.
        let mut result_stepper = None;
        loop {
            if let Some(DocGroup(attrs, _)) = stepper.doc.head() {
                match position {
                    Pos::Focus => if is_caret(&attrs, Some(client_id), true) {
                        result_stepper = Some(stepper);
                        break;
                    },
                    Pos::Anchor => if is_caret(&attrs, Some(client_id), false) {
                        result_stepper = Some(stepper);
                        break;
                    },
                    Pos::Start => if is_caret(&attrs, Some(client_id), false) || is_caret(&attrs, Some(client_id), true) {
                        result_stepper = Some(stepper);
                        break;
                    },
                    // Continue until last match
                    Pos::End => if is_caret(&attrs, Some(client_id), false) || is_caret(&attrs, Some(client_id), true) {
                        result_stepper = Some(stepper.clone());
                    },
                }
                
            }
            if stepper.next().is_none() {
                break;
            }
        }

        if let Some(result_stepper) = result_stepper {
            Ok(Walker {
                original_doc: doc.clone(),
                stepper: result_stepper,
            })
        } else {
            Err(format_err!("Could not find cursor at {:?} position.", position))
        }
    }

    pub fn to_cursor(doc: &Doc, cur: &CurSpan) -> Walker {
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
                None => if match_cur.is_done() {
                    break;
                } else {
                    match_cur.exit();
                    match_doc.exit();
                },
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

    pub fn next_char(&mut self) -> &mut Walker {
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

        self
    }

    pub fn back_char(&mut self) -> &mut Walker {
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

        self
    }

    pub fn to_writer(&self) -> OpWriter {
        let mut del = DelWriter::new();
        let mut add = AddWriter::new();

        // Walk the doc until we reach our current doc position.
        let mut doc_stepper = DocStepper::new(&self.original_doc.0);

        while self.stepper.doc != doc_stepper {
            match doc_stepper.head() {
                Some(DocChars(..)) => {
                    del.place(&DelSkip(1));
                    add.place(&AddSkip(1));
                    doc_stepper.skip(1);
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

        OpWriter { del, add }
    }

    pub fn stepper(&self) -> &DocStepper {
        &self.stepper.doc
    }
}

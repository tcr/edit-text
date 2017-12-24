use oatie::doc::*;
use oatie::stepper::*;
use oatie::writer::*;
use take_mut;

fn is_block(attrs: &Attrs) -> bool {
    use oatie::schema::*;
    Tag(attrs.clone()).tag_type() == Some(TrackType::Blocks)
}

fn is_caret(attrs: &Attrs, client_id: Option<&str>) -> bool {
    attrs["tag"] == "caret" && client_id.map(|id| attrs["client"] == id).unwrap_or(false)
}

#[derive(Clone, Debug)]
pub struct CaretStepper {
    doc: DocStepper,
    caret_pos: isize,
}

impl CaretStepper {
    pub fn new(doc: DocStepper) -> CaretStepper {
        CaretStepper {
            doc,
            caret_pos: -1,
        }
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
        } else if self.doc.unhead().is_none() {
            if let Some(DocGroup(ref attrs, _)) = self.doc.clone().unenter().head() {
                if is_block(attrs) {
                    return true;
                }
            }
        }
        return false;
    }
}

impl Iterator for CaretStepper {
    type Item = ();

    fn next(&mut self) -> Option<()> {
        match self.doc.head() {
            Some(DocChars(..)) => {
                self.doc.skip(1);
            }
            Some(DocGroup(attrs, _)) => {
                self.doc.enter();
            }
            None => if self.doc.is_done() {
                return None;
            } else {
                self.doc.exit();
            }
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
        if let Some(DocChars(..)) = self.doc.unhead() {
            return true;
        } else if self.doc.unhead().is_none() {
            if let Some(DocGroup(ref attrs, _)) = self.doc.clone().unenter().head() {
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
        if self.is_valid_caret_pos() {
            self.caret_pos -= 1;
        }

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

        Some(())
    }
}


#[derive(Debug, Clone)]
pub struct Walker {
    original_doc: Doc,
    stepper: CaretStepper,
}

impl Walker {
    pub fn doc(&self) -> &DocStepper {
        &self.stepper.doc
    }

    pub fn caret_pos(&self) -> isize {
        self.stepper.caret_pos
    }

    pub fn to_caret(doc: &Doc, client_id: &str) -> Walker {
        let original_doc = doc.clone();
        let mut doc = DocStepper::new(&doc.0);
        let mut cstep = CaretStepper::new(doc);

        // Iterate until we match the cursor.
        let matched = loop {
            if let Some(DocGroup(attrs, _)) = cstep.doc.head() {
                if is_caret(&attrs, Some(client_id)) {
                    break true;
                }
            }
            if cstep.next().is_none() {
                break false;
            }
        };
        if !matched {
            panic!("Didn't find a caret.");
        }

        Walker {
            original_doc,
            stepper: CaretStepper {
                doc: cstep.doc,
                caret_pos: cstep.caret_pos,
            }
        }
    }

    pub fn to_cursor(doc: &Doc, cur: &CurSpan) -> Walker {
        let mut stepper = CaretStepper {
            doc: DocStepper::new(&doc.0),
            caret_pos: -1,
        };

        let mut match_cur = CurStepper::new(cur);
        let mut match_doc = DocStepper::new(&doc.0);

        let mut matched = false;
        loop {
            match match_cur.get_head() {
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
                stepper.next();
            }
        }
        if !matched {
            panic!("Didn't find the cursor.");
        }

        // Snap to leftmost character boundary. It's possible the
        // cursor points to a character following a span or caret, but
        // we want our stepper to be on the immediate right of its character.
        let mut rstepper = stepper.rev();
        while !rstepper.is_valid_caret_pos() {
            rstepper.next();
        }

        // Next, increment by one full char (so cursor is always on right).
        let mut stepper = rstepper.rev();
        loop {
            stepper.next();
            if stepper.is_valid_caret_pos() {
                break;
            }
        }

        Walker {
            original_doc: doc.clone(),
            stepper,
        }
    }

    // TODO update the caret_pos as a result
    pub fn back_block(&mut self) -> &mut Walker {
        take_mut::take(&mut self.stepper, |mut stepper| {
            let mut rstepper = stepper.rev();

            // Iterate until we match the cursor.car
            let matched = loop {
                if let Some(DocGroup(attrs, _)) = rstepper.doc.head() {
                    if is_block(&attrs) {
                        break true;
                    }
                }
                if rstepper.next().is_none() {
                    break false;
                }
            };
            if !matched {
                panic!("Didn't find a block.");
            }

            rstepper.rev()
        });

        self
    }

    pub fn next_char(&mut self) -> &mut Walker {
        take_mut::take(&mut self.stepper, |mut stepper| {
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

            stepper
        });

        self
    }

    pub fn back_char(&mut self) -> &mut Walker {
        take_mut::take(&mut self.stepper, |mut stepper| {
            let mut rstepper = stepper.rev();

            let target_pos = rstepper.caret_pos - 1;

            // Iterate until we match the cursor.
            let matched = loop {
                if rstepper.caret_pos == target_pos && rstepper.is_valid_caret_pos() {
                    break true;
                }
                if rstepper.next().is_none() {
                    break false;
                }
            };

            rstepper.rev()
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
                    del.skip(1);
                    add.skip(1);
                    doc_stepper.skip(1);
                }
                Some(DocGroup(..)) => {
                    del.begin();
                    add.begin();
                    doc_stepper.enter();
                }
                None => {
                    del.exit();
                    add.exit();
                    if doc_stepper.is_done() {
                        break;
                    } else {
                        doc_stepper.exit();
                    }
                }
            }
        }

        OpWriter { del, add }
    }
}

pub struct OpWriter {
    pub del: DelWriter,
    pub add: AddWriter,
}

impl OpWriter {
    pub fn result(self) -> Op {
        (self.del.result(), self.add.result())
    }
}

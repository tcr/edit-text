use oatie::doc::*;
use oatie::stepper::*;
use oatie::writer::*;
use oatie::OT;

// tODO add a fast-forward option to skip to next caret??
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
}

impl Iterator for CaretStepper {
    type Item = ();

    fn next(&mut self) -> Option<()> {
        use oatie::schema::*;

        match self.doc.head() {
            Some(DocChars(..)) => {
                self.doc.skip(1);
                self.caret_pos += 1;
            }
            Some(DocGroup(attrs, _)) => {
                if Tag(attrs.clone()).tag_type() == Some(TrackType::Blocks) {
                    self.caret_pos += 1;
                }

                self.doc.enter();
            }
            None => if self.doc.is_done() {
                return None;
            } else {
                self.doc.exit();
            }
        }
        Some(())
    }
}


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
}

impl Iterator for ReverseCaretStepper {
    type Item = ();

    fn next(&mut self) -> Option<()> {
        use oatie::schema::*;

        match self.doc.head() {
            Some(DocChars(..)) => {
                self.caret_pos -= 1;
                self.doc.unskip(1);
            }
            Some(..) => {
                self.doc.unskip(1);
            }
            _ => {}
        }

        // TODO reverse is_done()

        match self.doc.head() {
            Some(DocChars(..)) => {
                // self.caret_pos -= 1;
            }
            Some(DocGroup(attrs, ..)) => {
                if Tag(attrs.clone()).tag_type() == Some(TrackType::Blocks) {
                    self.caret_pos -= 1;
                }
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
    pub doc: DocStepper,
    pub caret_pos: isize,
}

impl Walker {
    pub fn to_caret(doc: &Doc, client_id: &str) -> Walker {
        use oatie::schema::*;

        let original_doc = doc.clone();
        let mut doc = DocStepper::new(&doc.0);
        // TODO move cstep into Walker in place of doc / caret_pos
        let mut cstep = CaretStepper::new(doc);

        // Iterate until we match the cursor.
        let matched = loop {
            if let Some(DocGroup(attrs, _)) = cstep.doc.head() {
                if attrs["tag"] == "caret" && attrs["client"] == client_id {
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

        // Build return walker.
        let CaretStepper { doc, caret_pos } = cstep;
        Walker {
            original_doc,
            doc,
            caret_pos,
        }
    }

    pub fn to_cursor(doc: &Doc, cur: &CurSpan) -> Walker {
        use oatie::schema::*;

        // Walk the doc until the thing
        let mut walker = Walker {
            original_doc: doc.clone(),
            doc: DocStepper::new(&doc.0),
            caret_pos: -1,
        };

        let mut match_cur = CurStepper::new(cur);
        let mut matched = false;
        loop {
            match match_cur.head {
                Some(CurGroup) | Some(CurChar) => {
                    matched = true;
                    break;
                }
                _ => {}
            }

            match walker.doc.head() {
                Some(DocChars(..)) => {
                    walker.caret_pos += 1;

                    walker.doc.skip(1);
                    match_cur.skip();
                }
                Some(DocGroup(attrs, _)) => {
                    if Tag(attrs.clone()).tag_type() == Some(TrackType::Blocks) {
                        walker.caret_pos += 1;
                    }

                    walker.doc.enter();
                    match_cur.enter();
                }
                None => if walker.doc.is_done() {
                    break;
                } else {
                    walker.doc.exit();
                    match_cur.exit();
                },
            }
        }
        if !matched {
            panic!("Didn't find the cursor.");
        }

        walker
    }

    // TODO update the caret_pos as a result
    pub fn back_block(&mut self) -> &mut Walker {
        use oatie::schema::*;

        let mut cstep = ReverseCaretStepper {
            doc: self.doc.clone(),
            caret_pos: self.caret_pos,
        };

        // Iterate until we match the cursor.
        let matched = loop {
            if let Some(DocGroup(attrs, _)) = cstep.doc.peek() {
                if Tag(attrs.clone()).tag_type() == Some(TrackType::Blocks) {
                    cstep.doc.next();
                    break true;
                }
            }
            if cstep.next().is_none() {
                break false;
            }
        };
        if !matched {
            panic!("Didn't find a block.");
        }

        self.doc = cstep.doc;
        self.caret_pos = cstep.caret_pos;

        self

        // loop {
        //     // Find starting line of cursors
        //     // TODO this whole logic is bad for back_block, which should
        //     // just actually update the caret_pos as a result
        //     while self.doc.head_pos() > 0 {
        //         self.back_char();
        //     }
        //     self.doc.unenter();
        //     self.doc.next();

        //     match self.doc.head() {
        //         Some(DocGroup(attrs, ..)) => {
        //             if Tag(attrs.clone()).tag_type() == Some(TrackType::Blocks) {
        //                 break;
        //             }
        //         }
        //         _ => {}
        //     }
        // }
        // self
    }

    pub fn next_char(&mut self) -> &mut Walker {
        use oatie::schema::*;

        loop {
            match self.doc.head() {
                Some(DocChars(..)) => {
                    self.caret_pos += 1;
                    self.doc.skip(1);
                    break;
                }
                Some(DocGroup(attrs, _)) => {
                    if Tag(attrs.clone()).tag_type() == Some(TrackType::Blocks) {
                        self.caret_pos += 1;
                        break;
                    }

                    self.doc.enter();
                }
                None => if self.doc.is_done() {
                    break;
                } else {
                    self.doc.exit();
                },
            }
        }

        self
    }

    pub fn back_char(&mut self) -> &mut Walker {
        self.doc.unskip(1);
        self.snap_char()
    }

    pub fn snap_char(&mut self) -> &mut Walker {
        use oatie::schema::*;

        let mut matched = false;
        loop {
            match self.doc.head() {
                Some(DocChars(..)) => {
                    self.caret_pos -= 1;
                    matched = true;
                    println!("ooh");
                    break;
                }
                Some(DocGroup(..)) => {
                    self.doc.unenter();
                }
                None => {
                    // TODO there should be a backwards is_done()!!!
                    if self.doc.is_done() {
                        break;
                    } else {
                        self.doc.exit();
                        match self.doc.head() {
                            Some(DocGroup(attrs, _)) => {
                                self.doc.prev();
                                if Tag(attrs.clone()).tag_type() == Some(TrackType::Blocks) {
                                    self.caret_pos -= 1;
                                    break;
                                }
                            }
                            _ => unreachable!(),
                        }
                    }
                }
            }
        }
        if !matched {
            panic!("Couldn't snap to a character.");
        }
        self
    }

    pub fn to_writer(&self) -> OpWriter {
        let mut del = DelWriter::new();
        let mut add = AddWriter::new();

        // Walk the doc until we reach our current doc position.
        let mut doc_stepper = DocStepper::new(&self.original_doc.0);

        while self.doc != doc_stepper {
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

    pub fn apply_result(self, doc: &Doc) -> (Doc, Op) {
        let op = self.result();
        let new_doc = OT::apply(doc, &op);
        (new_doc, op)
    }
}

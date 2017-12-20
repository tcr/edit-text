use oatie::doc::*;
use oatie::stepper::*;
use oatie::writer::*;
use oatie::OT;

#[derive(Debug)]
pub struct Walker {
    original_doc: Doc,
    pub doc: DocStepper,
    caret_pos: isize,
}

impl Walker {
    pub fn to_caret(doc: &Doc) -> Walker {
        use oatie::schema::*;

        // Walk the doc until the thing
        let mut walker = Walker {
            original_doc: doc.clone(),
            doc: DocStepper::new(&doc.0),
            caret_pos: -1,
        };

        let mut matched = false;
        loop {
            match walker.doc.head() {
                Some(DocChars(..)) => {
                    walker.caret_pos += 1;
                    walker.doc.skip(1);
                },
                Some(DocGroup(attrs, _)) => {
                    if attrs["tag"] == "cursor" {
                        matched = true;
                        break;
                    }

                    if Tag(attrs.clone()).tag_type() == Some(TrackType::Blocks) {
                        walker.caret_pos += 1;
                    }

                    walker.doc.enter();
                }
                None => {
                    if walker.doc.is_done() {
                        break;
                    } else {
                        walker.doc.exit();
                    }
                }
            }
        }
        if !matched {
            panic!("Didn't find a caret.");
        }

        walker
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
                },
                Some(DocGroup(attrs, _)) => {
                    if Tag(attrs.clone()).tag_type() == Some(TrackType::Blocks) {
                        walker.caret_pos += 1;
                    }

                    walker.doc.enter();
                    match_cur.enter();
                }
                None => {
                    if walker.doc.is_done() {
                        break;
                    } else {
                        walker.doc.exit();
                        match_cur.exit();
                    }
                }
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

        loop {
            self.doc.unenter();
            self.doc.next();

            match self.doc.head() {
                Some(DocGroup(attrs, ..)) => {
                    if Tag(attrs.clone()).tag_type() == Some(TrackType::Blocks) {
                        break;
                    }
                }
                _ => {}
            }
        }
        self
    }

    pub fn next_char(&mut self) -> &mut Walker {
        use oatie::schema::*;

        loop {
            println!("next char {:?}", self.doc.head());
            match self.doc.head() {
                Some(DocChars(..)) => {
                    self.caret_pos += 1;
                    self.doc.skip(1);
                    break;
                },
                Some(DocGroup(attrs, _)) => {
                    if Tag(attrs.clone()).tag_type() == Some(TrackType::Blocks) {
                        self.caret_pos += 1;
                        break;
                    }

                    self.doc.enter();
                }
                None => {
                    if self.doc.is_done() {
                        break;
                    } else {
                        self.doc.exit();
                    }
                }
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
                    break;
                },
                Some(DocGroup(..)) => {
                    self.doc.unenter();
                }
                None => {
                    // TODO check backwards is_done()!!!
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
            panic!("Didn't find a cursor.");
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
                },
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

        OpWriter {
            del,
            add
        }
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
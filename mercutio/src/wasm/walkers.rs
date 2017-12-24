use oatie::doc::*;
use oatie::stepper::*;
use oatie::writer::*;
use oatie::OT;

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
        println!("next {:?}", self.doc.head());
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

        // // Debug
        // {
        //     let mut rstep = cstep.clone().rev();
        //     while let Some(..) = rstep.next() {
        //         continue;
        //     }
        //     println!("verified: {:?}", rstep.head);
        // }

        // Build return walker.
        let CaretStepper { doc, caret_pos } = cstep;
        Walker {
            original_doc,
            doc,
            caret_pos,
        }
    }

    pub fn to_cursor(doc: &Doc, cur: &CurSpan) -> Walker {
        let mut cstep = CaretStepper {
            doc: DocStepper::new(&doc.0),
            caret_pos: -1,
        };

        let mut match_cur = CurStepper::new(cur);
        let mut match_doc = DocStepper::new(&doc.0);

        println!("\n\nHI THERE: to_cursor");
        let mut matched = false;
        loop {
            match match_cur.head {
                Some(CurGroup) | Some(CurChar) => {
                    matched = true;
                    break;
                }
                _ => {}
            }

            // TODO make this an iterator also
            println!("-----> {:?}", cstep.doc.head());
            println!("-----  @> {:?}", match_cur.get_head());
            match match_cur.get_head() {
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

                    println!();
                    println!("yeah but {:?}", match_cur.head);
                    println!("yeah AND {:?}", match_doc.head());
                    println!();
                },
                // TODO merge this match with above
                Some(_) => unreachable!(),
            }

            // println!("going forever");
            while match_doc != cstep.doc {
                cstep.next();
                // println!("!!");
                // println!("!! _ {:?}", match_doc);
                // println!("!! V {:?}", cstep.doc);
                // println!("!!");
            }
        }
        if !matched {
            panic!("Didn't find the cursor.");
        }
        println!("\ndoing cstep\n");

        // let mut cstep = cstep.rev();

        while !cstep.is_valid_caret_pos() {
            cstep.next();
        }

        println!("\ndone\n");

        Walker {
            original_doc: doc.clone(),
            doc: cstep.doc,
            caret_pos: cstep.caret_pos,
        }
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
            if let Some(DocGroup(attrs, _)) = cstep.doc.head() {
                if is_block(&attrs) {
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
    }

    pub fn next_char(&mut self) -> &mut Walker {
        let mut cstep = CaretStepper {
            doc: self.doc.clone(),
            caret_pos: self.caret_pos,
        };

        // Iterate until we match the cursor.
        println!();
        let matched = loop {
            if cstep.caret_pos == self.caret_pos + 1 && cstep.is_valid_caret_pos() {
                break true;
            }
            if cstep.next().is_none() {
                break false;
            }
        };

        self.doc = cstep.doc;
        self.caret_pos = cstep.caret_pos;

        self
    }

    pub fn back_char(&mut self) -> &mut Walker {
        let mut cstep = ReverseCaretStepper {
            doc: self.doc.clone(),
            caret_pos: self.caret_pos,
        };

        println!("\n\nBACK\n\n");

        // Iterate until we match the cursor.
        let matched = loop {
            println!("wait {:?} --- {:?}", cstep.caret_pos, self.caret_pos);
            if cstep.caret_pos == self.caret_pos - 1 && cstep.is_valid_caret_pos() {
                break true;
            }
            // TODO need to set cstep.valid_char or nah
            if cstep.next().is_none() {
                break false;
            }
        };

        self.doc = cstep.doc;
        self.caret_pos = cstep.caret_pos;

        self
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

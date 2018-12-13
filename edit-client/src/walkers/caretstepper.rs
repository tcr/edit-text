use failure::Error;
use oatie::doc::*;
use oatie::stepper::*;
use oatie::transform::Schema;
use oatie::writer::*;
use take_mut;

pub fn is_block(attrs: &Attrs) -> bool {
    use oatie::schema::*;
    RtfSchema::track_type_from_attrs(attrs) == Some(RtfTrack::Blocks)
}

// TODO what does this refer to?
pub fn is_block_object(attrs: &Attrs) -> bool {
    use oatie::schema::*;
    RtfSchema::track_type_from_attrs(attrs) == Some(RtfTrack::BlockObjects)
}

pub fn is_caret(attrs: &Attrs, client_id: Option<&str>, focus: bool) -> bool {
    attrs["tag"] == "caret"
        && client_id
            .map(|id| attrs.get("client") == Some(&id.to_string()))
            .unwrap_or(false)
        && attrs.get("focus").map(|x| x == "true").unwrap_or(false) == focus
}

// Is any caret
pub fn is_any_caret(attrs: &Attrs) -> bool {
    attrs["tag"] == "caret"
}

#[derive(Clone, Debug)]
pub enum Pos {
    Start,
    Anchor,
    Focus,
    End,
}

#[derive(Clone, Debug)]
pub struct CaretStepper<'a> {
    pub doc: DocStepper<'a>,
    pub caret_pos: isize,
}

impl<'a> CaretStepper<'a> {
    pub fn new(doc: DocStepper<'a>) -> CaretStepper<'a> {
        // Start at caret pos 0
        let mut stepper = CaretStepper { doc, caret_pos: -1 };
        while stepper.caret_pos != 0 {
            if stepper.next().is_none() {
                break;
            }
        }
        stepper
    }

    pub fn rev(self) -> ReverseCaretStepper<'a> {
        ReverseCaretStepper {
            doc: self.doc,
            caret_pos: self.caret_pos,
        }
    }

    pub fn is_valid_caret_pos(&self) -> bool {
        if let Some(DocChars(..)) = self.doc.unhead() {
            return true;
        } else if self.doc.unhead().is_none() && !self.doc.is_back_done() {
            if is_block(self.doc.parent_attrs()) {
                return true;
            }
        }
        return false;
    }

    // TODO this is an easier alternative to .next() for skipping strings of chars,
    // but is it the best name or interface?
    pub fn skip_element(&mut self) -> Option<()> {
        let len = match self.doc.head() {
            Some(DocChars(val, _)) => {
                let len = val.char_len();
                self.doc.next();
                len
            }
            Some(DocGroup(..)) => {
                self.doc.enter();
                1
            }
            None => {
                if self.doc.is_done() {
                    return None;
                } else {
                    self.doc.exit();
                    1
                }
            }
        };

        if self.is_valid_caret_pos() {
            self.caret_pos += len as isize;
        }

        Some(())
    }
}

impl<'a> Iterator for CaretStepper<'a> {
    type Item = ();

    fn next(&mut self) -> Option<()> {
        match self.doc.head() {
            Some(DocChars(..)) => {
                self.doc.skip(1);
            }
            Some(DocGroup(..)) => {
                self.doc.enter();
            }
            None => {
                if self.doc.is_done() {
                    return None;
                } else {
                    self.doc.exit();
                }
            }
        }

        if self.is_valid_caret_pos() {
            self.caret_pos += 1;
        }

        Some(())
    }
}

#[derive(Clone, Debug)]
pub struct ReverseCaretStepper<'a> {
    pub doc: DocStepper<'a>,
    pub caret_pos: isize,
}

impl<'a> ReverseCaretStepper<'a> {
    pub fn rev(self) -> CaretStepper<'a> {
        CaretStepper {
            doc: self.doc,
            caret_pos: self.caret_pos,
        }
    }

    pub fn is_valid_caret_pos(&self) -> bool {
        // Skip over all preceding carets so we can identify the previous node
        // more easily.

        // Fast-path
        if let Some(DocChars(..)) = self.doc.unhead() {
            return true;
        } else if self.doc.unhead().is_none() {
            if self.doc.at_root() {
                // end of document, bail
                return false;
            }
            if is_block(self.doc.parent_attrs()) {
                return true;
            }
        }

        // Move back through the cursors, cloning the document stepper
        // TODO need a real caret stepper model so this can be avoided
        let mut doc2 = self.doc.clone();
        while let Some(DocGroup(ref attrs, _)) = doc2.unhead() {
            if is_any_caret(attrs) {
                doc2.unskip(1);
            } else {
                break;
            }
        }

        // Identically repeat fast-path logic
        if let Some(DocChars(..)) = doc2.unhead() {
            return true;
        } else if doc2.unhead().is_none() {
            if doc2.at_root() {
                // end of document, bail
                return false;
            }
            if is_block(doc2.parent_attrs()) {
                return true;
            }
        }
        return false;
    }
}

impl<'a> Iterator for ReverseCaretStepper<'a> {
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
                if self.doc.at_root() {
                    return None;
                } else {
                    self.doc.unenter();
                }
            }
        }

        if self.is_valid_caret_pos() {
            self.caret_pos -= 1;
        } else if let (None, true) = (self.doc.unhead(), self.doc.at_root()) {
            // Fix caret_pos to be -1 when we reach the end.
            // {edit.reset.caret_pos}
            self.caret_pos = -1;
        }

        Some(())
    }
}
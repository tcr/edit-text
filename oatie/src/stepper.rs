//! Enables stepping through a span operation.

#[macro_use]
use macros;

use doc::*;
use std::cmp;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct DelStepper {
    pub head: Option<DelElement>,
    pub rest: Vec<DelElement>,
    pub stack: Vec<Vec<DelElement>>,
}

impl DelStepper {
    pub fn new(span: &DelSpan) -> DelStepper {
        let mut ret = DelStepper {
            head: None,
            rest: span.to_vec(),
            stack: vec![],
        };
        ret.next();
        ret
    }

    pub fn next(&mut self) -> Option<DelElement> {
        let res = self.head.clone();
        self.head = if !self.rest.is_empty() {
            Some(self.rest.remove(0))
        } else {
            None
        };
        res
    }

    pub fn get_head(&self) -> DelElement {
        self.head.clone().unwrap()
    }

    pub fn is_done(&self) -> bool {
        self.head.is_none() && self.stack.is_empty()
    }

    pub fn enter(&mut self) {
        let head = self.head.clone();
        self.stack.push(self.rest.clone());
        match head {
            Some(DelGroup(ref span)) | Some(DelWithGroup(ref span)) => {
                self.head = None;
                self.rest = span.to_vec();
                self.next();
            }
            _ => panic!("DelStepper::enter() called on inappropriate element"),
        }
    }

    pub fn exit(&mut self) {
        let last = self.stack.pop().unwrap();
        self.rest = last;
        self.next();
    }

    pub fn into_span(self) -> DelSpan {
        let DelStepper { head, rest, .. } = self;
        if let Some(head) = head {
            let mut out = rest.to_vec();
            out.insert(0, head);
            out
        } else {
            vec![]
        }
    }
}

#[derive(Clone, Debug)]
pub struct AddStepper {
    pub head: Option<AddElement>,
    pub rest: Vec<AddElement>,
    pub stack: Vec<Vec<AddElement>>,
}

impl AddStepper {
    pub fn new(span: &AddSpan) -> AddStepper {
        let mut ret = AddStepper {
            head: None,
            rest: span.to_vec(),
            stack: vec![],
        };
        ret.next();
        ret
    }

    pub fn next(&mut self) -> Option<AddElement> {
        let res = self.head.clone();
        self.head = if !self.rest.is_empty() {
            Some(self.rest.remove(0))
        } else {
            None
        };
        res
    }

    pub fn get_head(&self) -> AddElement {
        self.head.clone().unwrap()
    }

    pub fn is_done(&self) -> bool {
        self.head.is_none() && self.stack.is_empty()
    }

    pub fn into_span(self) -> AddSpan {
        let AddStepper { head, rest, .. } = self;
        if let Some(head) = head {
            let mut out = rest.to_vec();
            out.insert(0, head);
            out
        } else {
            vec![]
        }
    }

    pub fn enter(&mut self) {
        let head = self.head.clone();
        self.stack.push(self.rest.clone());
        match head {
            Some(AddGroup(_, ref span)) | Some(AddWithGroup(ref span)) => {
                self.head = None;
                self.rest = span.to_vec();
                self.next();
            }
            _ => panic!("AddStepper::enter() called on inappropriate element"),
        }
    }

    pub fn exit(&mut self) {
        let last = self.stack.pop().unwrap();
        self.rest = last;
        self.next();
    }
}

#[derive(Clone, Debug)]
pub struct CurStepper {
    head: Option<CurElement>,
    pub rest: Vec<CurElement>,
    pub stack: Vec<Vec<CurElement>>,
}

use self::CurElement::*;

impl CurStepper {
    pub fn new(span: &CurSpan) -> CurStepper {
        let mut ret = CurStepper {
            head: None,
            rest: span.to_vec(),
            stack: vec![],
        };
        ret.next();
        ret
    }

    pub fn next(&mut self) -> Option<CurElement> {
        let res = self.head.clone();
        self.head = if !self.rest.is_empty() {
            Some(self.rest.remove(0))
        } else {
            None
        };
        res
    }

    pub fn head(&self) -> Option<CurElement> {
        self.head.clone()
    }

    pub fn is_done(&self) -> bool {
        self.head.is_none() && self.stack.is_empty()
    }

    pub fn skip(&mut self) {
        let do_next = match self.head {
            Some(CurSkip(ref mut count)) => {
                if *count > 1 {
                    *count -= 1;
                    false
                } else {
                    true
                }
            }
            Some(CurWithGroup(..)) | Some(CurGroup) | Some(CurChar) => true,
            _ => unimplemented!(),
        };
        if do_next {
            self.next();
        }
    }

    pub fn enter(&mut self) {
        let head = self.head.clone();
        self.stack.push(self.rest.clone());
        match head {
            Some(CurWithGroup(ref span)) => {
                self.head = None;
                self.rest = span.to_vec();
                self.next();
            }
            _ => panic!("CurStepper::enter() called on inappropriate element"),
        }
    }

    pub fn exit(&mut self) {
        let last = self.stack.pop().unwrap();
        self.rest = last;
        self.next();
    }
}

// DocStepper

// For now, this is the rental struct that DocStepper must use until
// all callsites are converted into supporting a lifetime-bound DocStepper.
rental! {
    pub mod doc_stepper {
        use super::*;

        #[rental(clone, debug, covariant)]
        pub struct DocStepperCursor {
            root: Arc<DocSpan>,
            inner: DocStepperInner<'root>,
        }
    }
}

// Where we define the impl on.
#[derive(Clone, Debug)]
pub struct DocStepper {
    cursor: doc_stepper::DocStepperCursor,
}

// Where the inner contents are.
#[derive(Clone, Debug)]
pub struct DocStepperInner<'a> {
    char_cursor: Option<CharCursor>,
    stack: Vec<(isize, &'a [DocElement])>,
}

// DocStepper impls

impl PartialEq for DocStepper {
    fn eq(&self, other: &DocStepper) -> bool {
        self.cursor.rent_all(|a| {
            other.cursor.rent_all(|b| {
                (a.inner.char_cursor.as_ref().map(|c| c.value()) == b.inner.char_cursor.as_ref().map(|c| c.value())
                    && a.inner.stack == b.inner.stack
                    && a.root == b.root)
            })
        })
    }
}

impl DocStepper {
    pub fn new(span: &DocSpan) -> DocStepper {
        let mut stepper = DocStepper {
            cursor: doc_stepper::DocStepperCursor::new(
                Arc::new(span.clone()),
                |span| DocStepperInner {
                    char_cursor: None,
                    stack: vec![(0, &span)],
                },
            ),
        };
        stepper.char_cursor_update();
        stepper
    }

    // Managing the char cursor, created each time we reach a DocChars.

    /// Move to the first character of the current string, or clear the
    /// cursor if we've reached a group.
    fn char_cursor_update(&mut self) {
        let cursor = if let Some(&DocChars(ref text)) = self.head_raw() {
            Some(CharCursor::from_docstring(text))
        } else {
            None
        };
        self.cursor.rent_all_mut(|target| target.inner.char_cursor = cursor);
    }

    /// Move to the last character - 1 of the current string, or clear the
    /// cursor if we've reached a group.
    fn char_cursor_update_prev(&mut self) {
        let cursor = match self.head() {
            Some(DocChars(ref text)) => {
                let mut cursor = CharCursor::from_docstring_end(text);
                cursor.value_sub(1);
                Some(cursor)
            }
            _ => None,
        };
        self.cursor.rent_all_mut(|target| target.inner.char_cursor = cursor);
    }

    fn char_cursor_expect(&self) -> &CharCursor {
        self.cursor.suffix().char_cursor.as_ref()
            .expect("Expected a generated char cursor")
    }

    fn char_cursor_expect_add(&mut self, add: usize) {
        self.cursor.rent_all_mut(|target| target.inner.char_cursor.as_mut()
            .expect("Expected a generated char cursor")
            .value_add(add));
    }

    fn char_cursor_expect_sub(&mut self, sub: usize) {
        self.cursor.rent_all_mut(|target| target.inner.char_cursor.as_mut()
            .expect("Expected a generated char cursor")
            .value_sub(sub));
    }

    // Current row in the stack is a DocSpan reference and an index.
    // What DocElement the index points to is the "head". If the head points
    // to a DocChars, we also create a char_cursor to index into the string.

    fn current<'a>(&'a self) -> &(isize, &'a [DocElement]) {
        self.cursor.suffix().stack.last().unwrap()
    }

    fn head_index(&self) -> usize {
        self.current().0 as usize
    }

    fn head_index_add<'a>(&'a mut self, add: usize) {
        self.cursor.rent_mut(|target| {
            target.stack.last_mut().unwrap().0 += add as isize;
        });
        self.char_cursor_update();
    }

    fn head_index_sub<'a>(&'a mut self, sub: usize) {
        self.cursor.rent_mut(|target| {
            target.stack.last_mut().unwrap().0 -= sub as isize;
        });
        self.char_cursor_update();
    }

    fn head_raw<'a>(&'a self) -> Option<&'a DocElement> {
        self.current().1.get(self.head_index())
    }

    fn unhead_raw<'a>(&'a self) -> Option<&'a DocElement> {
        // If we've split a string, don't modify the index.
        if self.cursor.suffix().char_cursor.as_ref()
            .map(|c| c.value() > 0)
            .unwrap_or(false) {
            return self.head_raw();
        }

        self.current().1.get(self.head_index() - 1)
    }

    // Cursor Public API

    pub fn next(&mut self) {
        self.head_index_add(1);
        self.char_cursor_update();
    }

    pub fn prev(&mut self) {
        self.head_index_sub(1);
        self.char_cursor_update_prev();
    }

    pub fn head(&self) -> Option<DocElement> {
        match self.head_raw() {
            Some(&DocChars(ref text)) => {
                // Expect cursor is at a string of length 1 at least
                // (meaning cursor has not passed to the end of the string)
                Some(DocChars(self.char_cursor_expect().right()
                    .expect("Encountered empty DocString").clone()))
            }
            Some(value) => Some(value.clone()),
            None => None,
        }
    }

    pub fn unhead(&self) -> Option<DocElement> {
        if let Some(&DocChars(ref text)) = self.head_raw() {
            // .left may be empty, so allow fall-through (don't .unwrap())
            if let Some(docstring) = self.char_cursor_expect().left() {
                return Some(DocChars(docstring.clone()));
            }
        }

        self.current().1
            .get((self.head_index() - 1) as usize)
            .map(|value| value.clone())
    }

    pub fn peek(&self) -> Option<DocElement> {
        match self.current().1.get((self.head_index() + 1) as usize) {
            Some(&DocChars(ref text)) => {
                // Pass along new text node
                Some(DocChars(text.clone()))
            }
            Some(value) => Some(value.clone()),
            None => None,
        }
    }

    pub fn unskip(&mut self, mut skip: usize) {
        while skip > 0 {
            match self.head_raw() {
                Some(DocChars(..)) => {
                    if self.char_cursor_expect().value() > 0 {
                        self.char_cursor_expect_sub(1);
                        skip -= 1;
                    } else {
                        self.prev();
                        skip -= 1;
                    }
                }
                Some(DocGroup(..)) | None => {
                    self.prev();
                    skip -= 1;
                }
            }
        }
    } 

    pub fn skip(&mut self, mut skip: usize) {
        while skip > 0 {
            let head = if let Some(head) = self.head_raw() {
                head
            } else {
                return;
            };

            match head {
                DocChars(ref text) => {
                    let remaining = text.char_len() - self.char_cursor_expect().value();
                    if skip >= remaining {
                        skip -= remaining;
                    } else {
                        self.char_cursor_expect_add(skip);
                        return;
                    }
                }
                DocGroup(..) => {
                    skip -= 1;
                }
            }
            self.next();
        }
    }

    /// The number of elements to skip until the end of the current group
    /// the stepper is tracking were reached. After that, head() returns None
    /// and exit() should be called.
    pub fn skip_len(&self) -> usize {
        self.current().1[self.head_index()..].to_vec().skip_len()
    }

    // Cursor stack operations.

    pub fn at_root(&self) -> bool {
        self.cursor.suffix().stack.len() <= 1
    }

    pub fn is_back_done(&self) -> bool {
        self.at_root() && self.unhead_raw().is_none()
    }

    pub fn is_done(&self) -> bool {
        self.at_root() && self.head_raw().is_none()
    }
    
    pub fn unenter(&mut self) -> &mut Self {
        self.cursor.rent_all_mut(|target| {
            target.inner.stack.pop();
        });
        self.char_cursor_update();
        self
    }

    pub fn enter(&mut self) -> &mut Self {
        self.cursor.rent_all_mut(|target| {
            let index = target.inner.stack.last().unwrap().0 as usize;
            if let &DocGroup(_, ref inner) = &target.inner.stack.last().unwrap().1[index] {
                target.inner.stack.push((0, inner));
            } else {
                panic!("DocStepper::enter() called on inappropriate element");
            }
        });
        self.char_cursor_update();
        self
    }

    pub fn unexit(&mut self) {
        self.prev();
        self.enter();
        self.head_index_add(self.current().1.len());
    }

    pub fn exit(&mut self) {
        self.unenter();
        self.next();
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[macro_use]
    use crate::macros;

    fn test_doc_0() -> DocSpan {
        doc_span![
            DocGroup({"tag": "h1"}, [
                DocChars("Cool"),
            ]),
        ]
    }

    #[test]
    fn docstepper_middle() {
        let doc = test_doc_0();
        let mut stepper = DocStepper::new(&doc);
        stepper.enter();
        stepper.skip(2);
        assert_eq!(stepper.head().unwrap(), DocChars(DocString::from_str("ol")));
    }

    #[test]
    #[should_panic]
    fn docstepper_peek_too_far_0() {
        let doc = test_doc_0();
        let mut stepper = DocStepper::new(&doc);
        stepper.enter();
        stepper.skip(2);
        stepper.peek().unwrap();
    }

    #[test]
    #[should_panic]
    fn docstepper_peek_too_far_1() {
        let doc = test_doc_0();
        let mut stepper = DocStepper::new(&doc);
        stepper.enter();
        stepper.skip(4);
        stepper.peek().unwrap();
    }

    #[test]
    #[should_panic]
    fn docstepper_peek_too_far_2() {
        let doc = test_doc_0();
        let mut stepper = DocStepper::new(&doc);
        stepper.enter();
        stepper.peek().unwrap();
    }

    #[test]
    fn docstepper_deep_0() {
        let doc = test_doc_0();
        let mut stepper = DocStepper::new(&doc);
        stepper.enter();
        stepper.skip(3);
        stepper.unskip(3);
        stepper.unenter();
        stepper.enter();
        stepper.skip(3);
        assert_eq!(stepper.peek().is_none(), true);
    }
}

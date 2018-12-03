use crate::stepper::*;
use crate::string::*;
use smallvec::SmallVec;

// DocStepper

// Where we define the impl on.
#[derive(Clone, Debug)]
pub struct DocStepper<'a> {
    pub(crate) char_cursor: Option<CharCursor>,
    pub(crate) stack: Vec<(isize, &'a [DocElement])>,
}

// DocStepper impls

impl<'a> PartialEq for DocStepper<'a> {
    fn eq(&self, b: &DocStepper<'a>) -> bool {
        let a = self;
        (a.char_cursor.as_ref().map(|c| c.value()) == b.char_cursor.as_ref().map(|c| c.value())
            && a.stack == b.stack)
    }
}

impl<'a> DocStepper<'a> {
    pub fn new<'b>(span: &'b [DocElement]) -> DocStepper<'b> {
        let mut stepper = DocStepper {
            char_cursor: None,
            stack: Vec::with_capacity(8),
        };
        stepper.stack.push((0, span));
        stepper.char_cursor_update();
        stepper
    }

    /// After an internal method changes what head will return, this method should be called.
    /// If head is a string, create a char_cursor on the first character of the string.
    /// If head is a group, clear the char_cursor.
    // TODO rename char_cursor_reset ?
    // TODO Make this pub(crate) once walkers.rs doesn't repend on it.
    #[no_mangle]
    pub fn char_cursor_update(&mut self) {
        let cursor = if let Some(&DocChars(ref text)) = self.head_raw() {
            Some(CharCursor::from_docstring(text))
        } else {
            None
        };
        self.char_cursor = cursor;
    }

    pub fn char_index(&self) -> Option<usize> {
        self.char_cursor
            .as_ref()
            .map(|cc| cc.left().map(|s| s.char_len()).unwrap_or(0))
    }

    /// Move to the last character - 1 of the current string, or clear the
    /// cursor if we've reached a group.
    pub(crate) fn char_cursor_update_prev(&mut self) {
        let cursor = match self.head() {
            Some(DocChars(ref text)) => {
                let mut cursor = CharCursor::from_docstring_end(text);
                cursor.value_sub(1);
                Some(cursor)
            }
            _ => None,
        };
        self.char_cursor = cursor;
    }

    pub fn char_cursor_expect(&self) -> &CharCursor {
        self.char_cursor
            .as_ref()
            .expect("Expected a generated char cursor")
    }

    fn char_cursor_expect_add(&mut self, add: usize) {
        self.char_cursor
            .as_mut()
            .expect("Expected a generated char cursor")
            .value_add(add);
    }

    fn char_cursor_expect_sub(&mut self, sub: usize) {
        self.char_cursor
            .as_mut()
            .expect("Expected a generated char cursor")
            .value_sub(sub);
    }

    // TODO hack around lifetime woes? in walkers.rs:to_writer
    pub unsafe fn raw_index(&self) -> (Option<usize>, Vec<isize>) {
        (
            self.char_cursor.as_ref().map(|cc| cc.value()),
            self.stack.iter().map(|(x, ..)| *x).collect::<Vec<_>>(),
        )
    }

    // Current row in the stack is a DocSpan reference and an index.
    // What DocElement the index points to is the "head". If the head points
    // to a DocChars, we also create a char_cursor to index into the string.

    pub(crate) fn current<'h>(&'h self) -> &'h (isize, &'a [DocElement]) {
        self.stack.last().unwrap()
    }

    pub fn parent_attrs(&self) -> &Attrs {
        let (index, ref list) = &self.stack[self.stack.len() - 2];
        if let DocGroup(ref attrs, ..) = &list[*index as usize] {
            attrs
        } else {
            unreachable!();
        }
    }

    pub(crate) fn head_index(&self) -> usize {
        self.current().0 as usize
    }

    pub(crate) fn head_index_add(&mut self, add: usize) {
        self.stack.last_mut().unwrap().0 += add as isize;
        self.char_cursor_update();
    }

    pub(crate) fn head_index_sub(&mut self, sub: usize) {
        self.stack.last_mut().unwrap().0 -= sub as isize;
        self.char_cursor_update();
    }

    pub(crate) fn head_raw<'h>(&'h self) -> Option<&'a DocElement> {
        self.current().1.get(self.head_index())
    }

    pub(crate) fn unhead_raw<'h>(&'h self) -> Option<&'a DocElement> {
        // If we've split a string, don't modify the index.
        if self
            .char_cursor
            .as_ref()
            .map(|c| c.value() > 0)
            .unwrap_or(false)
        {
            return self.head_raw();
        }

        self.current().1.get(self.head_index() - 1)
    }

    // Cursor Public API

    pub fn next(&mut self) {
        self.head_index_add(1);
        // TODO @aggressive_opt this is called in head_index_add, right?
        // self.char_cursor_update();
    }

    pub fn prev(&mut self) {
        self.head_index_sub(1);
        self.char_cursor_update_prev();
    }

    pub fn head<'h>(&'h self) -> Option<&'h DocElement> {
        match self.head_raw() {
            Some(&DocChars(ref text)) => {
                // Expect cursor is at a string of length 1 at least
                // (meaning cursor has not passed to the end of the string)
                Some(
                    self.char_cursor_expect()
                        .right_element()
                        .expect("Encountered empty DocString"),
                )
            }
            Some(ref value) => Some(value),
            None => None,
        }
    }

    pub fn unhead<'h>(&'h self) -> Option<&'h DocElement> {
        if let Some(&DocChars(..)) = self.head_raw() {
            // .left may be empty, so allow fall-through (don't .unwrap())
            if let Some(docstring) = self.char_cursor_expect().left_element() {
                return Some(docstring);
            }
        }

        self.current().1.get((self.head_index() - 1) as usize)
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
        if let Some(ref char_cursor) = &self.char_cursor {
            let remaining = char_cursor.index_from_end();
            if remaining == skip {
                self.next();
                return;
            } else if remaining > skip {
                self.char_cursor_expect_add(skip);
                return;
            } else {
                // remaining < skip, fall-through to loop
            }
        }

        while skip > 0 {
            let head = if let Some(head) = self.head_raw() {
                head
            } else {
                return;
            };

            match head {
                DocChars(ref text) => {
                    let remaining = text.char_len();
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
        self.stack.len() <= 1
    }

    pub fn is_back_done(&self) -> bool {
        self.at_root() && self.unhead_raw().is_none()
    }

    pub fn is_done(&self) -> bool {
        self.at_root() && self.head_raw().is_none()
    }

    pub fn unenter(&mut self) -> &mut Self {
        self.stack.pop();
        self.char_cursor_update();
        self
    }

    pub fn enter(&mut self) -> &mut Self {
        let index = self.stack.last().map(|x| x.0 as usize).unwrap();
        if let &DocGroup(_, ref inner) = self.stack.last().map(|x| &x.1[index]).unwrap() {
            self.stack.push((0, inner));
        } else {
            panic!("DocStepper::enter() called on inappropriate element");
        }
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

    pub(crate) fn exit_with_attrs(&mut self) -> Attrs {
        self.unenter();
        let attrs = if let Some(&DocGroup(ref attrs, ..)) = self.head_raw() {
            attrs.clone()
        } else {
            unreachable!();
        };
        self.next();
        attrs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        assert_eq!(
            stepper.head().unwrap(),
            &DocChars(DocString::from_str("ol"))
        );
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

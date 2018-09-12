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

// TODO can switch head to a usize??
rental! {
    pub mod doc_stepper {
        use crate::doc::*;
        use std::sync::Arc;

        #[rental(clone, debug, covariant)]
        pub struct DocStepperCursor {
            root: Arc<DocSpan>,
            stack: Vec<(isize, &'root [DocElement])>,
        }
    }
}

#[derive(Clone, Debug)]
pub struct DocStepper {
    pub char_debt: CharDebt,
    cursor: doc_stepper::DocStepperCursor,
}

impl PartialEq for DocStepper {
    fn eq(&self, other: &DocStepper) -> bool {
        self.char_debt == other.char_debt
            && self.cursor.rent_all(|a| {
                other.cursor.rent_all(|b| {
                    a.root == b.root && a.stack == b.stack
                })
            })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CharDebt(Option<(usize, DocString, usize)>);

// impl CharDebt {
//     fn () {
//         CharDebt {

//         }
//     }
// }

impl DocStepper {
    // Candidates to be moved to char_debt

    fn char_debt_value(&self) -> usize {
        if self.head_is_chars() {
            self.char_debt.0.as_ref().map(|x| x.0).unwrap_or(0)
        } else {
            unreachable!()
        }
    } 

    fn clear_char_debt(&mut self) {
        self.char_debt.0 = None;
    }

    fn char_debt_prepare(&mut self) {
        if let None = self.char_debt.0 {
            if let Some(&DocChars(ref text)) = self.head_raw() {
                self.char_debt.0 = Some((0, text.clone(), text.1.as_ref().map(|x| x.start).unwrap_or(0)));
            }
        }
    }

    fn char_debt_value_add(&mut self, add: usize) {
        self.char_debt_prepare();
        let tuple = self.char_debt.0.as_mut().unwrap();
        tuple.0 += add;
        unsafe {
            tuple.1.seek_forward(add);
        }
    }

    fn char_debt_value_sub(&mut self, sub: usize) {
        self.char_debt_prepare();
        let tuple = self.char_debt.0.as_mut().unwrap();
        tuple.0 -= sub;
        unsafe {
            tuple.1.seek_backward(sub);
        }
    }

    pub fn char_debt_docstring(&self) -> DocString {
        if let Some((ref _index, ref string, ..)) = self.char_debt.0.as_ref() {
            string.clone()
        } else {
            // Return a cloned version of current string, I guess
            match self.head_raw() {
                Some(&DocChars(ref text)) => {
                    text.clone()
                }
                _ => unreachable!(),
            }
        }
    }

    pub fn char_debt_docstring_prev(&self) -> Option<DocString> {
        if let Some((index, ref string, original_index)) = self.char_debt.0.as_ref() {
            let mut ret = string.clone();
            unsafe {
                let range = ret.byte_range_mut();
                let end = range.start;
                range.start = *original_index;
                range.end = end;
            }
            Some(ret)
        } else {
            None
        }
    }

    fn char_debt_prev(&mut self) {
        self.char_debt.0 = match self.head() {
            Some(DocChars(ref text)) => {
                Some((text.char_len() - 1, text.clone(), text.1.as_ref().map(|x| x.start).unwrap_or(0)))
            }
            _ => None,
        };
    }







    pub fn new(span: &DocSpan) -> DocStepper {
        DocStepper {
            char_debt: CharDebt(None),
            cursor: doc_stepper::DocStepperCursor::new(
                Arc::new(span.clone()),
                |span| vec![(0, &span)],
            ),
        }
    }

    pub fn prev(&mut self) -> Option<DocElement> {
        let res = self.head();
        self.head_index_sub(1);
        self.char_debt_prev();
        res
    }

    pub fn next(&mut self) -> Option<DocElement> {
        let res = self.head();
        self.head_index_add(1);
        self.clear_char_debt();
        res
    }








    fn index(&self) -> isize {
        self.cursor.suffix().last().unwrap().0
    }

    fn current<'a>(&'a self) -> &'a [DocElement] {
        &self.cursor.suffix().last().unwrap().1
    }

    pub fn stack<'a>(&'a self) -> &'a [(isize, &'a [DocElement])] {
        let mut vec = &self.cursor.suffix();
        &vec[0..vec.len() - 1]
    }

    fn head_index(&self) -> usize {
        self.cursor.suffix().last().unwrap().0 as usize
    }

    fn head_index_add<'a>(&'a mut self, add: usize) {
        self.cursor.rent_mut(|target| {
            target.last_mut().unwrap().0 += add as isize;
        });
    }

    fn head_index_sub<'a>(&'a mut self, sub: usize) {
        self.cursor.rent_mut(|target| {
            target.last_mut().unwrap().0 -= sub as isize;
        });
    }

    pub fn unenter(&mut self) -> &mut Self {
        self.cursor.rent_all_mut(|target| {
            target.stack.pop();
        });
        self
    }

    pub fn enter(&mut self) -> &mut Self {
        // let head = self.head_raw();
        self.cursor.rent_all_mut(|target| {
            let index = target.stack.last().unwrap().0 as usize;
            if let &DocGroup(_, ref inner) = &target.stack.last().unwrap().1[index] {
                target.stack.push((0, inner));
            } else {
                unreachable!();
            }
        });
        self.clear_char_debt();
        // match head {
        //     Some(DocGroup(ref attrs, ref span)) => {
        //         self.head = 0;
        //         self.clear_char_debt();
        //         self.cursor.rent_mut(|target| target.push((0, span)));
        //     }
        //     _ => panic!("DocStepper::enter() called on inappropriate element"),
        // }

        self
    }

    pub fn unexit(&mut self) {
        self.prev();
        self.enter();
        assert!(self.head_index() >= 0);
    }

    pub fn exit(&mut self) {
        self.unenter();

        // Increment pointer
        self.next();
    }












    fn head_raw<'a>(&'a self) -> Option<&'a DocElement> {
        self.current().get(self.head_index())
    }

    fn unhead_raw<'a>(&'a self) -> Option<&'a DocElement> {
        // If we've split a string, don't modify the index.
        if self.head_is_chars() {
            if self.char_debt_value() > 0 {
                return self.head_raw();
            }
        }

        self.current().get(self.head_index() - 1)
    }

    fn head_is_chars(&self) -> bool {
        if let Some(&DocChars(..)) = self.head_raw() {
            true
        } else {
            false
        }
    }

    pub fn head(&self) -> Option<DocElement> {
        match self.head_raw() {
            Some(&DocChars(ref text)) => {
                Some(DocChars(self.char_debt_docstring()))
            }
            Some(value) => Some(value.clone()),
            None => None,
        }
    }

    pub fn unhead(&self) -> Option<DocElement> {
        if let Some(&DocChars(ref text)) = self.head_raw() {
            let string = self.char_debt_docstring_prev().unwrap_or_else(|| text.clone());
            return Some(DocChars(string))
        }

        self.current()
            .get((self.head_index() - 1) as usize)
            .map(|value| value.clone())
    }

    pub fn peek(&self) -> Option<DocElement> {
        match self.current().get((self.head_index() + 1) as usize) {
            Some(&DocChars(ref text)) => {
                let (_, right) = text.split_at(self.char_debt_value());
                Some(DocChars(right))
            }
            Some(value) => Some(value.clone()),
            None => None,
        }
    }

    pub fn is_back_done(&self) -> bool {
        self.unhead_raw().is_none() && self.at_root()
    }

    pub fn is_done(&self) -> bool {
        self.head_raw().is_none() && self.at_root()
    }

    pub fn unskip(&mut self, mut skip: usize) {
        while skip > 0 && self.head_index() >= 0 {
            match self.head_raw() {
                Some(DocChars(..)) => {
                    if self.char_debt_value() > 0 {
                        self.char_debt_value_sub(1);
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
        while skip > 0 && !self.is_done() {
            match self.head_raw() {
                Some(DocChars(ref text)) => {
                    let remaining = text.char_len() - self.char_debt_value();
                    if skip >= remaining {
                        skip -= remaining;
                        self.next();
                    } else {
                        self.char_debt_value_add(skip);
                        break;
                    }
                }
                Some(DocGroup(..)) => {
                    self.next();
                    skip -= 1;
                }
                None => {
                    break;
                }
            }
        }
    }

    pub fn skip_len(&self) -> usize {
        self.current()[self.head_index()..].to_vec().skip_len()
    }

    pub fn at_root(&self) -> bool {
        self.stack().len() == 0
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

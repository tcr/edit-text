//! Enables stepping through a span operation.

use std::collections::HashMap;
use doc::*;
use std::borrow::ToOwned;
use std::cmp;

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
            Some(DelGroup(ref span)) |
            Some(DelWithGroup(ref span)) => {
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
            Some(AddGroup(_, ref span)) |
            Some(AddWithGroup(ref span)) => {
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
            Some(CurWithGroup(..)) | Some(CurGroup) | Some(CurChar) => {
                true
            }
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
#[derive(Clone, Debug, PartialEq)]
pub struct DocStepper {
    pub head: isize,
    pub char_debt: usize,
    rest: Vec<DocElement>,
    pub stack: Vec<(isize, Vec<DocElement>)>,
}

impl DocStepper {
    pub fn new(span: &DocSpan) -> DocStepper {
        DocStepper {
            head: 0,
            char_debt: 0,
            rest: span.to_vec(),
            stack: vec![],
        }
    }

    pub fn prev(&mut self) -> Option<DocElement> {
        let res = self.head();
        self.head -= 1;
        self.char_debt = match self.head() {
            Some(DocChars(ref text)) => text.char_len() - 1,
            _ => 0,
        };
        res
    }

    pub fn next(&mut self) -> Option<DocElement> {
        let res = self.head();
        self.head += 1;
        self.char_debt = 0;
        res
    }

    pub fn head_pos(&self) -> isize {
        self.head
    }

    pub fn head(&self) -> Option<DocElement> {
        match self.rest.get(self.head as usize) {
            Some(&DocChars(ref text)) => {
                Some(DocChars(text.clone().truncate(self.char_debt)))
            }
            Some(value) => {
                Some(value.clone())
            }
            None => None,
        }
    }

    pub fn unhead(&self) -> Option<DocElement> {
        if self.char_debt > 0 {
            if let Some(&DocChars(ref text)) = self.rest.get(self.head as usize) {
                return Some(DocChars(text.clone_slice(0, self.char_debt)));
            } else {
                unreachable!();
            }
        }

        self.rest.get((self.head - 1) as usize).map(|value| value.clone())
    }

    pub fn peek(&self) -> Option<DocElement> {
        match self.rest.get((self.head + 1) as usize) {
            Some(&DocChars(ref text)) => {
                Some(DocChars(text.clone().truncate(self.char_debt)))
            }
            Some(value) => {
                Some(value.clone())
            }
            None => None,
        }
    }

    pub fn is_back_done(&self) -> bool {
        self.unhead().is_none() && self.stack.is_empty()
    }

    pub fn is_done(&self) -> bool {
        self.head().is_none() && self.stack.is_empty()
    }

    pub fn unenter(&mut self) -> &mut Self {
        let (head, rest) = self.stack.pop().unwrap();
        self.head = head;
        self.char_debt = 0;
        self.rest = rest;

        self
    }

    pub fn enter(&mut self) -> &mut Self {
        let head = self.head();
        self.stack.push((self.head, self.rest.clone()));
        match head {
            Some(DocGroup(ref attrs, ref span)) => {
                self.head = 0;
                self.char_debt = 0;
                self.rest = span.to_vec();
            }
            _ => panic!("DocStepper::enter() called on inappropriate element"),
        }

        self
    }

    pub fn unexit(&mut self) {
        self.prev();

        let head = self.head();
        self.stack.push((self.head, self.rest.clone()));
        match head {
            Some(DocGroup(ref attrs, ref span)) => {
                self.head = span.len() as isize;
                self.char_debt = 0;
                self.rest = span.to_vec();
                // if span.len() > 0 {
                //     self.prev();
                // }
            }
            _ => panic!("Unexited wrong thing"),
        }

        assert!(self.head >= 0);
    }

    pub fn exit(&mut self) {
        let (head, rest) = self.stack.pop().unwrap();
        self.head = head;
        self.char_debt = 0;
        self.rest = rest;

        // Increment pointer
        self.next();
    }

    pub fn unskip(&mut self, mut skip: usize) {
        while skip > 0 && self.head >= 0 {
            match self.head() {
                Some(DocChars(ref inner)) => {
                    if self.char_debt > 0 {
                        self.char_debt -= 1;
                        skip -= 1;
                    } else {
                        self.prev();
                        skip -= 1;
                    }
                }
                Some(DocGroup(..)) => {
                    self.prev();
                    skip -= 1;
                }
                None => {
                    self.prev();
                    skip -= 1;
                }
            }
        }
    }

    pub fn skip(&mut self, mut skip: usize) {
        while skip > 0 && !self.is_done() {
            match self.head().unwrap() {
                DocChars(ref inner) => {
                    if inner.char_len() <= skip {
                        self.next();
                        skip -= inner.char_len();
                    } else {
                        self.char_debt += skip;
                        break;
                    }
                }
                DocGroup(..) => {
                    self.next();
                    skip -= 1;
                }
            }
        }
    }

    pub fn skip_len(&self) -> usize {
        self.rest[self.head as usize..].to_vec().skip_len()
    }
}


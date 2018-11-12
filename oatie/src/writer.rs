//! Classes for generating operation steps (add / del).

use std::borrow::ToOwned;
use std::cmp;
use std::collections::HashMap;

use compose;
use doc::*;
use normalize;
use stepper::*;

use failure::Error;
use term_painter::Attr::*;
use term_painter::Color::*;
use term_painter::ToStyle;

#[derive(Clone, Debug, Default)]
pub struct DelWriter {
    pub past: Vec<DelElement>,
    stack: Vec<Vec<DelElement>>,
}

impl DelWriter {
    pub fn new() -> DelWriter {
        DelWriter {
            past: vec![],
            stack: vec![],
        }
    }

    pub fn begin(&mut self) {
        let past = self.past.clone();
        self.past = vec![];
        self.stack.push(past);
    }

    pub fn exit(&mut self) {
        let past = self.past.clone();
        self.past = self
            .stack
            .pop()
            .expect("Cannot exit(), as we aren't in a group");

        if past.is_continuous_skip() {
            self.past.place(&DelSkip(1));
        } else {
            self.past.place(&DelWithGroup(past));
        }
    }

    pub fn close(&mut self) {
        let past = self.past.clone();
        self.past = self
            .stack
            .pop()
            .expect("Cannot close(), as we aren't in a group");
        self.past.place(&DelGroup(past));
    }

    pub fn exit_all(&mut self) {
        while !self.stack.is_empty() {
            self.exit();
        }
    }

    pub fn place(&mut self, elem: &DelElement) {
        self.past.place(elem);
    }

    pub fn place_all(&mut self, span: &DelSpan) {
        self.past.place_all(span);
    }

    pub fn result(self) -> DelSpan {
        if !self.stack.is_empty() {
            println!("{:?}", self);
            assert!(false, "cannot get result when stack is still full");
        }
        self.past
    }
}

#[derive(Clone, Debug, Default)]
pub struct AddWriter {
    pub past: Vec<AddElement>,
    stack: Vec<Vec<AddElement>>,
}

impl AddWriter {
    pub fn new() -> AddWriter {
        AddWriter {
            past: vec![],
            stack: vec![],
        }
    }

    pub fn begin(&mut self) {
        let past = self.past.clone();
        self.past = vec![];
        self.stack.push(past);
    }

    // TODO should there be an exit_strict that doesn't check is_continuous_skip()?
    pub fn exit(&mut self) {
        let past = self.past.clone();
        self.past = self
            .stack
            .pop()
            .expect("Cannot exit(), as we aren't in a group");

        if past.is_continuous_skip() {
            self.past.place(&AddSkip(1));
        } else {
            self.past.place(&AddWithGroup(past));
        }
    }

    pub fn close(&mut self, attrs: Attrs) {
        let past = self.past.clone();
        self.past = self
            .stack
            .pop()
            .expect("Cannot close(), as we aren't in a group");

        self.past.place(&AddGroup(attrs, past));
    }

    pub fn exit_all(&mut self) {
        while !self.stack.is_empty() {
            self.exit();
        }
    }

    pub fn place(&mut self, elem: &AddElement) {
        self.past.place(elem);
    }

    pub fn place_all(&mut self, span: &AddSpan) {
        self.past.place_all(span);
    }

    pub fn result(self) -> AddSpan {
        if !self.stack.is_empty() {
            println!("{:?}", self);
            assert!(false, "cannot get result when stack is still full");
        }
        self.past
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

#[derive(Clone, Debug, Default)]
pub struct CurWriter {
    pub past: Vec<CurElement>,
    stack: Vec<Vec<CurElement>>,
}

impl CurWriter {
    pub fn new() -> CurWriter {
        CurWriter {
            past: vec![],
            stack: vec![],
        }
    }

    pub fn begin(&mut self) {
        let past = self.past.clone();
        self.past = vec![];
        self.stack.push(past);
    }

    pub fn exit(&mut self) {
        let past = self.past.clone();
        self.past = self.stack.pop().unwrap();
        self.past.push(CurWithGroup(past));
    }

    pub fn exit_all(&mut self) {
        while !self.stack.is_empty() {
            self.exit();
        }
    }

    pub fn place(&mut self, elem: &CurElement) {
        self.past.place(elem);
    }

    pub fn place_all(&mut self, span: &CurSpan) {
        self.past.place_all(span);
    }

    pub fn result(self) -> CurSpan {
        if !self.stack.is_empty() {
            println!("{:?}", self);
            assert!(false, "cannot get result when stack is still full");
        }
        self.past
    }
}

#[derive(Clone, Debug)]
pub struct DocWriter {
    pub(crate) past: Vec<DocElement>, // TODO not public
    stack: Vec<Vec<DocElement>>,
}

impl DocWriter {
    pub fn new() -> DocWriter {
        DocWriter {
            past: vec![],
            stack: vec![],
        }
    }

    pub fn begin(&mut self) {
        let past = self.past.clone();
        self.past = vec![];
        self.stack.push(past);
    }

    pub fn close(&mut self, attrs: HashMap<String, String>) {
        let past = self.past.clone();
        self.past = self.stack.pop().unwrap();
        self.past.push(DocGroup(attrs, past));
    }

    pub fn place(&mut self, elem: &DocElement) {
        self.past.place(elem);
    }

    pub fn place_all(&mut self, span: &DocSpan) {
        self.past.place_all(span);
    }

    pub fn result(self) -> Result<DocSpan, Error> {
        if !self.stack.is_empty() {
            println!("{:?}", self);
            bail!("cannot get result when stack is still full");
        }
        Ok(self.past)
    }

    pub(crate) fn unwrap_self(&mut self) {
        while !self.past.is_empty() {
            self.stack.last_mut().unwrap().push(self.past.remove(0));
        }
        self.past = self.stack.pop().unwrap();
    }

    pub(crate) fn wrap_previous(&mut self, count: usize, attrs: Attrs) {
        let start = self.past.len() - count;
        let group = self.past.split_off(start);
        self.past.push(DocGroup(attrs, group));
    }
}

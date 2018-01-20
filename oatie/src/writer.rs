use std::collections::HashMap;
use std::borrow::ToOwned;
use std::cmp;

use doc::*;
use stepper::*;
use compose;
use normalize;

use term_painter::ToStyle;
use term_painter::Color::*;
use term_painter::Attr::*;


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
        self.past = self.stack.pop().expect("Cannot exit(), as we aren't in a group");
        self.past.push(DelWithGroup(past));
    }

    pub fn close(&mut self) {
        let past = self.past.clone();
        self.past = self.stack.pop().expect("Cannot close(), as we aren't in a group");
        self.past.push(DelGroup(past));
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

    pub fn exit(&mut self) {
        let past = self.past.clone();
        self.past = self.stack.pop().unwrap();
        self.past.push(AddWithGroup(past));
    }

    pub fn close(&mut self, attrs: Attrs) {
        let past = self.past.clone();
        self.past = self.stack.pop().unwrap();

        self.past.push(AddGroup(attrs, past));
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

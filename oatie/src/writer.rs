//! Classes for generating operation steps (add / del).

use crate::doc::*;
use failure::Error;

#[derive(Clone, Debug, Default)]
pub struct DelWriter<S: Schema> {
    pub past: Vec<DelElement<S>>,
    stack: Vec<Vec<DelElement<S>>>,
}

impl<S: Schema> DelWriter<S> {
    pub fn new() -> DelWriter<S> {
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

    pub fn place(&mut self, elem: &DelElement<S>) {
        self.past.place(elem);
    }

    pub fn place_all(&mut self, span: &DelSpan<S>) {
        self.past.place_all(span);
    }

    pub fn result(self) -> DelSpan<S> {
        if !self.stack.is_empty() {
            println!("{:?}", self);
            assert!(false, "cannot get result when stack is still full");
        }
        self.past
    }
}

#[derive(Clone, Debug, Default)]
pub struct AddWriter<S: Schema> {
    pub past: Vec<AddElement<S>>,
    stack: Vec<Vec<AddElement<S>>>,
}

impl<S: Schema> AddWriter<S> {
    pub fn new() -> AddWriter<S> {
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

    pub fn close(&mut self, attrs: S::GroupProperties) {
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

    pub fn place(&mut self, elem: &AddElement<S>) {
        self.past.place(elem);
    }

    pub fn place_all(&mut self, span: &AddSpan<S>) {
        self.past.place_all(span);
    }

    pub fn result(self) -> AddSpan<S> {
        if !self.stack.is_empty() {
            println!("{:?}", self);
            assert!(false, "cannot get result when stack is still full");
        }
        self.past
    }
}

pub struct OpWriter<S: Schema> {
    pub del: DelWriter<S>,
    pub add: AddWriter<S>,
}

impl<S: Schema> OpWriter<S> {
    pub fn exit_result(mut self) -> Op<S> {
        self.del.exit_all();
        self.add.exit_all();
        self.result()
    }

    pub fn result(self) -> Op<S> {
        Op(self.del.result(), self.add.result())
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
pub struct DocWriter<S: Schema> {
    pub(crate) past: Vec<DocElement<S>>, // TODO not public
    stack: Vec<Vec<DocElement<S>>>,
}

impl<S: Schema> DocWriter<S> {
    pub fn new() -> DocWriter<S> {
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

    pub fn close(&mut self, attrs: S::GroupProperties) {
        let past = self.past.clone();
        self.past = self.stack.pop().unwrap();
        self.past.push(DocGroup(attrs, past));
    }

    pub fn place(&mut self, elem: &DocElement<S>) {
        self.past.place(elem);
    }

    pub fn place_all(&mut self, span: &DocSpan<S>) {
        self.past.place_all(span);
    }

    pub fn result(self) -> Result<DocSpan<S>, Error> {
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

    pub(crate) fn wrap_previous(&mut self, count: usize, attrs: S::GroupProperties) {
        let start = self.past.len() - count;
        let group = self.past.split_off(start);
        self.past.push(DocGroup(attrs, group));
    }
}

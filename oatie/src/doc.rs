//! Document definitions.

use std::collections::HashMap;

pub type Attrs = HashMap<String, String>;


pub type DocSpan = Vec<DocElement>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DocElement {
    DocChars(String),
    DocGroup(Attrs, DocSpan),
}

pub use self::DocElement::*;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Doc(pub Vec<DocElement>);


pub trait DocPlaceable {
    fn skip_len(&self) -> usize;
}

impl DocPlaceable for DocSpan {
    fn skip_len(&self) -> usize {
        let mut ret = 0;
        for item in self {
            ret += match *item {
                DocChars(ref value) => value.chars().count(),
                DocGroup(..) => 1,
            };
        }
        ret
    }
}




pub type DelSpan = Vec<DelElement>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DelElement {
    DelSkip(usize),
    DelWithGroup(DelSpan),
    DelChars(usize),
    DelGroup(DelSpan),
    DelGroupAll,
    DelMany(usize),

    // TODO what
    DelObject,
}

pub use self::DelElement::*;


pub type AddSpan = Vec<AddElement>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum AddElement {
    AddSkip(usize),
    AddWithGroup(AddSpan),
    AddChars(String),
    AddGroup(Attrs, AddSpan),
}

pub use self::AddElement::*;

pub type Op = (DelSpan, AddSpan);


fn del_place_chars(res: &mut DelSpan, count: usize) {
    if !res.is_empty() {
        let idx = res.len() - 1;
        if let DelChars(ref mut prefix) = res[idx] {
            *prefix += count;
            return;
        }
    }
    res.push(DelChars(count));
}

fn del_place_skip(res: &mut DelSpan, count: usize) {
    if !res.is_empty() {
        let idx = res.len() - 1;
        if let DelSkip(ref mut prefix) = res[idx] {
            *prefix += count;
            return;
        }
    }

    assert!(count > 0);
    res.push(DelSkip(count));
}

fn del_place_many(res: &mut DelSpan, count: usize) {
    if !res.is_empty() {
        let idx = res.len() - 1;
        if let DelMany(ref mut prefix) = res[idx] {
            *prefix += count;
            return;
        }
    }

    assert!(count > 0);
    res.push(DelMany(count));
}

fn del_place_any(res: &mut DelSpan, value: &DelElement) {
    match *value {
        DelChars(count) => {
            del_place_chars(res, count);
        }
        DelSkip(count) => {
            del_place_skip(res, count);
        }
        DelMany(count) => {
            del_place_many(res, count);
        }
        DelGroup(..) | DelGroupAll | DelWithGroup(..) | DelObject => {
            res.push(value.clone());
        }
    }
}

fn add_place_chars(res: &mut AddSpan, value: String) {
    if !res.is_empty() {
        let idx = res.len() - 1;
        if let AddChars(ref mut prefix) = res[idx] {
            prefix.push_str(&value[..]);
            return;
        }
    }
    res.push(AddChars(value));
}

fn add_place_skip(res: &mut AddSpan, count: usize) {
    if !res.is_empty() {
        let idx = res.len() - 1;
        if let AddSkip(ref mut prefix) = res[idx] {
            *prefix += count;
            return;
        }
    }

    assert!(count > 0);
    res.push(AddSkip(count));
}

fn add_place_any(res: &mut AddSpan, value: &AddElement) {
    match *value {
        AddChars(ref value) => {
            add_place_chars(res, value.clone());
        }
        AddSkip(count) => {
            add_place_skip(res, count);
        }
        _ => {
            res.push(value.clone());
        }
    }
}

pub trait DelPlaceable {
    fn place_all(&mut self, all: &[DelElement]);
    fn place(&mut self, value: &DelElement);
    fn skip_len(&self) -> usize;
    fn skip_post_len(&self) -> usize;
}

impl DelPlaceable for DelSpan {
    fn place_all(&mut self, all: &[DelElement]) {
        for i in all {
            self.place(i);
        }
    }

    fn place(&mut self, value: &DelElement) {
        del_place_any(self, value);
    }

    fn skip_len(&self) -> usize {
        let mut ret = 0;
        for item in self {
            ret += match *item {
                DelSkip(len) | DelChars(len) | DelMany(len) => len,
                DelObject | DelGroup(..) | DelGroupAll | DelWithGroup(..) => 1,
            };
        }
        ret
    }

    fn skip_post_len(&self) -> usize {
        let mut ret = 0;
        for item in self {
            ret += match *item {
                DelSkip(len) => len,
                DelObject | DelChars(..) | DelMany(..) | DelGroupAll => 0,
                DelWithGroup(..) => 1,
                DelGroup(ref span) => span.skip_post_len(),
            };
        }
        ret
    }
}

pub trait AddPlaceable {
    fn place_all(&mut self, all: &[AddElement]);
    fn place(&mut self, value: &AddElement);
    fn skip_pre_len(&self) -> usize;
    fn skip_len(&self) -> usize;
}

impl AddPlaceable for AddSpan {
    fn place_all(&mut self, all: &[AddElement]) {
        for i in all {
            self.place(i);
        }
    }

    fn place(&mut self, value: &AddElement) {
        add_place_any(self, value);
    }

    fn skip_pre_len(&self) -> usize {
        let mut ret = 0;
        for item in self {
            ret += match *item {
                AddSkip(len) => len,
                AddChars(ref chars) => 0,
                AddGroup(..) => 0,
                AddWithGroup(..) => 1,
            };
        }
        ret
    }

    fn skip_len(&self) -> usize {
        let mut ret = 0;
        for item in self {
            ret += match *item {
                AddSkip(len) => len,
                AddChars(ref chars) => chars.chars().count(),
                AddGroup(..) | AddWithGroup(..) => 1,
            };
        }
        ret
    }
}



pub type CurSpan = Vec<CurElement>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CurElement {
    CurSkip(usize),
    CurWithGroup(CurSpan),
    CurGroup,
    CurChar,
}

pub use self::CurElement::*;

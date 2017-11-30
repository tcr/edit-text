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


pub type DelSpan = Vec<DelElement>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DelElement {
    DelSkip(usize),
    DelWithGroup(DelSpan),
    DelChars(usize),
    DelGroup(DelSpan),
    DelGroupAll,
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
    if res.len() > 0 {
        let idx = res.len() - 1;
        if let DelChars(ref mut prefix) = res[idx] {
            *prefix += count;
            return;
        }
    }
    res.push(DelChars(count));
}

fn del_place_skip(res: &mut DelSpan, count: usize) {
    if res.len() > 0 {
        let idx = res.len() - 1;
        if let DelSkip(ref mut prefix) = res[idx] {
            *prefix += count;
            return;
        }
    }
    res.push(DelSkip(count));
}

fn del_place_any(res: &mut DelSpan, value: &DelElement) {
    match value {
        &DelChars(count) => {
            del_place_chars(res, count);
        }
        &DelSkip(count) => {
            del_place_skip(res, count);
        }
        _ => {
            res.push(value.clone());
        }
    }
}

fn add_place_chars(res: &mut AddSpan, value: String) {
    if res.len() > 0 {
        let idx = res.len() - 1;
        if let &mut AddChars(ref mut prefix) = &mut res[idx] {
            prefix.push_str(&value[..]);
            return;
        }
    }
    res.push(AddChars(value));
}

fn add_place_skip(res: &mut AddSpan, count: usize) {
    if res.len() > 0 {
        let idx = res.len() - 1;
        if let &mut AddSkip(ref mut prefix) = &mut res[idx] {
            *prefix += count;
            return;
        }
    }
    res.push(AddSkip(count));
}

fn add_place_any(res: &mut AddSpan, value: &AddElement) {
    match value {
        &AddChars(ref value) => {
            add_place_chars(res, value.clone());
        }
        &AddSkip(count) => {
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
            ret += match item {
                &DelSkip(len) => len,
                &DelChars(len) => len,
                &DelGroup(..) | &DelGroupAll | &DelWithGroup(..) => 1,
            };
        }
        ret
    }

    fn skip_post_len(&self) -> usize {
        let mut ret = 0;
        for item in self {
            ret += match item {
                &DelSkip(len) => len,
                &DelChars(..) => 0,
                &DelGroup(..) | &DelGroupAll => 0,
                &DelWithGroup(..) => 1,
            };
        }
        ret
    }
}

pub trait AddPlaceable {
    fn place_all(&mut self, all: &[AddElement]);
    fn place(&mut self, value: &AddElement);
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

    fn skip_len(&self) -> usize {
        let mut ret = 0;
        for item in self {
            ret += match item {
                &AddSkip(len) => len,
                &AddChars(ref chars) => chars.len(),
                &AddGroup(..) | &AddWithGroup(..) => 1,
            };
        }
        ret
    }
}

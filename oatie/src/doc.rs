//! Document definitions.

use std::collections::HashMap;

pub use self::DocElement::*;
pub use self::AddElement::*;
pub use self::DelElement::*;

pub type Attrs = HashMap<String, String>;
pub type DocSpan = Vec<DocElement>;

pub type DelSpan = Vec<DelElement>;
pub type AddSpan = Vec<AddElement>;
pub type Op = (DelSpan, AddSpan);

pub type CurSpan = Vec<CurElement>;


#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DocElement {
    DocChars(String),
    DocGroup(Attrs, DocSpan),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Doc(pub Vec<DocElement>);

pub trait DocPlaceable {
    fn skip_len(&self) -> usize;
    fn place_all(&mut self, all: &[DocElement]);
    fn place(&mut self, value: &DocElement);
}

impl DocPlaceable for DocSpan {
    fn place_all(&mut self, all: &[DocElement]) {
        for i in all {
            self.place(i);
        }
    }

    fn place(&mut self, elem: &DocElement) {
        match *elem {
            DocChars(ref text) => {
                assert!(text.chars().count() > 0);
                if let Some(&mut DocChars(ref mut value)) = self.last_mut() {
                    value.push_str(text);
                } else {
                    self.push(DocChars(text.to_owned()));
                }
            }
            DocGroup(..) => {
                self.push(elem.clone());
            }
        }
    }

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


#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DelElement {
    DelSkip(usize),
    DelWithGroup(DelSpan),
    DelChars(usize),
    DelGroup(DelSpan),

    // TODO Implement these
    // DelGroupAll,
    // DelMany(usize),
    // DelObject,
}

pub trait DelPlaceable {
    fn place_all(&mut self, all: &[DelElement]);
    fn place(&mut self, value: &DelElement);
    fn skip_pre_len(&self) -> usize;
    fn skip_post_len(&self) -> usize;
}

impl DelPlaceable for DelSpan {
    fn place_all(&mut self, all: &[DelElement]) {
        for i in all {
            self.place(i);
        }
    }

    fn place(&mut self, elem: &DelElement) {
        match *elem {
            DelChars(count) => {
                assert!(count > 0);
                if let Some(&mut DelChars(ref mut value)) = self.last_mut() {
                    *value += count;
                } else {
                    self.push(DelChars(count));
                }
            }
            DelSkip(count) => {
                assert!(count > 0);
                if let Some(&mut DelSkip(ref mut value)) = self.last_mut() {
                    *value += count;
                } else {
                    self.push(DelSkip(count));
                }
            }
            DelGroup(..) | DelWithGroup(..) => {
                self.push(elem.clone());
            }
            // DelGroupAll | DelObject => {
            //     unimplemented!();
            // }
            // DelMany(count) => {
            //     unimplemented!();
            // }
        }
    }

    fn skip_pre_len(&self) -> usize {
        let mut ret = 0;
        for item in self {
            ret += match *item {
                DelSkip(len) | DelChars(len) => len,
                DelGroup(..) |DelWithGroup(..) => 1,
                // DelMany(len) => len,
                // DelObject | DelGroupAll  => 1,
            };
        }
        ret
    }

    fn skip_post_len(&self) -> usize {
        let mut ret = 0;
        for item in self {
            ret += match *item {
                DelSkip(len) => len,
                DelChars(..) => 0,
                DelWithGroup(..) => 1,
                DelGroup(ref span) => span.skip_post_len(),
                // DelObject | DelMany(..) | DelGroupAll => 0,
            };
        }
        ret
    }
}


#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum AddElement {
    AddSkip(usize),
    AddWithGroup(AddSpan),
    AddChars(String),
    AddGroup(Attrs, AddSpan),
}

pub trait AddPlaceable {
    fn place_all(&mut self, all: &[AddElement]);
    fn place(&mut self, value: &AddElement);
    fn skip_pre_len(&self) -> usize;
    fn skip_post_len(&self) -> usize;
}

impl AddPlaceable for AddSpan {
    fn place_all(&mut self, all: &[AddElement]) {
        for i in all {
            self.place(i);
        }
    }

    fn place(&mut self, elem: &AddElement) {
        match *elem {
            AddChars(ref text) => {
                assert!(text.chars().count() > 0);
                if let Some(&mut AddChars(ref mut value)) = self.last_mut() {
                    value.push_str(text);
                } else {
                    self.push(AddChars(text.to_owned()));
                }
            }
            AddSkip(count) => {
                assert!(count > 0);
                if let Some(&mut AddSkip(ref mut value)) = self.last_mut() {
                    *value += count;
                } else {
                    self.push(AddSkip(count));
                }
            }
            AddGroup(..) | AddWithGroup(..) => {
                self.push(elem.clone());
            }
        }
    }

    fn skip_pre_len(&self) -> usize {
        let mut ret = 0;
        for item in self {
            ret += match *item {
                AddSkip(len) => len,
                AddChars(ref chars) => 0,
                AddGroup(_, ref span) => span.skip_pre_len(),
                AddWithGroup(..) => 1,
            };
        }
        ret
    }

    fn skip_post_len(&self) -> usize {
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


#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CurElement {
    CurSkip(usize),
    CurWithGroup(CurSpan),
    CurGroup,
    CurChar,
}

pub use self::CurElement::*;

pub trait CurPlaceable {
    fn place_all(&mut self, all: &[CurElement]);
    fn place(&mut self, value: &CurElement);
}

impl CurPlaceable for CurSpan {
    fn place_all(&mut self, all: &[CurElement]) {
        for i in all {
            self.place(i);
        }
    }

    fn place(&mut self, elem: &CurElement) {
        match *elem {
            CurSkip(count) => {
                assert!(count > 0);
                if let Some(&mut CurSkip(ref mut value)) = self.last_mut() {
                    *value += count;
                } else {
                    self.push(CurSkip(count));
                }
            }
            CurGroup | CurChar | CurWithGroup(..) => {
                self.push(elem.clone());
            }
        }
    }
}
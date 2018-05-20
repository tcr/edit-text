//! Defines document types, operation types, and cursor types.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use std::ops::Range;

pub use self::AddElement::*;
pub use self::DelElement::*;
pub use self::DocElement::*;

pub type Attrs = HashMap<String, String>;
pub type DocSpan = Vec<DocElement>;

pub type DelSpan = Vec<DelElement>;
pub type AddSpan = Vec<AddElement>;
pub type Op = (DelSpan, AddSpan);

pub type CurSpan = Vec<CurElement>;

/// Abstraction for String that allows a limited set of operations
/// with good optimization. (Or that's the idea.)
#[derive(Clone, Debug, PartialEq)]
pub struct DocString(Arc<String>, Option<Range<usize>>);

impl DocString {
    pub fn from_string(input: String) -> DocString {
        DocString(Arc::new(input), None)
    }

    pub fn from_slice(input: &[char]) -> DocString {
        DocString(Arc::new(input.iter().collect()), None)
    }

    pub fn from_str(input: &str) -> DocString {
        DocString(Arc::new(input.to_owned()), None)
    }

    pub fn as_str(&self) -> &str {
        if let Some(ref range) = self.1 {
            &self.0[range.clone()]
        } else {
            &self.0
        }
    }

    pub fn push_str(&mut self, input: &str) {
        let mut value = self.to_string();
        value.push_str(input);
        self.0 = Arc::new(value);
        self.1 = None;
    }

    // TODO consume self?
    pub fn split_at(&self, char_boundary: usize) -> (DocString, DocString) {
        let (byte_index, _) = self.as_str().char_indices().skip(char_boundary).next().unwrap();
        let mut start = 0;
        let mut end = self.0.len();
        if let Some(ref range) = self.1 {
            start = range.start;
            end = range.end;
        }
        (
            DocString(self.0.clone(), Some((start + 0)..(start + byte_index))),
            DocString(self.0.clone(), Some((start + byte_index)..end)),
        )
    }

    pub fn to_string(&self) -> String {
        self.as_str().to_owned()
    }

    pub fn is_empty(&self) -> bool {
        self.as_str().is_empty()
    }

    pub fn push_doc_string(&mut self, input: &DocString) {
        self.push_str(input.as_str());
    }

    pub fn into_string(self) -> String {
        // TODO make this faster by deconstructing the Rc?
        self.to_string()
    }

    pub fn char_len(&self) -> usize {
        self.as_str().chars().count()
    }
}

impl Serialize for DocString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for DocString {
    fn deserialize<D>(deserializer: D) -> Result<DocString, D::Error>
    where
        D: Deserializer<'de>,
    {
        let string = <String as Deserialize>::deserialize(deserializer)?;
        Ok(DocString::from_string(string))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DocElement {
    DocChars(DocString),
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
                assert!(text.char_len() > 0);
                if let Some(&mut DocChars(ref mut value)) = self.last_mut() {
                    value.push_doc_string(text);
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
                DocChars(ref value) => value.char_len(),
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

    /// Optimization for depth-first code to recursively return skips up
    /// the walker.
    fn is_continuous_skip(&self) -> bool;
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
            } // DelGroupAll | DelObject => {
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
                DelGroup(..) | DelWithGroup(..) => 1,
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

    fn is_continuous_skip(&self) -> bool {
        if self.len() > 1 {
            // Will never be a continuous skip
            false
        } else if self.is_empty() {
            // is []
            true
        } else if let DelSkip(_) = self[0] {
            // is [DelSkip(n)]
            true
        } else {
            // is [DelSomething(n)]
            false
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum AddElement {
    AddSkip(usize),
    AddWithGroup(AddSpan),
    AddChars(DocString),
    AddGroup(Attrs, AddSpan),
}

pub trait AddPlaceable {
    fn place_all(&mut self, all: &[AddElement]);
    fn place(&mut self, value: &AddElement);
    fn skip_pre_len(&self) -> usize;
    fn skip_post_len(&self) -> usize;
    fn is_continuous_skip(&self) -> bool;
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
                assert!(text.char_len() > 0);
                if let Some(&mut AddChars(ref mut value)) = self.last_mut() {
                    value.push_doc_string(text);
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
                AddChars(ref chars) => chars.char_len(),
                AddGroup(..) | AddWithGroup(..) => 1,
            };
        }
        ret
    }

    fn is_continuous_skip(&self) -> bool {
        if self.len() > 1 {
            // Will never be a continuous skip
            false
        } else if self.is_empty() {
            // is []
            true
        } else if let AddSkip(_) = self[0] {
            // is [DelSkip(n)]
            true
        } else {
            // is [DelSomething(n)]
            false
        }
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

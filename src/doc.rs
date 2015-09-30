use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Atom {
	Char(char),
	Enter(HashMap<String, String>),
	Leave,
}

pub type Attrs = HashMap<String, String>;

pub use self::Atom::*;

pub type DocSpan = Vec<DocElement>;

#[derive(Clone, Debug, PartialEq)]
pub enum DocElement {
	DocChars(String),
	DocGroup(Attrs, DocSpan),
}

pub use self::DocElement::*;


pub type DelSpan = Vec<DelElement>;

#[derive(Clone, Debug, PartialEq)]
pub enum DelElement {
	DelSkip(usize),
	DelWithGroup(DelSpan),
	DelChars(usize),
	// DelGroup(DelSpan),
	DelGroupAll,
}

pub use self::DelElement::*;


pub type AddSpan = Vec<AddElement>;

#[derive(Clone, Debug, PartialEq)]
pub enum AddElement {
	AddSkip(usize),
	AddWithGroup(AddSpan),
	AddChars(String),
	AddGroup(Attrs, AddSpan),
}

pub use self::AddElement::*;

pub type Op = (DelSpan, AddSpan);

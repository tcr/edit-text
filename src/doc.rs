use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Atom {
	Char(char),
	Enter(HashMap<String, String>),
	Leave,
}

pub type DocSpan = Vec<DocElement>;

#[derive(Clone, Debug, PartialEq)]
pub enum DocElement {
	DocChars(String),
	DocGroup(HashMap<String, String>, DocSpan),
}


pub type DelSpan = Vec<DelElement>;

#[derive(Clone, Debug, PartialEq)]
pub enum DelElement {
	DelSkip(usize),
	DelWithGroup(DelSpan),
	DelChars(usize),
	DelGroup,
}


pub type AddSpan = Vec<AddElement>;

#[derive(Clone, Debug, PartialEq)]
pub enum AddElement {
	AddSkip(usize),
	AddWithGroup(AddSpan),
	AddChars(String),
	AddGroup(HashMap<String, String>, AddSpan),
}

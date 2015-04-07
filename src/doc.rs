use std::collections::HashMap;

#[derive(PartialEq)]
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
	DelChars(usize),
	WithChars(usize),
	DelGroup,
	WithGroup(DelSpan),
}

use std::collections::HashMap;

#[derive(PartialEq)]
pub enum Atom {
	Char(char),
	Enter(HashMap<String, String>),
	Leave,
}

pub type DocSpan = Vec<DocElement>;

pub enum DocElement {
	DocChars(String),
	DocGroup(HashMap<String, String>, DocSpan),
}


pub type DelSpan = Vec<DelElement>;

pub enum DelElement {
	DelChars(i32),
	WithChars(i32),
	DelGroup(DelSpan),
	WithGroup(DelSpan),
}

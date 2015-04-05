use std::collections::HashMap;

pub type DSpan = Vec<DocElement>;

pub enum DocElement {
	DString(String),
	DGroup(HashMap<String, String>, DSpan),
}

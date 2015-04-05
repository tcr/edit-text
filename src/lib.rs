mod doc;

use std::collections::HashMap;
use doc::{DSpan, DocElement};
use doc::DocElement::*;

pub fn debug_span(val:&DSpan) {
	for i in val {
		debug_elem(i);
	}
}

pub fn debug_elem(val:&DocElement) {
	match val {
		&DString(ref value) => {
			println!("str({})", value);
		},
		&DGroup(ref attrs, ref span) => {
			println!("attrs({})", attrs.capacity());
			println!("span({})", span.capacity());
		},
	}
}

pub fn simple() -> DSpan {
	vec![
		DString("Hello world!".to_string()),
		DGroup(HashMap::new(), vec![]),
	]
}

#[test]
fn try_this() {
	let source:DSpan = vec![
		DString("Hello world!".to_string()),
		DGroup(HashMap::new(), vec![]),
	];

	debug_span(&source);
}

#![feature(collections)]

mod doc;

use std::collections::HashMap;
use doc::{DocSpan, DocElement, Atom};
use doc::DocElement::*;

pub fn debug_span(val:&DocSpan) {
	for i in val {
		debug_elem(i);
	}
}

pub fn debug_elem(val:&DocElement) {
	match val {
		&DocChars(ref value) => {
			println!("str({})", value);
		},
		&DocGroup(ref attrs, ref span) => {
			println!("attrs({})", attrs.capacity());
			println!("span({})", span.capacity());
		},
	}
}

pub fn simple() -> DocSpan {
	vec![
		DocChars("Hello world!".to_string()),
		DocGroup(HashMap::new(), vec![]),
	]
}

pub fn iterate(span:&DocSpan) -> Vec<Atom> {
	let mut atoms = vec![];
	for elem in span {
		match elem {
			&DocChars(ref value) => {
				for c in value.chars() {
					atoms.push(Atom::Char(c));
				}
			},
			&DocGroup(ref attrs, ref span) => {
				atoms.push(Atom::Enter(attrs.clone()));
				atoms.append(&mut iterate(span));
				atoms.push(Atom::Leave);
			},
		}
	}
	atoms
}

#[test]
fn try_this() {
	let source:DocSpan = vec![
		DocChars("Hello world!".to_string()),
		DocGroup(HashMap::new(), vec![]),
	];

	debug_span(&source);

	let source_atoms = vec![
		Atom::Char('H'),
		Atom::Char('e'),
		Atom::Char('l'),
		Atom::Char('l'),
		Atom::Char('o'),
		Atom::Char(' '),
		Atom::Char('w'),
		Atom::Char('o'),
		Atom::Char('r'),
		Atom::Char('l'),
		Atom::Char('d'),
		Atom::Char('!'),
		Atom::Enter(HashMap::new()),
		Atom::Leave,
	];
	
	if !(iterate(&source) == source_atoms) {
		panic!("iteration doesnt match");
	}
}

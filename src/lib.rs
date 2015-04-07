#![feature(collections)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unused_imports)]

mod doc;

use std::collections::HashMap;
use doc::{DocSpan, DocElement, DelSpan, DelElement, Atom};
use doc::DocElement::*;
use doc::DelElement::*;
use std::borrow::ToOwned;

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
		DocChars("Hello world!".to_owned()),
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

pub fn apply_delete(spanvec:&DocSpan, delvec:&DelSpan) -> DocSpan {
	let mut span = &spanvec[..];
	let mut del = &delvec[..];

	let mut first = span[0].clone();
	span = &span[1..];

	let mut res:DocSpan = Vec::with_capacity(span.len());
	
	let mut d = del[0].clone();
	del = &del[1..];

	loop {
		let mut nextdel = true;
		let mut nextfirst = true;

		match d.clone() {
			DelChars(count) => {
				match first.clone() {
					DocChars(ref value) => {
						let len = value.chars().count();
						if len > count {
							first = DocChars(value[count..len].to_owned());
							nextfirst = false;
						} else if len < count {
							panic!("attempted deletion of too much");
						}
					},
					_ => {
						panic!("Invalid DelChars");
					}
				}
			},
			WithChars(count) => {
				match first.clone() {
					DocChars(ref value) => {
						let len = value.chars().count();
						if len < count {
							d = WithChars(count - len);
							nextdel = false;
						} else if len > count {
							let mut place_chars = |value:String| {
								if res.len() > 0 {
									let idx = res.len() - 1;
									if let &mut DocChars(ref mut prefix) = &mut res[idx] {
										prefix.push_str(&value[..]);
										return;
									}
								}
								res.push(DocChars(value));
							};

							place_chars(value[0..count].to_owned());
							first = DocChars(value[count..len].to_owned());
							nextfirst = false;
						}
					},
					_ => {
						panic!("Invalid WithChars");
					}
				}
			},
			DelGroup => {
				match first.clone() {
					DocGroup(..) => {},
					_ => {
						panic!("Invalid DelGroup");
					}
				}
			},
			WithGroup(ref delspan) => {
				match first.clone() {
					DocGroup(..) => {
						res.push(first.clone());
					},
					_ => {
						panic!("Invalid DelGroup");
					}
				}
			},
		}

		if nextdel {
			if del.len() == 0 {
				if !nextfirst {
					res.push(first)
				}
				res.push_all(span);
				break;
			}

			d = del[0].clone();
			del = &del[1..];
		}

		if nextfirst {
			if span.len() == 0 {
				panic!("exhausted document");
			}

			first = span[0].clone();
			span = &span[1..];
		}
	}

	res
}

#[test]
fn try_this() {
	let source:DocSpan = vec![
		DocChars("Hello world!".to_owned()),
		DocGroup(HashMap::new(), vec![]),
	];

	debug_span(&source);
	
	assert_eq!(iterate(&vec![
		DocChars("Hello world!".to_owned()),
		DocGroup(HashMap::new(), vec![]),
	]), vec![
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
	]);

	assert_eq!(apply_delete(&vec![
		DocChars("Hello world!".to_owned()),
		DocGroup(HashMap::new(), vec![]),
	], &vec![
		DelChars(3),
		WithChars(2),
		DelChars(1),
		WithChars(1),
		DelChars(5),
		DelGroup,
	]), vec![
		DocChars("low".to_owned()),
	]);

	assert_eq!(apply_delete(&vec![
		DocChars("Hello World!".to_owned()),
	], &vec![
		DelChars(6),
	]), vec![
		DocChars("World!".to_owned()),
	]);
}

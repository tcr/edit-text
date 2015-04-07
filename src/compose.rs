use std::collections::HashMap;
use doc::{DocSpan, DocElement, DelSpan, DelElement, AddSpan, AddElement, Atom, Op};
use doc::DocElement::*;
use doc::DelElement::*;
use doc::AddElement::*;
use std::borrow::ToOwned;
use std::cmp;

struct DelSlice<'a> {
	head:Option<DelElement>,
	rest:&'a [DelElement],
}

impl<'a> DelSlice<'a> {
	fn new(span:&'a DelSpan) -> DelSlice {
		DelSlice {
			head: Some(span[0].clone()),
			rest: &span[1..],
		}
	}

	fn next(&mut self) -> DelElement  {
		let res = self.head.clone().unwrap();
		if self.rest.len() == 0 {
			self.head = None;
			self.rest = &[];
		} else {
			self.head = Some(self.rest[0].clone());
			self.rest = &self.rest[1..];
		}
		res
	}

	fn get_head(&self) -> DelElement {
		self.head.clone().unwrap()
	}

	fn is_done(&self) -> bool {
		self.head.is_none()
	}
}

struct AddSlice<'a> {
	head:Option<AddElement>,
	rest:&'a [AddElement],
}

impl<'a> AddSlice<'a> {
	fn new(span:&'a AddSpan) -> AddSlice {
		AddSlice {
			head: Some(span[0].clone()),
			rest: &span[1..],
		}
	}

	fn next(&mut self) -> AddElement  {
		let res = self.head.clone().unwrap();
		if self.rest.len() == 0 {
			self.head = None;
			self.rest = &[];
		} else {
			self.head = Some(self.rest[0].clone());
			self.rest = &self.rest[1..];
		}
		res
	}

	fn get_head(&self) -> AddElement {
		self.head.clone().unwrap()
	}

	fn is_done(&self) -> bool {
		self.head.is_none()
	}
}

fn del_place_chars(res:&mut DelSpan, count:usize) {
	if res.len() > 0 {
		let idx = res.len() - 1;
		if let &mut DelChars(ref mut prefix) = &mut res[idx] {
			*prefix += count;
			return;
		}
	}
	res.push(DelChars(count));
}

fn del_place_any(res:&mut DelSpan, value:&DelElement) {
	match value {
		&DelChars(count) => {
			del_place_chars(res, count);
		},
		_ => {
			res.push(value.clone());
		}
	}
}

fn add_place_chars(res:&mut AddSpan, value:String) {
	if res.len() > 0 {
		let idx = res.len() - 1;
		if let &mut AddChars(ref mut prefix) = &mut res[idx] {
			prefix.push_str(&value[..]);
			return;
		}
	}
	res.push(AddChars(value));
}

fn add_place_any(res:&mut AddSpan, value:&AddElement) {
	match value {
		&AddChars(ref value) => {
			add_place_chars(res, value.clone());
		},
		_ => {
			res.push(value.clone());
		}
	}
}

fn compose_del_del(avec:&DelSpan, bvec:&DelSpan) -> DelSpan {
	let mut res = Vec::with_capacity(avec.len() + bvec.len());

	let mut a = DelSlice::new(avec);
	let mut b = DelSlice::new(bvec);

	while !a.is_done() {
		match a.get_head() {
			DelSkip(acount) => {
				match b.head.clone() {
					Some(DelSkip(bcount)) => {
						res.push(DelSkip(cmp::min(acount, bcount)));
						if acount > bcount {
							a.head = Some(DelSkip(acount - bcount));
							b.next();
						} else if acount < bcount {
							b.head = Some(DelSkip(bcount - acount));
							a.next();
						} else {
							a.next();
							b.next();
						}
					},
					Some(DelWithGroup(ref span)) => {
						if acount > 1 {
							a.head = Some(DelSkip(acount - 1));
						} else {
							a.next();
						}
						res.push(b.next());
					},
					Some(DelChars(bcount)) => {
						del_place_any(&mut res, &DelChars(cmp::min(acount, bcount)));
						if acount > bcount {
							a.head = Some(DelSkip(acount - bcount));
							b.next();
						} else if acount < bcount {
							b.head = Some(DelChars(bcount - acount));
							a.next();
						} else {
							a.next();
							b.next();
						}
					},
					Some(DelGroup) => {
						if acount > 1 {
							a.head = Some(DelSkip(acount - 1));
						} else {
							a.next();
						}
						res.push(b.next());
					},
					None => {
						res.push(a.next());
					}
				}
			},
			DelWithGroup(ref span) => {
				match b.head.clone() {
					Some(DelSkip(bcount)) => {
						if bcount > 1 {
							b.head = Some(DelSkip(bcount - 1));
						} else {
							b.next();
						}
						res.push(a.next());
					},
					Some(DelWithGroup(ref bspan)) => {
						res.push(DelWithGroup(compose_del_del(span, bspan)));
						a.next();
						b.next();
					},
					Some(DelChars(bcount)) => {
						panic!("DelWithGroup vs DelChars is bad");
					},
					Some(DelGroup) => {
						a.next();
						res.push(b.next());
					},
					None => {
						res.push(a.next());
					}
				}
			},
			DelChars(count) => {
				del_place_any(&mut res, &DelChars(count));
				a.next();
			},
			DelGroup => {
				res.push(DelGroup);
				a.next();
			},
		}
	}

	if !b.is_done() {
		del_place_any(&mut res, &b.get_head());
		res.push_all(b.rest);
	}

	res
}

fn compose_add_add(avec:&AddSpan, bvec:&AddSpan) -> AddSpan {
	let mut res = Vec::with_capacity(avec.len() + bvec.len());

	let mut a = AddSlice::new(avec);
	let mut b = AddSlice::new(bvec);

	while !b.is_done() {
		match b.get_head() {
			AddChars(value) => {
				add_place_any(&mut res, &b.next());
			},
			AddSkip(bcount) => {
				match a.get_head() {
					AddChars(value) => {
						let len = value.chars().count();
						if bcount < len {
							add_place_any(&mut res, &AddChars(value[..bcount].to_owned()));
							a.head = Some(AddChars(value[bcount..].to_owned()));
							b.next();
						} else if bcount > len {
							a.next();
							b.head = Some(AddSkip(bcount - len));
						} else {
							add_place_any(&mut res, &a.get_head());
							a.next();
							b.next();
						}
					},
					AddSkip(acount) => {
						res.push(AddSkip(cmp::min(acount, bcount)));
						if acount > bcount {
							a.head = Some(AddSkip(acount - bcount));
							b.next();
						} else if acount < bcount {
							b.head = Some(AddSkip(bcount - acount));
							a.next();
						} else {
							a.next();
							b.next();
						}
					},
					_ => {
						panic!("Unimplemented");
					}
				}
			},
			_ => {
				panic!("Unimplemented");
			},
		}
	}

	if !a.is_done() {
		add_place_any(&mut res, &a.get_head());
		res.push_all(a.rest);
	}

	res
}


#[test]
fn try_this() {
	assert_eq!(compose_del_del(&vec![
		DelSkip(6),
		DelChars(6),
	], &vec![
		DelChars(3),
	]), vec![
		DelChars(3),
		DelSkip(3),
		DelChars(6),
	]);

	assert_eq!(compose_del_del(&vec![
		DelSkip(6),
		DelChars(6),
	], &vec![
		DelChars(6),
	]), vec![
		DelChars(12),
	]);

	assert_eq!(compose_del_del(&vec![
		DelWithGroup(vec![
			DelChars(6),
		]),
	], &vec![
		DelGroup,
	]), vec![
		DelGroup,
	]);

	assert_eq!(compose_del_del(&vec![
		DelWithGroup(vec![
			DelChars(6),
		]),
	], &vec![
		DelWithGroup(vec![
			DelChars(6),
		]),
	]), vec![
		DelWithGroup(vec![
			DelChars(12),
		]),
	]);

	assert_eq!(compose_add_add(&vec![
		AddChars("World!".to_owned()),
	], &vec![
		AddChars("Hello ".to_owned()),
	]), vec![
		AddChars("Hello World!".to_owned()),
	]);

	assert_eq!(compose_add_add(&vec![
		AddChars("edef".to_owned()),
	], &vec![
		AddChars("d".to_owned()),
		AddSkip(1),
		AddChars("a".to_owned()),
		AddSkip(1),
		AddChars("b".to_owned()),
		AddSkip(1),
		AddChars("e".to_owned()),
		AddSkip(1),
	]), vec![
		AddChars("deadbeef".to_owned()),
	]);

	assert_eq!(compose_add_add(&vec![
		AddSkip(10),
		AddChars("h".to_owned()),
	], &vec![
		AddSkip(11),
		AddChars("i".to_owned()),
	]), vec![
		AddSkip(10),
		AddChars("hi".to_owned()),
	]);
}

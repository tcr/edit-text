#![allow(unused_mut)]

use std::collections::HashMap;
use std::borrow::ToOwned;
use std::cmp;

use doc::*;
use stepper::*;
use string;

#[derive(PartialEq)]
enum TrackType {
	TextBlock
}

struct Track {
	tag_a: Option<String>,
	tag_real: String,
	tag_b: Option<String>,
	is_original_a: bool,
	is_original_b: bool,
}

fn get_type(attrs:&Attrs) -> TrackType {
	TrackType::TextBlock	
}

struct Transform {
	tracks: Vec<Track>
}

impl Transform {
    // fn use() {
    	
    //   var a = iterA.tag;
    //   iterA.apply(insrA);
    //   iterA.apply(insrB);
    //   delrA.enter();
    //   delrB.enter();
    //   iterA.next();
    //   iterB.next();
    //   tran.push(a, a, a, true, true);
    // }
}

fn transform_insertions(avec:&AddSpan, bvec:&AddSpan) -> AddSpan {
	// let mut res = Vec::with_capacity(avec.len() + bvec.len());

	let mut t = Transform {
		tracks: vec![]
	};

	let mut a = AddSlice::new(avec);
	let mut b = AddSlice::new(bvec);

	loop {
		match (a.head.clone(), b.head.clone()) {
			(Some(AddGroup(ref aattrs, _)), Some(AddGroup(ref battrs, _))) => {
				let atype = get_type(aattrs);
				let btype = get_type(battrs);

				if atype == btype {
					println!("My");
				}

				// a.enter();
				// b.enter();
			},
			(Some(AddSkip(..)), Some(AddSkip(..))) => {
				// a.next();
				// b.next();
			},
			(None, None) => {
				// a.exit();
				// b.exit();
			},
			_ => {
				panic!("No idea");
			},
		}

		break;
	}

	vec![
		AddGroup(container! { (string("tag"), string("p")) }, vec![AddSkip(4)]),
		AddGroup(container! { (string("tag"), string("p")) }, vec![AddSkip(2)])
	]
}

#[test]
fn test_transform_goose() {
	let a = vec![
		AddGroup(container! { (string("tag"), string("p")) }, vec![AddSkip(6)])
	];
	let b = vec![
		AddGroup(container! { (string("tag"), string("p")) }, vec![AddSkip(4)])
	];

	let result = vec![
		AddGroup(container! { (string("tag"), string("p")) }, vec![AddSkip(4)]),
		AddGroup(container! { (string("tag"), string("p")) }, vec![AddSkip(2)])
	];

	transform_insertions(&a, &b);
}

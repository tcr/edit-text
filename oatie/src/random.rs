use super::*;
use super::compose::*;
use super::doc::*;
use rand::{thread_rng, Rng};
use std::collections::HashMap;

/// Given a document span, create a random Add operation that can be applied
/// to the span.
pub fn random_add_span(input: &DocSpan) -> AddSpan {
    let mut rng = thread_rng();

    let mut res: AddSpan = vec![];
    for elem in input {
        match elem {
            &DocChars(ref value) => {
                let mut n = 0;
                let max = value.chars().count();

                // Iterate up to `max` characters.
                while n < max {
                    // Skip a random number of characters.
                    let slice = rng.gen_range(1, max - n + 1);
                    res.place(&AddSkip(slice));
                    n += slice;

                    // Decide whether to add new characters or a new (empty) group.
                    if n < max || rng.gen_weighted_bool(2) {
                        if rng.gen_weighted_bool(2) {
                            let len = rng.gen_range(1, 5);
                            res.place(&AddChars(rng.gen_ascii_chars().take(len).collect()));
                        } else {
                            res.place(&AddGroup(HashMap::new(), vec![]));
                        }
                    }
                }
            },
            &DocGroup(_, ref span) => {
                if rng.gen_weighted_bool(2) {
                    res.place(&AddWithGroup(random_add_span(span)));
                } else {
                    res.place(&AddSkip(1));
                }
            },
        }
    }
    // for _ in 0..rng.gen_range(1, 2) {
    // 	match rng.gen_range(0, 3) {
    // 		0 => { add_place_any(&mut res, &AddSkip(1)); },
    // 		1 => { add_place_any(&mut res, &AddGroup(HashMap::new(), vec![])); },
    // 		2 => { add_place_any(&mut res, &AddChars(rng.gen_ascii_chars().take(3).collect())); },
    // 		_ => {},
    // 	}
    // }
    res
}

pub fn random_del_span(input:&DocSpan) -> DelSpan {
    let mut rng = thread_rng();

    let mut res = vec![];
    for elem in input {
        match elem {
            &DocChars(ref value) => {
                let mut n = 0;
                let max = value.chars().count();
                while n < max {
                    if max - n == 1 {
                        res.push(DelSkip(1));
                        n += 1;
                    } else {
                        let slice = rng.gen_range(2, max - n + 1);
                        if slice == 2 {
                            res.push(DelSkip(1));
                            res.push(DelChars(1));
                            n += 2;
                        } else {
                            let keep = rng.gen_range(1, slice - 1);
                            res.push(DelSkip(keep));
                            res.push(DelChars(slice - keep));
                            n += slice;
                        }
                    }
                }
            },
            &DocGroup(_, ref span) => {
                match rng.gen_range(0, 3) {
                    0 => res.place(&DelWithGroup(random_del_span(span))),
                    1 => res.place(&DelGroupAll),
                    2 => res.place(&DelSkip(1)),
                    _ => {
                        unreachable!();
                    },
                }
            },
        }
    }
    res
}

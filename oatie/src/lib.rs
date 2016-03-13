#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unused_imports)]

#[macro_use] extern crate log;
extern crate env_logger;
extern crate rand;
#[macro_use] extern crate literator;
extern crate rustc_serialize;
extern crate term_painter;

pub mod doc;
pub mod compose;
pub mod transform;
pub mod stepper;

use std::collections::HashMap;
use doc::*;

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

fn place_chars(res:&mut DocSpan, value:String) {
    if res.len() > 0 {
        let idx = res.len() - 1;
        if let &mut DocChars(ref mut prefix) = &mut res[idx] {
            prefix.push_str(&value[..]);
            return;
        }
    }
    res.push(DocChars(value));
}

fn place_any(res:&mut DocSpan, value:&DocElement) {
    match value {
        &DocChars(ref string) => {
            place_chars(res, string.clone());
        },
        _ => {
            res.push(value.clone());
        }
    }
}

fn place_many(res: &mut DocSpan, values: &[DocElement]) {
    if values.len() > 0 {
        place_any(res, &values[0]);
        res.extend_from_slice(&values[1..]);
    }
}

pub fn apply_add_inner(spanvec: &DocSpan, delvec: &AddSpan) -> (DocSpan, DocSpan) {
    let mut span = &spanvec[..];
    let mut del = &delvec[..];

    let mut first = None;
    if span.len() > 0 {
        first = Some(span[0].clone());
        span = &span[1..]
    }

    let mut res: DocSpan = Vec::with_capacity(span.len());

    if del.len() == 0 {
        return (vec![], spanvec.clone().to_vec());
    }

    let mut d = del[0].clone();
    del = &del[1..];

    let mut exhausted = first.is_none();

    trace!("ABOUT TO APPLY ADD {:?} {:?}", first, span);

    loop {
        // Flags for whether we have partially or fully consumed an atom.
        let mut nextdel = true;
        let mut nextfirst = true;

        if exhausted {
            match d {
                AddSkip(..) | AddWithGroup(..) => {
                    panic!("exhausted document on {:?}", d);
                },
                _ => {},
            }
        }

        trace!("next {:?} {:?} {:?}", d, first, exhausted);

        match d.clone() {
            AddSkip(count) => {
                match first.clone().unwrap() {
                    DocChars(ref value) => {
                        let len = value.chars().count();
                        if len < count {
                            place_chars(&mut res, value.to_owned());
                            d = AddSkip(count - len);
                            nextdel = false;
                        } else if len > count {
                            place_chars(&mut res, value[0..count].to_owned());
                            first = Some(DocChars(value[count..len].to_owned()));
                            nextfirst = false;
                        } else {
                            place_chars(&mut res, value.to_owned());
                        }
                    },
                    DocGroup(..) => {
                        res.push(first.clone().unwrap());
                        if count > 1 {
                            d = AddSkip(count - 1);
                            nextdel = false;
                        }
                    },
                }
            },
            AddWithGroup(ref delspan) => {
                match first.clone().unwrap() {
                    DocGroup(ref attrs, ref span) => {
                        res.push(DocGroup(attrs.clone(), apply_add(span, delspan)));
                    },
                    _ => {
                        panic!("Invalid DelGroupAll");
                    }
                }
            },
            AddChars(value) => {
                place_chars(&mut res, value);
                nextfirst = false;
            },
            AddGroup(attrs, innerspan) => {
                let mut subdoc = vec![];
                if !exhausted {
                    subdoc.push(first.clone().unwrap());
                    subdoc.extend_from_slice(span);
                }
                trace!("CALLING INNER {:?} {:?}", subdoc, innerspan);

                let (inner, rest) = apply_add_inner(&subdoc, &innerspan);
                place_any(&mut res, &DocGroup(attrs, inner));

                trace!("REST OF INNER {:?} {:?}", rest, del);

                let inner = apply_add(&rest, &del.to_vec());
                place_many(&mut res, &inner);
                return (res, vec![]);
            },
        }

        if nextdel {
            if del.len() == 0 {
                let mut remaining = vec![];
                if !nextfirst && !first.is_none() && !exhausted {
                    remaining.push(first.clone().unwrap());
                    // place_any(&mut res, &first.clone().unwrap());
                }
                remaining.extend_from_slice(span);
                return (res, remaining);
            }

            d = del[0].clone();
            del = &del[1..];
        }

        if nextfirst {
            if span.len() == 0 {
                exhausted = true;
            } else {
                first = Some(span[0].clone());
                span = &span[1..];
            }
        }
    }
}

pub fn apply_add(spanvec: &DocSpan, delvec: &AddSpan) -> DocSpan {
    let (mut res, remaining) = apply_add_inner(spanvec, delvec);

    // TODO never accept unbalanced components?
    if !remaining.is_empty() {
        place_many(&mut res, &remaining);
        // panic!("Unbalanced apply_add");
    }
    res
}

pub fn apply_delete(spanvec:&DocSpan, delvec:&DelSpan) -> DocSpan {
    let mut span = &spanvec[..];
    let mut del = &delvec[..];

    let mut res:DocSpan = Vec::with_capacity(span.len());

    if del.len() == 0 {
        return span.clone().to_vec();
    }

    let mut first = span[0].clone();
    span = &span[1..];

    let mut d = del[0].clone();
    del = &del[1..];

    loop {
        let mut nextdel = true;
        let mut nextfirst = true;

        match d.clone() {
            DelSkip(count) => {
                match first.clone() {
                    DocChars(ref value) => {
                        let len = value.chars().count();
                        if len < count {
                            place_chars(&mut res, value.clone());
                            d = DelSkip(count - len);
                            nextdel = false;
                        } else if len > count {
                            place_chars(&mut res, value[0..count].to_owned());
                            first = DocChars(value[count..len].to_owned());
                            nextfirst = false;
                        } else {
                            place_chars(&mut res, value.clone());
                        }
                    },
                    DocGroup(..) => {
                        res.push(first.clone());
                        if count > 1 {
                            d = DelSkip(count - 1);
                            nextdel = false;
                        }
                    },
                }
            },
            DelWithGroup(ref delspan) => {
                match first.clone() {
                    DocGroup(ref attrs, ref span) => {
                        res.push(DocGroup(attrs.clone(), apply_delete(span, delspan)));
                    },
                    _ => {
                        panic!("Invalid DelGroupAll");
                    }
                }
            },
            DelGroup(ref delspan) => {
                match first.clone() {
                    DocGroup(ref attrs, ref span) => {
                        res.extend_from_slice(&apply_delete(span, delspan)[..]);
                    },
                    _ => {
                        panic!("Invalid DelGroup");
                    }
                }
            },
            DelChars(count) => {
                match first.clone() {
                    DocChars(ref value) => {
                        let len = value.chars().count();
                        if len > count {
                            first = DocChars(value[count..].to_owned());
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
            DelGroupAll => {
                match first.clone() {
                    DocGroup(..) => {},
                    _ => {
                        panic!("Invalid DelGroupAll");
                    }
                }
            },
        }

        if nextdel {
            if del.len() == 0 {
                if !nextfirst {
                    place_any(&mut res, &first)
                }
                if span.len() > 0 {
                    place_any(&mut res, &span[0]);
                    res.extend_from_slice(&span[1..]);
                }
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

pub fn apply_operation(spanvec:&DocSpan, op:&Op) -> DocSpan {
    let &(ref delvec, ref addvec) = op;
    let postdel = apply_delete(spanvec, delvec);
    apply_add(&postdel, addvec)
}

fn normalize_del (mut del: DelSpan) -> DelSpan {
    // let mut tail = true;
    // del.into_iter().rev().map(|x| {
    //     //TODO
    //     x
    // }).filter(move |x| {
    //     match x {
    //         &DelSkip(_) => {
    //             false
    //         },
    //         _ => true
    //     }
    // }).rev().collect()
    if let Some(&DelSkip(..)) = del.last() {
        del.pop();
    }
    del
}

pub fn normalize (op:Op) -> Op {
    // TODO all
    (normalize_del(op.0), op.1)
}

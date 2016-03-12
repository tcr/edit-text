use std::collections::HashMap;
use doc::*;
use std::borrow::ToOwned;
use std::cmp;

use apply_add;
use apply_delete;
use apply_operation;
use test_start;
use stepper::*;
use normalize;

fn compose_del_del_inner(res:&mut DelSpan, a:&mut DelSlice, b:&mut DelSlice) {
    while !a.is_done() && !b.is_done() {
        match a.get_head() {
            DelSkip(acount) => {
                match b.head.clone() {
                    Some(DelSkip(bcount)) => {
                        res.place(&DelSkip(cmp::min(acount, bcount)));
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
                        res.place(&b.next());
                    },
                    Some(DelGroup(ref span)) => {
                        if acount > 1 {
                            a.head = Some(DelSkip(acount - 1));
                        } else {
                            a.next();
                        }
                        res.place(&b.next());
                    },
                    Some(DelChars(bcount)) => {
                        res.place(&DelChars(cmp::min(acount, bcount)));
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
                    Some(DelGroupAll) => {
                        if acount > 1 {
                            a.head = Some(DelSkip(acount - 1));
                        } else {
                            a.next();
                        }
                        res.place(&b.next());
                    },
                    None => {
                        res.place(&a.next());
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
                        res.place(&a.next());
                    },
                    Some(DelWithGroup(ref bspan)) => {
                        res.place(&DelWithGroup(compose_del_del(span, bspan)));
                        a.next();
                        b.next();
                    },
                    Some(DelGroup(ref bspan)) => {
                        res.place(&DelGroup(compose_del_del(span, bspan)));
                        a.next();
                        b.next();
                    },
                    Some(DelChars(bcount)) => {
                        panic!("DelWithGroup vs DelChars is bad");
                    },
                    Some(DelGroupAll) => {
                        a.next();
                        res.place(&b.next());
                    },
                    None => {
                        res.place(&a.next());
                    }
                }
            },
            DelGroup(ref span) => {
                let mut c = DelSlice::new(span);
				let mut inner: DelSpan = vec![];
                compose_del_del_inner(&mut inner, &mut c, b);
                res.place(&DelGroup(inner));
                if !c.is_done() {
					res.place(&c.head.unwrap());
                    res.place_all(c.rest);
                }
                a.next();
            },
            DelChars(count) => {
                res.place(&DelChars(count));
                a.next();
            },
            DelGroupAll => {
                res.place(&DelGroupAll);
                a.next();
            },
        }
    }
}

pub fn compose_del_del(avec: &DelSpan, bvec: &DelSpan) -> DelSpan {
    let mut res = Vec::with_capacity(avec.len() + bvec.len());

    let mut a = DelSlice::new(avec);
    let mut b = DelSlice::new(bvec);

    compose_del_del_inner(&mut res, &mut a, &mut b);

    if !a.is_done() {
        res.place(&a.get_head());
        res.place_all(a.rest);
    }

    if !b.is_done() {
        res.place(&b.get_head());
        res.place_all(b.rest);
    }

    res
}

fn compose_add_add_inner(res:&mut AddSpan, a:&mut AddSlice, b:&mut AddSlice) {
    while !b.is_done() && !a.is_done() {
        match b.get_head() {
            AddChars(value) => {
                res.place(&b.next());
            },
            AddSkip(bcount) => {
                match a.get_head() {
                    AddChars(value) => {
                        let len = value.chars().count();
                        if bcount < len {
                            res.place(&AddChars(value[..bcount].to_owned()));
                            a.head = Some(AddChars(value[bcount..].to_owned()));
                            b.next();
                        } else if bcount > len {
                            res.place(&a.next());
                            b.head = Some(AddSkip(bcount - len));
                        } else {
                            res.place(&a.get_head());
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
                    AddWithGroup(span) => {
                        res.push(a.next());
                        if bcount == 1 {
                            b.next();
                        } else {
                            b.head = Some(AddSkip(bcount - 1));
                        }
                    },
                    AddGroup(..) => {
                        res.push(a.next());
                        if bcount == 1 {
                            b.next();
                        } else {
                            b.head = Some(AddSkip(bcount - 1));
                        }
                    },
                }
            },
            AddGroup(ref attrs, ref bspan) => {
                let mut c = AddSlice::new(bspan);
                let mut inner = vec![];
                compose_add_add_inner(&mut inner, a, &mut c);
                if !c.is_done() {
                    inner.place(&c.get_head());
                    inner.place_all(c.rest);
                }
                res.push(AddGroup(attrs.clone(), inner));
                b.next();
            },
            AddWithGroup(ref bspan) => {
                match a.get_head() {
                    AddChars(value) => {
                        panic!("Cannot compose AddWithGroup with AddChars");
                    }
                    AddSkip(acount) => {
                        if acount == 1 {
                            a.next();
                        } else {
                            a.head = Some(AddSkip(acount - 1));
                        }
                        res.push(b.next());
                    },
                    AddWithGroup(ref aspan) => {
                        res.push(AddWithGroup(compose_add_add(aspan, bspan)));
                        a.next();
                        b.next();
                    },
                    AddGroup(ref attrs, ref aspan) => {
                        res.push(AddGroup(attrs.clone(), compose_add_add(aspan, bspan)));
                        a.next();
                        b.next();
                    },
                }
            },
        }
    }
}

fn compose_add_add(avec:&AddSpan, bvec:&AddSpan) -> AddSpan {
    let mut res = Vec::with_capacity(avec.len() + bvec.len());

    let mut a = AddSlice::new(avec);
    let mut b = AddSlice::new(bvec);

    compose_add_add_inner(&mut res, &mut a, &mut b);

    if !b.is_done() {
        res.place(&b.get_head());
        res.place_all(b.rest);
    }

    if !a.is_done() {
        res.place(&a.get_head());
        res.place_all(a.rest);
    }

    res
}

pub fn compose_add_del(avec: &AddSpan, bvec: &DelSpan) -> Op {
    let mut delres: DelSpan = Vec::with_capacity(avec.len() + bvec.len());
    let mut addres: AddSpan = Vec::with_capacity(avec.len() + bvec.len());

    let mut a = AddSlice::new(avec);
    let mut b = DelSlice::new(bvec);

    while !b.is_done() && !a.is_done() {
        match b.get_head() {
            DelChars(bcount) => {
                match a.get_head() {
                    AddChars(avalue) => {
                        let alen = avalue.chars().count();
                        if bcount < alen {
                            a.head = Some(AddChars(avalue[bcount..].to_owned()));
                            b.next();
                        } else if bcount > alen {
                            a.next();
                            b.head = Some(DelChars(bcount - alen));
                        } else {
                            a.next();
                            b.next();
                        }
                    },
                    AddSkip(acount) => {
                        if bcount < acount {
                            a.head = Some(AddSkip(acount - bcount));
                            delres.place(&b.next());
                        } else if bcount > acount {
                            a.next();
                            delres.place(&DelChars(acount));
                            b.head = Some(DelChars(bcount - acount));
                        } else {
                            a.next();
                            delres.place(&b.next());
                        }
                    },
                    _ => {
                        panic!("Unimplemented or Unexpected");
                    },
                }
            },
            DelSkip(bcount) => {
                match a.get_head() {
                    AddChars(avalue) => {
                        let alen = avalue.chars().count();
                        if bcount < alen {
                            addres.place(&AddChars(avalue[..bcount].to_owned()));
                            a.head = Some(AddChars(avalue[bcount..].to_owned()));
                            b.next();
                        } else if bcount > alen {
                            addres.place(&a.next());
                            b.head = Some(DelSkip(bcount - alen));
                        } else {
                            addres.place(&a.get_head());
                            a.next();
                            b.next();
                        }
                    },
                    AddSkip(acount) => {
                        addres.place(&AddSkip(cmp::min(acount, bcount)));
                        delres.place(&DelSkip(cmp::min(acount, bcount)));
                        if acount > bcount {
                            a.head = Some(AddSkip(acount - bcount));
                            b.next();
                        } else if acount < bcount {
                            a.next();
                            b.head = Some(DelSkip(bcount - acount));
                        } else {
                            a.next();
                            b.next();
                        }
                    },
                    AddWithGroup(..) => {
                        addres.place(&a.next());
                        delres.place(&DelSkip(1));
                        if bcount == 1 {
                            b.next();
                        } else {
                            b.head = Some(DelSkip(bcount - 1));
                        }
                    },
                    AddGroup(..) => {
                        addres.place(&a.next());
                        if bcount == 1 {
                            b.next();
                        } else {
                            b.head = Some(DelSkip(bcount - 1));
                        }
                    },
                }
            },
            DelWithGroup(span) => {
                match a.get_head() {
                    AddChars(avalue) => {
                        panic!("DelWithGroup by AddChars is ILLEGAL");
                    },
                    AddSkip(acount) => {
                        delres.place(&b.next());
                        addres.place(&AddSkip(1));
                        if acount > 1 {
                            a.head = Some(AddSkip(acount - 1));
                        } else {
                            a.next();
                        }
                    },
                    AddWithGroup(insspan) => {
                        a.next();
                        b.next();

                        let (del, ins) = compose_add_del(&insspan, &span);
                        delres.place(&DelWithGroup(del));
                        addres.place(&AddWithGroup(ins));
                    },
                    AddGroup(attr, insspan) => {
                        a.next();
                        b.next();

                        let (_, ins) = compose_add_del(&insspan, &span);
                        addres.place(&AddGroup(attr, ins));
                    },
                }
            },
            DelGroup(span) => {
                match a.get_head() {
                    AddChars(avalue) => {
                        panic!("DelGroup by AddChars is ILLEGAL");
                    },
                    AddSkip(acount) => {
                        delres.place(&b.next());
                        addres.place(&AddSkip(1));
                        if acount > 1 {
                            a.head = Some(AddSkip(acount - 1));
                        } else {
                            a.next();
                        }
                    },
                    AddWithGroup(insspan) => {
                        a.next();
                        b.next();

                        let (del, ins) = compose_add_del(&insspan, &span);
                        delres.place(&DelGroup(del));
                        addres.place_all(&ins[..]);
                    },
                    AddGroup(attr, insspan) => {
                        a.next();
                        b.next();

                        let (del, ins) = compose_add_del(&insspan, &span);
                        delres.place_all(&del[..]);
                        addres.place_all(&ins[..]);
                    },
                }
            },
            DelGroupAll => {
                match a.get_head() {
                    AddChars(avalue) => {
                        panic!("DelGroupAll by AddChars is ILLEGAL");
                    },
                    AddSkip(acount) => {
                        delres.place(&b.next());
                        if acount > 1 {
                            a.head = Some(AddSkip(acount - 1));
                        } else {
                            a.next();
                        }
                    },
                    AddWithGroup(insspan) => {
                        a.next();
                        delres.place(&b.next());
                    },
                    AddGroup(attr, insspan) => {
                        a.next();
                        b.next();
                    },
                }
            },
        }
    }

    if !b.is_done() {
        delres.place(&b.get_head());
        delres.place_all(b.rest);
    }

    if !a.is_done() {
		delres.place(&DelSkip(1 + a.rest.len()));
        addres.place(&a.get_head());
        addres.place_all(a.rest);
    }

    (delres, addres)
}



#[test]
fn test_compose_del_del() {
    test_start();

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
        DelGroupAll,
    ]), vec![
        DelGroupAll,
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

    assert_eq!(compose_del_del(&vec![
        DelSkip(2), DelChars(6), DelSkip(1), DelChars(2), DelSkip(1)
    ], &vec![
        DelSkip(1), DelChars(1), DelSkip(1)
    ]), vec![
        DelSkip(1), DelChars(7), DelSkip(1), DelChars(2), DelSkip(1)
    ]);
}

#[test]
fn test_compose_add_add() {
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

    assert_eq!(compose_add_add(&vec![
        AddSkip(5), AddChars("yEH".to_owned()), AddSkip(1), AddChars("GlG5".to_owned()), AddSkip(4), AddChars("nnG".to_owned()), AddSkip(1), AddChars("ra8c".to_owned()), AddSkip(1)
    ], &vec![
        AddSkip(10), AddChars("Eh".to_owned()), AddSkip(16),
    ]), vec![
        AddSkip(5), AddChars("yEH".to_owned()), AddSkip(1), AddChars("GEhlG5".to_owned()), AddSkip(4), AddChars("nnG".to_owned()), AddSkip(1), AddChars("ra8c".to_owned()), AddSkip(1)
    ]);
}

#[test]
fn test_compose_add_del() {
    test_start();

    assert_eq!(compose_add_del(&vec![
        AddSkip(4), AddChars("0O".to_owned()), AddSkip(5), AddChars("mnc".to_owned()), AddSkip(3), AddChars("gbL".to_owned()),
    ], &vec![
        DelSkip(1), DelChars(1), DelSkip(3), DelChars(2), DelSkip(2), DelChars(9), DelSkip(1), DelChars(1),
    ]), (vec![
        DelSkip(1), DelChars(1), DelSkip(2), DelChars(1), DelSkip(2), DelChars(5),
    ], vec![
        AddSkip(3), AddChars("0".to_owned()), AddSkip(2), AddChars("b".to_owned()),
    ]));
}

use rand::{thread_rng, Rng};

/// Given a document span, create a random Add operation that can be applied
/// to the span.
fn random_add_span(input: &DocSpan) -> AddSpan {
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
            &DocGroup(ref attrs, ref span) => {
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


fn random_del_span(input:&DocSpan) -> DelSpan {
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
            &DocGroup(ref attr, ref span) => {
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

#[test]
fn monkey_add_add() {
    test_start();

    for i in 0..1000 {
        let start = vec![
            DocChars("Hello world!".to_owned()),
        ];

        trace!("start {:?}", start);

        let a = random_add_span(&start);
        trace!("Random A: {:?}", a);

        let middle = apply_add(&start, &a);
        let b = random_add_span(&middle);
        trace!("Random B: {:?}", a);
        let end = apply_add(&middle, &b);

        let composed = compose_add_add(&a, &b);
        trace!("Composed: {:?}", composed);
        let otherend = apply_add(&start, &composed);

        trace!("middle {:?}", middle);
        trace!("b {:?}", b);
        trace!("end {:?}", end);

        trace!("composed {:?}", composed);
        trace!("otherend {:?}", otherend);

        assert_eq!(end, otherend);

        trace!("-----");
    }
}

#[test]
fn monkey_del_del() {
    test_start();

    for i in 0..1000 {
        let start = vec![
            DocChars("Hello world!".to_owned()),
        ];

        trace!("start {:?}", start);

        let a = random_del_span(&start);
        trace!("a {:?}", a);

        let middle = apply_delete(&start, &a);
        let b = random_del_span(&middle);
        let end = apply_delete(&middle, &b);

        let composed = compose_del_del(&a, &b);
        let otherend = apply_delete(&start, &composed);

        trace!("middle {:?}", middle);
        trace!("b {:?}", b);
        trace!("end {:?}", end);

        trace!("composed {:?}", composed);
        trace!("otherend {:?}", otherend);

        assert_eq!(end, otherend);
    }
}

#[test]
fn monkey_add_del() {
    test_start();

    for i in 0..1000 {
        let start = vec![
            DocChars("Hello world!".to_owned()),
        ];

        trace!("start {:?}", start);

        let a = random_add_span(&start);
        trace!("a {:?}", a);

        let middle = apply_add(&start, &a);
        let b = random_del_span(&middle);
        let end = apply_delete(&middle, &b);

        trace!("middle {:?}", middle);
        trace!("b {:?}", b);
        trace!("end {:?}", end);

        let (dela, addb) = compose_add_del(&a, &b);
        trace!("dela {:?}", dela);
        trace!("addb {:?}", addb);

        let middle2 = apply_delete(&start, &dela);
        trace!("middle2 {:?}", middle2);
        let otherend = apply_add(&middle2, &addb);
        trace!("otherend {:?}", otherend);

        assert_eq!(end, otherend);
    }
}

pub fn compose(a:&Op, b:&Op) -> Op {
    let &(ref adel, ref ains) = a;
    let &(ref bdel, ref bins) = b;

    let (mdel, mins) = compose_add_del(ains, bdel);
    (compose_del_del(adel, &mdel), compose_add_add(&mins, bins))
}

fn random_op(input:&DocSpan) -> Op {
    trace!("random_op: input {:?}", input);
    let del = random_del_span(input);
    trace!("random_op: del {:?}", del);
    let middle = apply_delete(input, &del);
    let ins = random_add_span(&middle);
    (del, ins)
}

#[test]
fn test_compose() {
    test_start();

    assert_eq!(normalize(compose(&(vec![], vec![
        AddGroup(container! { ("tag".into(), "p".into()) }, vec![AddSkip(6)])
    ]), &(vec![
        DelGroup(vec![DelSkip(6)])
    ], vec![
        AddGroup(container! { ("tag".into(), "p".into() )}, vec![AddSkip(4)]),
        AddGroup(container! { ("tag".into(), "p".into()) }, vec![AddSkip(2)])
    ]))), (vec![], vec![
        AddGroup(container! { ("tag".into(), "p".into() )}, vec![AddSkip(4)]),
        AddGroup(container! { ("tag".into(), "p".into()) }, vec![AddSkip(2)])
    ]));
}


#[test]
fn monkey_compose() {
    test_start();

    let mut start = vec![
        DocChars("Hello world!".to_owned()),
    ];

    for i in 0..100 {
        trace!("start {:?}", start);

        let a = random_op(&start);
        trace!("a {:?}", a);

        let middle = apply_operation(&start, &a);
        trace!("middle {:?}", middle);

        let b = random_op(&middle);
        trace!("b {:?}", b);

        let end = apply_operation(&middle, &b);
        trace!("end {:?}", end);

        let composed = compose(&a, &b);
        trace!("composed {:?}", composed);

        let otherend = apply_operation(&start, &composed);
        trace!("otherend {:?}", otherend);

        assert_eq!(end, otherend);

        start = end;
    }
}

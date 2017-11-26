#[macro_use] extern crate oatie;
extern crate term_painter;
#[macro_use] extern crate log;
extern crate env_logger;
extern crate rand;

use oatie::*;
use oatie::doc::*;
use oatie::compose::*;
use rand::{thread_rng, Rng};
use std::collections::HashMap;

fn test_start() {
    let _ = env_logger::init();
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


    assert_eq!(compose_del_del(
        &del_span![DelGroup([DelSkip(11)])],
        &del_span![DelSkip(6), DelChars(1)],
    ), del_span![DelGroup([DelSkip(6), DelChars(1), DelSkip(4)])]);
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

// #[test]
// fn test_multiple_compose_1() {
//     println!("yeah");

//     let doc = doc_span![DocGroup({"tag": "ul"}, [DocGroup({"tag": "li"}, [DocGroup({"tag": "h1"}, [DocChars("Hello! "), DocGroup({"tag": "b"}, [DocChars("what\'s")]), DocChars(" up?")]), DocGroup({"tag": "p"}, [DocChars("World!")])])])];

//     let mut ops: Vec<Op> = vec![
//         op_span!(
//             [DelWithGroup([DelWithGroup([DelWithGroup([DelSkip(6), DelChars(1)])])])],
//             [],
//         ),
//         op_span!(
//             [DelWithGroup([DelWithGroup([DelGroup([DelSkip(11)])])])],
//             [AddWithGroup([AddWithGroup([AddGroup({"tag": "h1"}, [AddSkip(6)]), AddGroup({"tag": "p"}, [AddSkip(5)])])])],
//         ),
//         op_span!(
//             [],
//             [AddWithGroup([AddWithGroup([AddSkip(1), AddWithGroup([AddWithGroup([AddSkip(1), AddChars("W")])])])])],
//         ),
//         op_span!(
//             [DelWithGroup([DelWithGroup([DelSkip(1), DelWithGroup([DelWithGroup([DelChars(1)])])])])],
//             [],
//         ),
//     ];

//     let mut op = op_span!([], []);
//     for i in ops.into_iter() {

//         println!("compose: op_span!(");
//         println!("  {:?},", i.0);
//         println!("  {:?},", i.1);
//         println!(")");

//         op = compose(&op, &i);

//         println!("applying: op_span!(");
//         println!("  {:?},", op.0);
//         println!("  {:?},", op.1);
//         println!(")");
//         let out = apply_operation(&doc, &op);

//         println!("doc: {:?}", out);
//     }

// // CMP add [DelWithGroup([DelWithGroup([DelGroup([DelWithGroup([DelChars(1), DelSkip(1)]), DelSkip(5), DelChars(1), DelSkip(5)])])])]
// //     del [AddWithGroup([AddWithGroup([AddGroup({"tag": "h1"}, [AddSkip(6)]), AddGroup({"tag": "p"}, [AddWithGroup([AddChars("W")]), AddSkip(4)])])])]
// // start obj [DocGroup({"tag": "ul"}, [DocGroup({"tag": "li"}, [DocGroup({"tag": "h1"}, [DocChars("Hello! "), DocGroup({"tag": "b"}, [DocChars("what\'s")]), DocChars(" up?")]), DocGroup({"tag": "p"}, [DocChars("World!")])])])]
// }

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

#[test]
fn monkey_add_add() {
    test_start();

    for _ in 0..1000 {
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

    for _ in 0..1000 {
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

    for _ in 0..1000 {
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
        AddGroup(map! { "tag" => "p" }, vec![AddSkip(6)])
    ]), &(vec![
        DelGroup(vec![DelSkip(6)])
    ], vec![
        AddGroup(map! { "tag" => "p" }, vec![AddSkip(4)]),
        AddGroup(map! { "tag" => "p" }, vec![AddSkip(2)])
    ]))), (vec![], vec![
        AddGroup(map! { "tag" => "p" }, vec![AddSkip(4)]),
        AddGroup(map! { "tag" => "p" }, vec![AddSkip(2)])
    ]));
}


#[test]
fn monkey_compose() {
    test_start();

    let mut start = vec![
        DocChars("Hello world!".to_owned()),
    ];

    for _ in 0..100 {
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

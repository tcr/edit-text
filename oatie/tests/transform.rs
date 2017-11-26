#![allow(unused_mut)]

#[macro_use] extern crate env_logger;
#[macro_use] extern crate oatie;
extern crate log;
extern crate term_painter;

use oatie::compose;
use oatie::doc::*;
use oatie::normalize;
use oatie::transform::*;
use oatie::apply_operation;

fn test_start() {
    let _ = env_logger::init();
}

#[test]
fn test_transform_anthem() {
    test_start();

    let a = vec![
        AddGroup(map! { "tag" => "p" }, vec![
            AddSkip(10)
        ]),
        AddGroup(map! { "tag" => "p" }, vec![
            AddSkip(10)
        ]),
    ];
    let b = vec![
        AddSkip(5),
        AddGroup(map! { "tag" => "b" }, vec![
            AddSkip(10)
        ]),
    ];

    let (a_, b_) = transform_insertions(&a, &b);

    let res = (vec![], vec![
        AddGroup(map! { "tag" => "p" }, vec![
            AddSkip(5),
            AddGroup(map! { "tag" => "b" }, vec![
                AddSkip(5),
            ]),
        ]),
        AddGroup(map! { "tag" => "p" }, vec![
            AddGroup(map! { "tag" => "b" }, vec![
                AddSkip(5),
            ]),
            AddSkip(5),
        ]),
    ]);

    let a_res = normalize(compose::compose(&(vec![], a), &a_));
    let b_res = normalize(compose::compose(&(vec![], b.clone()), &b_));
    assert_eq!(a_res, res.clone());
    assert_eq!(b_res, res.clone());
}

#[test]
fn test_transform_yellow() {
    let a = vec![
        AddGroup(map! { "tag" => "ul" }, vec![
            AddGroup(map! { "tag" => "li" }, vec![
                AddSkip(5)
            ])
        ]),
    ];
    let b = vec![
        AddSkip(3),
        AddGroup(map! { "tag" => "p" }, vec![
            AddSkip(2)
        ]),
        AddGroup(map! { "tag" => "p" }, vec![
            AddSkip(3)
        ]),
    ];

    let (a_, b_) = transform_insertions(&a, &b);

    let res = (vec![], vec![
        AddGroup(map! { "tag" => "ul" }, vec![
            AddGroup(map! { "tag" => "li" }, vec![
                AddSkip(3),
                AddGroup(map! { "tag" => "p" }, vec![
                    AddSkip(2)
                ]),
            ])
        ]),
        AddGroup(map! { "tag" => "p" }, vec![
            AddSkip(3)
        ]),
    ]);

    let a_res = normalize(compose::compose(&(vec![], a), &a_));
    let b_res = normalize(compose::compose(&(vec![], b.clone()), &b_));
    assert_eq!(a_res, res.clone());
    assert_eq!(b_res, res.clone());
}

#[test]
fn test_transform_black() {
    // TODO revert back to things with li's
    let a = vec![
        AddGroup(map! { "tag" => "ul" }, vec![
            AddGroup(map! { "tag" => "li" }, vec![
                AddSkip(5)
            ])
        ]),
    ];
    let b = vec![
        AddSkip(2),
        AddGroup(map! { "tag" => "ul" }, vec![
            AddGroup(map! { "tag" => "li" }, vec![
                AddSkip(2)
            ])
        ]),
    ];

    println!("HERE IS A: {:?}", a);
    println!("HERE IS B: {:?}", b);

    let (a_, b_) = transform_insertions(&a, &b);

    println!("lol");

    let res = (vec![], vec![
        AddGroup(map! { "tag" => "ul" }, vec![
            AddGroup(map! { "tag" => "li" }, vec![
                AddSkip(2)
            ]),
            AddGroup(map! { "tag" => "li" }, vec![
                AddSkip(2)
            ]),
            AddGroup(map! { "tag" => "li" }, vec![
                AddSkip(1)
            ])
        ]),
    ]);

    let a_res = normalize(compose::compose(&(vec![], a), &a_));
    let b_res = normalize(compose::compose(&(vec![], b.clone()), &b_));

    println!("A : {:?}", a_res);
    println!("B : {:?}", b_res);
    println!("r : {:?}", res);

    assert_eq!(a_res, res.clone());
    assert_eq!(b_res, res.clone());
}

#[test]
fn test_transform_ferociously() {
    let a = vec![
        AddGroup(map! { "tag" => "h1" }, vec![
            AddSkip(8)
        ]),
        AddGroup(map! { "tag" => "p" }, vec![
            AddSkip(5)
        ]),
    ];
    let b = vec![
        AddGroup(map! { "tag" => "h3" }, vec![
            AddSkip(8)
        ]),
    ];

    let (a_, b_) = transform_insertions(&a, &b);

    let a_res = normalize(compose::compose(&(vec![], a), &a_));
    let b_res = normalize(compose::compose(&(vec![], b), &b_));
    assert_eq!(a_res, b_res);
}

#[test]
fn test_transform_tony() {
    let a = vec![
        AddWithGroup(vec![
            AddWithGroup(vec![
                AddWithGroup(vec![
                ]),
            ])
        ]),
        AddGroup(map! { "tag" => "p" }, vec![
            AddSkip(5)
        ]),
    ];
    let b = vec![
        AddGroup(map! { "tag" => "h3" }, vec![
            AddSkip(8)
        ]),
    ];

    let (a_, b_) = transform_insertions(&a, &b);

    let a_res = normalize(compose::compose(&(vec![], a), &a_));
    let b_res = normalize(compose::compose(&(vec![], b), &b_));
    assert_eq!(a_res, b_res);
}

#[test]
fn test_transform_drone() {
    let a = vec![
        AddWithGroup(vec![
            AddWithGroup(vec![
                AddWithGroup(vec![
                    AddSkip(4),
                    AddChars("a".into()),
                ]),
            ])
        ]),
    ];
    let b = vec![
        AddWithGroup(vec![
            AddWithGroup(vec![
                AddWithGroup(vec![
                    AddSkip(4),
                    AddChars("b".into()),
                ]),
            ])
        ]),
    ];

    let (a_, b_) = transform_insertions(&a, &b);

    let a_res = normalize(compose::compose(&(vec![], a), &a_));
    let b_res = normalize(compose::compose(&(vec![], b), &b_));
    assert_eq!(a_res, b_res);
}

#[test]
fn test_transform_feedback() {
    let a = vec![
        // AddWithGroup(vec![
        //     AddWithGroup(vec![
                AddWithGroup(vec![
                    AddSkip(1),
                    AddGroup(map! { "tag" => "b" }, vec![AddSkip(3)]),
                ]),
        //     ])
        // ]),
    ];
    let b = vec![
        // AddWithGroup(vec![
        //     AddWithGroup(vec![
                AddWithGroup(vec![
                    AddSkip(2),
                    AddGroup(map! { "tag" => "b" }, vec![AddSkip(3)]),
                ]),
        //     ])
        // ]),
    ];

    let (a_, b_) = transform_insertions(&a, &b);

    let a_res = normalize(compose::compose(&(vec![], a.clone()), &a_));
    let b_res = normalize(compose::compose(&(vec![], b.clone()), &b_));
    assert_eq!(a_res.1, b_res.1); // TODO fix the normalize case for deletes??


    let (b_, a_) = transform_insertions(&b, &a);

    let a_res = normalize(compose::compose(&(vec![], b.clone()), &b_));
    let b_res = normalize(compose::compose(&(vec![], a.clone()), &a_));
    assert_eq!(a_res.1, b_res.1); // TODO fix the normalize case for deletes??
}

#[test]
fn test_transform_dawn() {
    let a = vec![
        DelSkip(2),
        DelChars(1),
    ];
    let b = vec![
        DelSkip(2),
        DelChars(1),
    ];

    let (a_, b_) = transform_deletions(&a, &b);

    let res = (vec![
        DelSkip(2),
        DelChars(1),
    ], vec![]);

    let a_res = normalize(compose::compose(&(a, vec![]), &(a_, vec![])));
    let b_res = normalize(compose::compose(&(b, vec![]), &(b_, vec![])));

    println!("A : {:?}", a_res);
    println!("B : {:?}", b_res);
    println!("r : {:?}", res);

    assert_eq!(a_res, res.clone());
    assert_eq!(b_res, res.clone());
}

#[test]
fn test_transform_zone() {
    let a = vec![
        DelSkip(1),
        DelChars(1),
    ];
    let b = vec![
        DelSkip(2),
        DelChars(1),
    ];

    let (a_, b_) = transform_deletions(&a, &b);

    let res = (vec![
        DelSkip(1),
        DelChars(2),
    ], vec![]);

    let a_res = normalize(compose::compose(&(a, vec![]), &(a_, vec![])));
    let b_res = normalize(compose::compose(&(b, vec![]), &(b_, vec![])));

    println!("A : {:?}", a_res);
    println!("B : {:?}", b_res);
    println!("r : {:?}", res);

    assert_eq!(a_res, res.clone());
    assert_eq!(b_res, res.clone());
}

#[test]
fn test_transform_everyday() {
    let a = vec![
        DelWithGroup(vec![
            DelGroup(vec![]),
        ]),
    ];
    let b = vec![
        DelWithGroup(vec![
            DelGroup(vec![]),
        ]),
    ];

    let (a_, b_) = transform_deletions(&a, &b);

    let res = (vec![
        DelWithGroup(vec![
            DelGroup(vec![]),
        ]),
    ], vec![]);

    let a_res = normalize(compose::compose(&(a, vec![]), &(a_, vec![])));
    let b_res = normalize(compose::compose(&(b, vec![]), &(b_, vec![])));

    println!("A : {:?}", a_res);
    println!("B : {:?}", b_res);
    println!("r : {:?}", res);

    assert_eq!(a_res, res.clone());
    assert_eq!(b_res, res.clone());
}


#[test]
fn test_transform_pick() {
    let a = (vec![
        DelWithGroup(vec![
            DelGroup(vec![]),
        ]),
    ], vec![]);
    let b = (vec![], vec![
        AddWithGroup(vec![
            AddWithGroup(vec![
                AddChars("hi".into()),
            ]),
        ]),
    ]);

    let (a_, b_) = transform(&a, &b);

    // let res = (vec![
    //     DelWithGroup(vec![
    //         DelGroup(vec![]),
    //     ]),
    // ], vec![]);

    let a_res = normalize(compose::compose(&a, &a_));
    let b_res = normalize(compose::compose(&b, &b_));

    // println!("A : {:?}", a_res);
    // println!("B : {:?}", b_res);
    // println!("r : {:?}", res);
    //
    // assert_eq!(a_res, res.clone());
    // assert_eq!(b_res, res.clone());

    assert_eq!(a_res, b_res);
}

#[test]
fn test_transform_hot() {
    let a = (vec![
        DelWithGroup(vec![
            DelWithGroup(vec![
                DelGroup(vec![
                    DelSkip(6),
                ]),
            ]),
        ]),
    ], vec![
        AddWithGroup(vec![
            AddWithGroup(vec![
                AddGroup(map! { "tag" => "p"}, vec![
                    AddChars("hi".into()),
                    AddSkip(6),
                ]),
            ]),
        ]),
    ]);
    let b = (vec![], vec![
        AddWithGroup(vec![
            AddWithGroup(vec![
                AddWithGroup(vec![
                    AddSkip(6),
                    AddChars("a".into()),
                ]),
            ]),
        ]),
    ]);

    let (a_, b_) = transform(&a, &b);

    let a_res = normalize(compose::compose(&a, &a_));
    let b_res = normalize(compose::compose(&b, &b_));

    println!("");
    println!("A' {:?}", a_res);
    println!("B' {:?}", b_res);
    println!("");

    assert_eq!(a_res, b_res);
}

fn op_transform_compare(a: Op, b: Op) {
    let (a_, b_) = transform(&a, &b);

    let a_res = normalize(compose::compose(&a, &a_));
    let b_res = normalize(compose::compose(&b, &b_));

    println!("");
    println!("A' {:?}", a_res);
    println!("B' {:?}", b_res);
    println!("");

    assert_eq!(a_res, b_res);
}

#[test]
fn test_transform_piece() {
    op_transform_compare(
        op_span!(
            [DelWithGroup([DelWithGroup([DelWithGroup([DelSkip(10), DelChars(5)])])])],
            [],
        ),
        op_span!(
            [DelWithGroup([DelWithGroup([DelWithGroup([DelSkip(7), DelGroupAll(), DelChars(7)])])])],
            [],
        )
    )
}

#[test]
fn test_transform_del() {
    let mut doc = doc_span![
        DocGroup({"tag": "li"}, [
            DocGroup({"tag": "h1"}, [
                DocChars("Hello! Sup?"),
            ]),
            DocGroup({"tag": "p"}, [
                DocChars("World!"),
            ]),
        ]),
    ];

    // Flatten client A operations.
    let op_a = op_span!(
        [DelWithGroup([DelGroup([DelSkip(6), DelChars(1), DelSkip(4)])])],
        [AddWithGroup([AddGroup({"tag": "h1"}, [AddSkip(6)]), AddGroup({"tag": "p"}, [AddSkip(4)])])],
    );

    // Flatten client B operations.
    let op_b = op_span!(
        [DelSkip(1)],
        [AddWithGroup([AddGroup({"tag": "ul"}, [AddGroup({"tag": "li"}, [AddSkip(1)])])])],
    );

    // Tranform
    let (a_, b_) = transform(&op_a, &op_b);

    // Apply original ops
    let doc_a = apply_operation(&doc, &op_a);
    let doc_b = apply_operation(&doc, &op_b);

    // Apply transformed ops
    let a_res = apply_operation(&doc_a, &a_);
    let b_res = apply_operation(&doc_b, &b_);

    // Compare
    assert_eq!(a_res, b_res);
}
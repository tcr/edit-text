#![allow(unused_imports)]

#[macro_use]
extern crate env_logger;
extern crate log;
#[macro_use]
extern crate oatie;
extern crate term_painter;

use oatie::compose;
use oatie::doc::*;
use oatie::normalize;
use oatie::transform::*;
use oatie::apply_operation;
use oatie::transform_test::run_transform_test;
use std::path::Path;
use std::fs::{read_dir, File};
use std::io::prelude::*;

fn test_start() {
    let _ = env_logger::init();
}

#[test]
fn test_transform_anthem() {
    test_start();

    let a = add_span![
        AddGroup({"tag": "p"}, [AddSkip(10)]), AddGroup({"tag": "p"}, [AddSkip(10)]),
    ];
    let b = add_span![
        AddSkip(5), AddGroup({"tag": "b"}, [AddSkip(10)]),
    ];

    let (a_, b_) = transform_insertions(&a, &b);

    let res = op_span!(
        [],
        [
            AddGroup({"tag": "p"}, [AddSkip(5), AddGroup({"tag": "b"}, [AddSkip(5)])]),
            AddGroup({"tag": "p"}, [AddGroup({"tag": "b"}, [AddSkip(5)]), AddSkip(5)]),
        ],
    );

    let a_res = normalize(compose::compose(&(vec![], a), &a_));
    let b_res = normalize(compose::compose(&(vec![], b.clone()), &b_));
    assert_eq!(a_res, res.clone());
    assert_eq!(b_res, res.clone());
}

#[test]
fn test_transform_yellow() {
    let a = add_span![
        AddGroup({"tag": "ul"}, [
            AddGroup({"tag": "li"}, [
                AddSkip(5)
            ])
        ]),
    ];
    let b = add_span![
        AddSkip(3),
        AddGroup({"tag": "p"}, [
            AddSkip(2)
        ]),
        AddGroup({"tag": "p"}, [
            AddSkip(3)
        ]),
    ];

    let (a_, b_) = transform_insertions(&a, &b);

    let res = op_span!([], [
        AddGroup({"tag": "ul"}, [
            AddGroup({"tag": "li"}, [
                AddSkip(3),
                AddGroup({"tag": "p"}, [
                    AddSkip(2)
                ]),
            ])
        ]),
        AddGroup({"tag": "p"}, [
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
    let a = add_span![
        AddGroup({"tag": "ul"}, [
            AddGroup({"tag": "li"}, [
                AddSkip(5)
            ])
        ]),
    ];
    let b = add_span![
        AddSkip(2),
        AddGroup({"tag": "ul"}, [
            AddGroup({"tag": "li"}, [
                AddSkip(2)
            ])
        ]),
    ];

    println!("HERE IS A: {:?}", a);
    println!("HERE IS B: {:?}", b);

    let (a_, b_) = transform_insertions(&a, &b);

    println!("lol");

    let res = op_span!([], [
        AddGroup({"tag": "ul"}, [
            AddGroup({"tag": "li"}, [
                AddSkip(2)
            ]),
            AddGroup({"tag": "li"}, [
                AddSkip(2)
            ]),
            AddGroup({"tag": "li"}, [
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
    let a = add_span![
        AddGroup({"tag": "h1"}, [
            AddSkip(8)
        ]),
        AddGroup({"tag": "p"}, [
            AddSkip(5)
        ]),
    ];
    let b = add_span![
        AddGroup({"tag": "h3"}, [
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
    let a = add_span![
        AddWithGroup([
            AddWithGroup([
                AddWithGroup([
                ]),
            ])
        ]),
        AddGroup({"tag": "p"}, [
            AddSkip(5)
        ]),
    ];
    let b = add_span![
        AddGroup({"tag": "h3"}, [
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
    let a = add_span![
        AddWithGroup([AddWithGroup([AddWithGroup([AddSkip(4), AddChars("a")])])]),
    ];
    let b = add_span![
        AddWithGroup([AddWithGroup([AddWithGroup([AddSkip(4), AddChars("b")])])]),
    ];

    let (a_, b_) = transform_insertions(&a, &b);

    let a_res = normalize(compose::compose(&(vec![], a), &a_));
    let b_res = normalize(compose::compose(&(vec![], b), &b_));
    assert_eq!(a_res, b_res);
}

#[test]
fn test_transform_feedback() {
    let a = add_span![
        // AddWithGroup(vec![
        //     AddWithGroup(vec![
                AddWithGroup([
                    AddSkip(1),
                    AddGroup({"tag": "b"}, [AddSkip(3)]),
                ]),
        //     ])
        // ]),
    ];
    let b = add_span![
        // AddWithGroup(vec![
        //     AddWithGroup(vec![
                AddWithGroup([
                    AddSkip(2),
                    AddGroup({"tag": "b"}, [AddSkip(3)]),
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
    let a = del_span![DelSkip(2), DelChars(1),];
    let b = del_span![DelSkip(2), DelChars(1),];

    let (a_, b_) = transform_deletions(&a, &b);

    let res = op_span!([DelSkip(2), DelChars(1)], []);

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
    let a = del_span![DelSkip(1), DelChars(1),];
    let b = del_span![DelSkip(2), DelChars(1),];

    let (a_, b_) = transform_deletions(&a, &b);

    let res = op_span!([DelSkip(1), DelChars(2)], []);

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
    let a = del_span![DelWithGroup([DelGroup([])]),];
    let b = del_span![DelWithGroup([DelGroup([])]),];

    let (a_, b_) = transform_deletions(&a, &b);

    let res = op_span!([DelWithGroup([DelGroup([])])], []);

    let a_res = normalize(compose::compose(&(a, vec![]), &(a_, vec![])));
    let b_res = normalize(compose::compose(&(b, vec![]), &(b_, vec![])));

    println!("A : {:?}", a_res);
    println!("B : {:?}", b_res);
    println!("r : {:?}", res);

    assert_eq!(a_res, res.clone());
    assert_eq!(b_res, res.clone());
}

#[test]
fn test_transform_del() {
    let doc = doc_span![
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

// TODO how do you ? this
#[test]
fn test_transform_folder() {
    let folder = Path::new(file!())
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("in");
    for path in read_dir(folder).unwrap() {
        println!();
        println!();
        println!();
        println!();
        println!();
        println!("{:?}", path);
        println!();

        // Load the test file
        let mut f = File::open(path.unwrap().path()).unwrap();
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();

        // TODO this should include some result
        run_transform_test(&s).unwrap();
    }
}

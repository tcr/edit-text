#![allow(unused_imports)]

extern crate env_logger;
#[macro_use]
extern crate log;
#[macro_use]
extern crate oatie;
extern crate rand;
extern crate term_painter;

use oatie::*;
use oatie::compose::*;
use oatie::doc::*;
use rand::{thread_rng, Rng};
use std::collections::HashMap;

fn test_start() {
    let _ = env_logger::init();
}

#[test]
fn test_compose_del_del() {
    test_start();

    assert_eq!(
        compose_del_del(&vec![DelSkip(6), DelChars(6)], &vec![DelChars(3)]),
        vec![DelChars(3), DelSkip(3), DelChars(6)]
    );

    assert_eq!(
        compose_del_del(&vec![DelSkip(6), DelChars(6)], &vec![DelChars(6)]),
        vec![DelChars(12)]
    );

    assert_eq!(
        compose_del_del(&vec![DelWithGroup(vec![DelChars(6)])], &vec![DelGroupAll]),
        vec![DelGroupAll]
    );

    assert_eq!(
        compose_del_del(
            &vec![DelWithGroup(vec![DelChars(6)])],
            &vec![DelWithGroup(vec![DelChars(6)])],
        ),
        vec![DelWithGroup(vec![DelChars(12)])]
    );

    assert_eq!(
        compose_del_del(
            &vec![DelSkip(2), DelChars(6), DelSkip(1), DelChars(2), DelSkip(1)],
            &vec![DelSkip(1), DelChars(1), DelSkip(1)],
        ),
        vec![DelSkip(1), DelChars(7), DelSkip(1), DelChars(2), DelSkip(1)]
    );


    assert_eq!(
        compose_del_del(
            &del_span![DelGroup([DelSkip(11)])],
            &del_span![DelSkip(6), DelChars(1)],
        ),
        del_span![DelGroup([DelSkip(6), DelChars(1), DelSkip(4)])]
    );
}

#[test]
fn test_compose_add_add() {
    assert_eq!(
        compose_add_add(
            &vec![AddChars("World!".to_owned())],
            &vec![AddChars("Hello ".to_owned())],
        ),
        vec![AddChars("Hello World!".to_owned())]
    );

    assert_eq!(
        compose_add_add(
            &vec![AddChars("edef".to_owned())],
            &vec![
                AddChars("d".to_owned()),
                AddSkip(1),
                AddChars("a".to_owned()),
                AddSkip(1),
                AddChars("b".to_owned()),
                AddSkip(1),
                AddChars("e".to_owned()),
                AddSkip(1),
            ],
        ),
        vec![AddChars("deadbeef".to_owned())]
    );

    assert_eq!(
        compose_add_add(
            &vec![AddSkip(10), AddChars("h".to_owned())],
            &vec![AddSkip(11), AddChars("i".to_owned())],
        ),
        vec![AddSkip(10), AddChars("hi".to_owned())]
    );

    assert_eq!(
        compose_add_add(
            &vec![
                AddSkip(5),
                AddChars("yEH".to_owned()),
                AddSkip(1),
                AddChars("GlG5".to_owned()),
                AddSkip(4),
                AddChars("nnG".to_owned()),
                AddSkip(1),
                AddChars("ra8c".to_owned()),
                AddSkip(1),
            ],
            &vec![AddSkip(10), AddChars("Eh".to_owned()), AddSkip(16)],
        ),
        vec![
            AddSkip(5),
            AddChars("yEH".to_owned()),
            AddSkip(1),
            AddChars("GEhlG5".to_owned()),
            AddSkip(4),
            AddChars("nnG".to_owned()),
            AddSkip(1),
            AddChars("ra8c".to_owned()),
            AddSkip(1),
        ]
    );
}

#[test]
fn test_compose_add_del() {
    test_start();

    assert_eq!(
        compose_add_del(
            &vec![
                AddSkip(4),
                AddChars("0O".to_owned()),
                AddSkip(5),
                AddChars("mnc".to_owned()),
                AddSkip(3),
                AddChars("gbL".to_owned()),
            ],
            &vec![
                DelSkip(1),
                DelChars(1),
                DelSkip(3),
                DelChars(2),
                DelSkip(2),
                DelChars(9),
                DelSkip(1),
                DelChars(1),
            ],
        ),
        (
            vec![
                DelSkip(1),
                DelChars(1),
                DelSkip(2),
                DelChars(1),
                DelSkip(2),
                DelChars(5),
            ],
            vec![
                AddSkip(3),
                AddChars("0".to_owned()),
                AddSkip(2),
                AddChars("b".to_owned()),
            ],
        )
    );
}

#[test]
fn test_compose() {
    test_start();

    assert_eq!(
        normalize(compose(
            &op_span!([], [
        AddGroup({"tag": "p"}, [AddSkip(6)])
    ]),
            &op_span!([
        DelGroup([DelSkip(6)])
    ], [
        AddGroup({"tag": "p"}, [AddSkip(4)]),
        AddGroup({"tag": "p"}, [AddSkip(2)])
    ]),
        )),
        op_span!([], [
        AddGroup({"tag": "p"}, [AddSkip(4)]),
        AddGroup({"tag": "p"}, [AddSkip(2)])
    ])
    );


    assert_eq!(
        compose(
            &op_span!(
                [DelWithGroup([DelSkip(5), DelWithGroup([]), DelSkip(1)])],
                [AddWithGroup([AddSkip(5), AddWithGroup([]), AddSkip(1), AddGroup({"client": "left", "tag": "caret"}, [])])],
            ),
            &op_span!(
                [DelWithGroup([DelSkip(5), DelGroup([])])],
                []
            ),
        ),
        op_span!(
            [DelWithGroup([DelSkip(5), DelGroup([]), DelSkip(1)])], 
            [AddWithGroup([AddSkip(6), AddGroup({"tag": "caret", "client": "left"}, [])])],
        ),
    );
}

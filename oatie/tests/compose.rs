#![allow(unused_imports)]

extern crate env_logger;
#[macro_use]
extern crate log;
#[macro_use]
extern crate oatie;
extern crate term_painter;

use oatie::apply::*;
use oatie::compose::*;
use oatie::doc::*;
use oatie::*;
use oatie::normalize::*;
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

    // assert_eq!(
    //     compose_del_del(&vec![DelWithGroup(vec![DelChars(6)])], &vec![DelGroupAll]),
    //     vec![DelGroupAll]
    // );

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
            &vec![AddChars(DocString::from_str("World!"))],
            &vec![AddChars(DocString::from_str("Hello "))],
        ),
        vec![AddChars(DocString::from_str("Hello World!"))],
    );

    assert_eq!(
        compose_add_add(
            &vec![AddChars(DocString::from_str("edef"))],
            &vec![
                AddChars(DocString::from_str("d")),
                AddSkip(1),
                AddChars(DocString::from_str("a")),
                AddSkip(1),
                AddChars(DocString::from_str("b")),
                AddSkip(1),
                AddChars(DocString::from_str("e")),
                AddSkip(1),
            ],
        ),
        vec![AddChars(DocString::from_str("deadbeef"))],
    );

    assert_eq!(
        compose_add_add(
            &vec![AddSkip(10), AddChars(DocString::from_str("h"))],
            &vec![AddSkip(11), AddChars(DocString::from_str("i"))],
        ),
        vec![AddSkip(10), AddChars(DocString::from_str("hi"))],
    );

    assert_eq!(
        compose_add_add(
            &vec![
                AddSkip(5),
                AddChars(DocString::from_str("yEH")),
                AddSkip(1),
                AddChars(DocString::from_str("GlG5")),
                AddSkip(4),
                AddChars(DocString::from_str("nnG")),
                AddSkip(1),
                AddChars(DocString::from_str("ra8c")),
                AddSkip(1),
            ],
            &vec![
                AddSkip(10),
                AddChars(DocString::from_str("Eh")),
                AddSkip(16),
            ],
        ),
        vec![
            AddSkip(5),
            AddChars(DocString::from_str("yEH")),
            AddSkip(1),
            AddChars(DocString::from_str("GEhlG5")),
            AddSkip(4),
            AddChars(DocString::from_str("nnG")),
            AddSkip(1),
            AddChars(DocString::from_str("ra8c")),
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
                AddChars(DocString::from_str("0O")),
                AddSkip(5),
                AddChars(DocString::from_str("mnc")),
                AddSkip(3),
                AddChars(DocString::from_str("gbL")),
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
                AddChars(DocString::from_str("0")),
                AddSkip(2),
                AddChars(DocString::from_str("b")),
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
            &op_span!([DelWithGroup([DelSkip(5), DelGroup([])])], []),
        ),
        op_span!(
            [DelWithGroup([DelSkip(5), DelGroup([]), DelSkip(1)])],
            [AddWithGroup([AddSkip(6), AddGroup({"tag": "caret", "client": "left"}, [])])],
        ),
    );
}

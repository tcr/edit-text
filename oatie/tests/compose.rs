#![allow(unused_imports)]

use env_logger;
#[macro_use]
extern crate log;
#[macro_use]
extern crate oatie;

use oatie::apply::*;
use oatie::compose::*;
use oatie::doc::*;
use oatie::normalize::*;
use oatie::rtf::*;
use oatie::*;
use std::collections::HashMap;

fn test_start() {
    let _ = env_logger::init();
}

#[test]
fn test_compose_del_del() {
    test_start();

    assert_eq!(
        compose_del_del::<RtfSchema>(&vec![DelSkip(6), DelChars(6)], &vec![DelChars(3)]),
        vec![DelChars(3), DelSkip(3), DelChars(6)]
    );

    assert_eq!(
        compose_del_del::<RtfSchema>(&vec![DelSkip(6), DelChars(6)], &vec![DelChars(6)]),
        vec![DelChars(12)]
    );

    // assert_eq!(
    //     compose_del_del(&vec![DelWithGroup(vec![DelChars(6)])], &vec![DelGroupAll]),
    //     vec![DelGroupAll]
    // );

    assert_eq!(
        compose_del_del::<RtfSchema>(
            &vec![DelWithGroup(vec![DelChars(6)])],
            &vec![DelWithGroup(vec![DelChars(6)])],
        ),
        vec![DelWithGroup(vec![DelChars(12)])]
    );

    assert_eq!(
        compose_del_del::<RtfSchema>(
            &vec![DelSkip(2), DelChars(6), DelSkip(1), DelChars(2), DelSkip(1)],
            &vec![DelSkip(1), DelChars(1), DelSkip(1)],
        ),
        vec![DelSkip(1), DelChars(7), DelSkip(1), DelChars(2), DelSkip(1)]
    );

    assert_eq!(
        compose_del_del::<RtfSchema>(
            &del_span![DelGroup([DelSkip(11)])],
            &del_span![DelSkip(6), DelChars(1)],
        ),
        del_span![DelGroup([DelSkip(6), DelChars(1), DelSkip(4)])]
    );
}

#[test]
fn test_compose_add_add() {
    assert_eq!(
        compose_add_add::<RtfSchema>(
            &vec![AddChars(DocString::from_str("World!"), StyleSet::new())],
            &vec![AddChars(DocString::from_str("Hello "), StyleSet::new())],
        ),
        vec![AddChars(
            DocString::from_str("Hello World!"),
            StyleSet::new()
        )],
    );

    assert_eq!(
        compose_add_add::<RtfSchema>(
            &vec![AddChars(DocString::from_str("edef"), StyleSet::new())],
            &vec![
                AddChars(DocString::from_str("d"), StyleSet::new()),
                AddSkip(1),
                AddChars(DocString::from_str("a"), StyleSet::new()),
                AddSkip(1),
                AddChars(DocString::from_str("b"), StyleSet::new()),
                AddSkip(1),
                AddChars(DocString::from_str("e"), StyleSet::new()),
                AddSkip(1),
            ],
        ),
        vec![AddChars(DocString::from_str("deadbeef"), StyleSet::new())],
    );

    assert_eq!(
        compose_add_add::<RtfSchema>(
            &vec![
                AddSkip(10),
                AddChars(DocString::from_str("h"), StyleSet::new())
            ],
            &vec![
                AddSkip(11),
                AddChars(DocString::from_str("i"), StyleSet::new())
            ],
        ),
        vec![
            AddSkip(10),
            AddChars(DocString::from_str("hi"), StyleSet::new())
        ],
    );

    assert_eq!(
        compose_add_add::<RtfSchema>(
            &vec![
                AddSkip(5),
                AddChars(DocString::from_str("yEH"), StyleSet::new()),
                AddSkip(1),
                AddChars(DocString::from_str("GlG5"), StyleSet::new()),
                AddSkip(4),
                AddChars(DocString::from_str("nnG"), StyleSet::new()),
                AddSkip(1),
                AddChars(DocString::from_str("ra8c"), StyleSet::new()),
                AddSkip(1),
            ],
            &vec![
                AddSkip(10),
                AddChars(DocString::from_str("Eh"), StyleSet::new()),
                AddSkip(16),
            ],
        ),
        vec![
            AddSkip(5),
            AddChars(DocString::from_str("yEH"), StyleSet::new()),
            AddSkip(1),
            AddChars(DocString::from_str("GEhlG5"), StyleSet::new()),
            AddSkip(4),
            AddChars(DocString::from_str("nnG"), StyleSet::new()),
            AddSkip(1),
            AddChars(DocString::from_str("ra8c"), StyleSet::new()),
            AddSkip(1),
        ]
    );
}

#[test]
fn test_compose_add_del() {
    test_start();

    assert_eq!(
        compose_add_del::<RtfSchema>(
            &vec![
                AddSkip(4),
                AddChars(DocString::from_str("0O"), StyleSet::new()),
                AddSkip(5),
                AddChars(DocString::from_str("mnc"), StyleSet::new()),
                AddSkip(3),
                AddChars(DocString::from_str("gbL"), StyleSet::new()),
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
                AddChars(DocString::from_str("0"), StyleSet::new()),
                AddSkip(2),
                AddChars(DocString::from_str("b"), StyleSet::new()),
            ],
        )
    );
}

#[test]
fn test_compose() {
    test_start();

    assert_eq!(
        normalize::<RtfSchema>(compose(
            &op_span!([], [AddGroup(Attrs::Text, [AddSkip(6)]),]),
            &op_span!(
                [DelGroup([DelSkip(6)])],
                [
                    AddGroup(Attrs::Text, [AddSkip(4)]),
                    AddGroup(Attrs::Text, [AddSkip(2)]),
                ]
            ),
        )),
        op_span!(
            [],
            [
                AddGroup(Attrs::Text, [AddSkip(4)]),
                AddGroup(Attrs::Text, [AddSkip(2)]),
            ]
        )
    );

    assert_eq!(
        compose::<RtfSchema>(
            &op_span!(
                [DelWithGroup([DelSkip(5), DelWithGroup([]), DelSkip(1)])],
                [AddWithGroup([
                    AddSkip(5),
                    AddWithGroup([]),
                    AddSkip(1),
                    AddGroup(
                        Attrs::Caret {
                            client_id: "left".to_string(),
                            focus: true
                        },
                        []
                    )
                ])],
            ),
            &op_span!([DelWithGroup([DelSkip(5), DelGroup([])])], []),
        ),
        op_span!(
            [DelWithGroup([DelSkip(5), DelGroup([]), DelSkip(1)])],
            [AddWithGroup([
                AddSkip(6),
                AddGroup(
                    Attrs::Caret {
                        client_id: "left".to_string(),
                        focus: true
                    },
                    []
                )
            ])],
        ),
    );
}

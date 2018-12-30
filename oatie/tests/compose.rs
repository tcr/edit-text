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
        compose_del_del::<RtfSchema>(&vec![DelSkip(6), DelText(6)], &vec![DelText(3)]),
        vec![DelText(3), DelSkip(3), DelText(6)]
    );

    assert_eq!(
        compose_del_del::<RtfSchema>(&vec![DelSkip(6), DelText(6)], &vec![DelText(6)]),
        vec![DelText(12)]
    );

    // assert_eq!(
    //     compose_del_del(&vec![DelWithGroup(vec![DelText(6)])], &vec![DelGroupAll]),
    //     vec![DelGroupAll]
    // );

    assert_eq!(
        compose_del_del::<RtfSchema>(
            &vec![DelWithGroup(vec![DelText(6)])],
            &vec![DelWithGroup(vec![DelText(6)])],
        ),
        vec![DelWithGroup(vec![DelText(12)])]
    );

    assert_eq!(
        compose_del_del::<RtfSchema>(
            &vec![DelSkip(2), DelText(6), DelSkip(1), DelText(2), DelSkip(1)],
            &vec![DelSkip(1), DelText(1), DelSkip(1)],
        ),
        vec![DelSkip(1), DelText(7), DelSkip(1), DelText(2), DelSkip(1)]
    );

    assert_eq!(
        compose_del_del::<RtfSchema>(
            &del_span![DelGroup([DelSkip(11)])],
            &del_span![DelSkip(6), DelText(1)],
        ),
        del_span![DelGroup([DelSkip(6), DelText(1), DelSkip(4)])]
    );
}

#[test]
fn test_compose_add_add() {
    assert_eq!(
        compose_add_add::<RtfSchema>(
            &vec![AddText(StyleSet::new(), DocString::from_str("World!"))],
            &vec![AddText(StyleSet::new(), DocString::from_str("Hello "))],
        ),
        vec![AddText(
            StyleSet::new(),
            DocString::from_str("Hello World!"),
        )],
    );

    assert_eq!(
        compose_add_add::<RtfSchema>(
            &vec![AddText(StyleSet::new(), DocString::from_str("edef"))],
            &vec![
                AddText(StyleSet::new(), DocString::from_str("d")),
                AddSkip(1),
                AddText(StyleSet::new(), DocString::from_str("a")),
                AddSkip(1),
                AddText(StyleSet::new(), DocString::from_str("b")),
                AddSkip(1),
                AddText(StyleSet::new(), DocString::from_str("e")),
                AddSkip(1),
            ],
        ),
        vec![AddText(StyleSet::new(), DocString::from_str("deadbeef"))],
    );

    assert_eq!(
        compose_add_add::<RtfSchema>(
            &vec![
                AddSkip(10),
                AddText(StyleSet::new(), DocString::from_str("h"))
            ],
            &vec![
                AddSkip(11),
                AddText(StyleSet::new(), DocString::from_str("i"))
            ],
        ),
        vec![
            AddSkip(10),
            AddText(StyleSet::new(), DocString::from_str("hi"))
        ],
    );

    assert_eq!(
        compose_add_add::<RtfSchema>(
            &vec![
                AddSkip(5),
                AddText(StyleSet::new(), DocString::from_str("yEH")),
                AddSkip(1),
                AddText(StyleSet::new(), DocString::from_str("GlG5")),
                AddSkip(4),
                AddText(StyleSet::new(), DocString::from_str("nnG")),
                AddSkip(1),
                AddText(StyleSet::new(), DocString::from_str("ra8c")),
                AddSkip(1),
            ],
            &vec![
                AddSkip(10),
                AddText(StyleSet::new(), DocString::from_str("Eh")),
                AddSkip(16),
            ],
        ),
        vec![
            AddSkip(5),
            AddText(StyleSet::new(), DocString::from_str("yEH")),
            AddSkip(1),
            AddText(StyleSet::new(), DocString::from_str("GEhlG5")),
            AddSkip(4),
            AddText(StyleSet::new(), DocString::from_str("nnG")),
            AddSkip(1),
            AddText(StyleSet::new(), DocString::from_str("ra8c")),
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
                AddText(StyleSet::new(), DocString::from_str("0O")),
                AddSkip(5),
                AddText(StyleSet::new(), DocString::from_str("mnc")),
                AddSkip(3),
                AddText(StyleSet::new(), DocString::from_str("gbL")),
            ],
            &vec![
                DelSkip(1),
                DelText(1),
                DelSkip(3),
                DelText(2),
                DelSkip(2),
                DelText(9),
                DelSkip(1),
                DelText(1),
            ],
        ),
        (
            vec![
                DelSkip(1),
                DelText(1),
                DelSkip(2),
                DelText(1),
                DelSkip(2),
                DelText(5),
            ],
            vec![
                AddSkip(3),
                AddText(StyleSet::new(), DocString::from_str("0")),
                AddSkip(2),
                AddText(StyleSet::new(), DocString::from_str("b")),
            ],
        )
    );
}

#[test]
fn test_compose() {
    test_start();

    assert_eq!(
        normalize::<RtfSchema>(compose(
            &op_span!([], [AddGroup(Attrs::Para, [AddSkip(6)]),]),
            &op_span!(
                [DelGroup([DelSkip(6)])],
                [
                    AddGroup(Attrs::Para, [AddSkip(4)]),
                    AddGroup(Attrs::Para, [AddSkip(2)]),
                ]
            ),
        )),
        op_span!(
            [],
            [
                AddGroup(Attrs::Para, [AddSkip(4)]),
                AddGroup(Attrs::Para, [AddSkip(2)]),
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

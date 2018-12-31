#![allow(unused_imports)]

use env_logger;
#[macro_use]
extern crate log;

use std::collections::HashMap;

use oatie::apply::*;
use oatie::doc::AddElement::*;
use oatie::doc::DelElement::*;
use oatie::doc::DocElement::*;
use oatie::doc::*;
use oatie::rtf::*;
use oatie::*;

pub fn test_start() {
    if let Ok(_) = env_logger::init() {
        // good
    }
}

#[test]
fn try_this() {
    test_start();

    // let source: DocSpan = vec![
    //     DocText(DocString::from_str("Hello world!")),
    //     DocGroup(HashMap::new(), vec![]),
    // ];

    // TODO this has a DelGroupAll that should be removed:
    // assert_eq!(
    //     apply_delete(
    //         &vec![
    //             DocText("Hello world!".to_owned()),
    //             DocGroup(HashMap::new(), vec![]),
    //         ],
    //         &vec![
    //             DelText(3),
    //             DelSkip(2),
    //             DelText(1),
    //             DelSkip(1),
    //             DelText(5),
    //             DelGroupAll,
    //         ],
    //     ),
    //     vec![DocText("low".to_owned())]
    // );

    assert_eq!(
        apply_delete::<RtfSchema>(
            &vec![DocText(
                StyleSet::new(),
                DocString::from_str("Hello World!")
            )],
            &vec![DelText(6)],
        ),
        vec![DocText(StyleSet::new(), DocString::from_str("World!"))]
    );

    assert_eq!(
        apply_add::<RtfSchema>(
            &vec![DocText(StyleSet::new(), DocString::from_str("World!"))],
            &vec![AddText(StyleSet::new(), DocString::from_str("Hello "))],
        ),
        vec![DocText(
            StyleSet::new(),
            DocString::from_str("Hello World!")
        )],
    );

    assert_eq!(
        apply_add::<RtfSchema>(
            &vec![
                DocGroup(Attrs::Para, vec![]),
                DocText(StyleSet::new(), DocString::from_str("World!")),
            ],
            &vec![
                AddSkip(1),
                AddText(StyleSet::new(), DocString::from_str("Hello "))
            ],
        ),
        vec![
            DocGroup(Attrs::Para, vec![]),
            DocText(StyleSet::new(), DocString::from_str("Hello World!")),
        ]
    );

    assert_eq!(
        apply_delete::<RtfSchema>(
            &vec![DocGroup(
                Attrs::Para,
                vec![DocText(
                    StyleSet::new(),
                    DocString::from_str("Hello Damned World!")
                )],
            )],
            &vec![DelWithGroup(vec![DelSkip(6), DelText(7)])],
        ),
        vec![DocGroup(
            Attrs::Para,
            vec![DocText(
                StyleSet::new(),
                DocString::from_str("Hello World!")
            )],
        )]
    );

    assert_eq!(
        apply_add::<RtfSchema>(
            &vec![DocGroup(
                Attrs::Para,
                vec![DocText(StyleSet::new(), DocString::from_str("Hello!"))],
            )],
            &vec![AddWithGroup(vec![
                AddSkip(5),
                AddText(StyleSet::new(), DocString::from_str(" World")),
            ])],
        ),
        vec![DocGroup(
            Attrs::Para,
            vec![DocText(
                StyleSet::new(),
                DocString::from_str("Hello World!")
            )],
        )]
    );

    assert_eq!(
        apply_operation::<RtfSchema>(
            &vec![DocText(
                StyleSet::new(),
                DocString::from_str("Goodbye World!")
            )],
            &Op(
                vec![DelText(7)],
                vec![AddText(StyleSet::new(), DocString::from_str("Hello"))],
            )
        ),
        vec![DocText(
            StyleSet::new(),
            DocString::from_str("Hello World!")
        )]
    );

    assert_eq!(
        apply_add::<RtfSchema>(
            &vec![DocText(
                StyleSet::new(),
                DocString::from_str("Hello world!")
            )],
            &vec![
                AddSkip(10),
                AddText(StyleSet::new(), DocString::from_str("dd49")),
                AddSkip(2),
            ],
        ),
        vec![DocText(
            StyleSet::new(),
            DocString::from_str("Hello worldd49d!")
        )]
    );
}

#[test]
fn test_lib_op() {
    test_start();

    assert_eq!(
        apply_operation::<RtfSchema>(
            &vec![
                DocText(StyleSet::new(), DocString::from_str("Heo")),
                DocGroup(Attrs::Para, vec![]),
                DocText(StyleSet::new(), DocString::from_str("!")),
            ],
            &Op(
                vec![DelSkip(1), DelText(1), DelSkip(2), DelSkip(1)],
                vec![AddSkip(3)],
            ),
        ),
        vec![
            DocText(StyleSet::new(), DocString::from_str("Ho")),
            DocGroup(Attrs::Para, vec![]),
            DocText(StyleSet::new(), DocString::from_str("!")),
        ]
    );
}

#[test]
fn apply_ghost() {
    test_start();

    assert_eq!(
        apply_operation::<RtfSchema>(
            &doc_span![DocText(" stop crying, little hip hop")],
            &op!([], [AddText("\u{01f47b}")]),
        ),
        doc_span![DocText("\u{01f47b} stop crying, little hip hop")]
    );
}

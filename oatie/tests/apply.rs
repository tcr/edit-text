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
use oatie::*;
use oatie::rtf::*;

pub fn test_start() {
    if let Ok(_) = env_logger::init() {
        // good
    }
}

#[test]
fn try_this() {
    test_start();

    // let source: DocSpan = vec![
    //     DocChars(DocString::from_str("Hello world!")),
    //     DocGroup(HashMap::new(), vec![]),
    // ];

    // TODO this has a DelGroupAll that should be removed:
    // assert_eq!(
    //     apply_delete(
    //         &vec![
    //             DocChars("Hello world!".to_owned()),
    //             DocGroup(HashMap::new(), vec![]),
    //         ],
    //         &vec![
    //             DelChars(3),
    //             DelSkip(2),
    //             DelChars(1),
    //             DelSkip(1),
    //             DelChars(5),
    //             DelGroupAll,
    //         ],
    //     ),
    //     vec![DocChars("low".to_owned())]
    // );

    assert_eq!(
        apply_delete::<RtfSchema>(
            &vec![DocChars(StyleSet::new(), DocString::from_str("Hello World!"))],
            &vec![DelChars(6)],
        ),
        vec![DocChars(StyleSet::new(), DocString::from_str("World!"))]
    );

    assert_eq!(
        apply_add::<RtfSchema>(
            &vec![DocChars(StyleSet::new(), DocString::from_str("World!"))],
            &vec![AddChars(StyleSet::new(), DocString::from_str("Hello "))],
        ),
        vec![DocChars(StyleSet::new(), DocString::from_str("Hello World!"))],
    );

    assert_eq!(
        apply_add::<RtfSchema>(
            &vec![
                DocGroup(Attrs::Text, vec![]),
                DocChars(StyleSet::new(), DocString::from_str("World!")),
            ],
            &vec![AddSkip(1), AddChars(StyleSet::new(), DocString::from_str("Hello "))],
        ),
        vec![
            DocGroup(Attrs::Text, vec![]),
            DocChars(StyleSet::new(), DocString::from_str("Hello World!")),
        ]
    );

    assert_eq!(
        apply_delete::<RtfSchema>(
            &vec![DocGroup(
                Attrs::Text,
                vec![DocChars(StyleSet::new(), DocString::from_str("Hello Damned World!"))],
            )],
            &vec![DelWithGroup(vec![DelSkip(6), DelChars(7)])],
        ),
        vec![DocGroup(
            Attrs::Text,
            vec![DocChars(StyleSet::new(), DocString::from_str("Hello World!"))],
        )]
    );

    assert_eq!(
        apply_add::<RtfSchema>(
            &vec![DocGroup(
                Attrs::Text,
                vec![DocChars(StyleSet::new(), DocString::from_str("Hello!"))],
            )],
            &vec![AddWithGroup(vec![
                AddSkip(5),
                AddChars(StyleSet::new(), DocString::from_str(" World")),
            ])],
        ),
        vec![DocGroup(
            Attrs::Text,
            vec![DocChars(StyleSet::new(), DocString::from_str("Hello World!"))],
        )]
    );

    assert_eq!(
        apply_operation::<RtfSchema>(
            &vec![DocChars(StyleSet::new(), DocString::from_str("Goodbye World!"))],
            &(
                vec![DelChars(7)],
                vec![AddChars(StyleSet::new(), DocString::from_str("Hello"))],
            )
        ),
        vec![DocChars(StyleSet::new(), DocString::from_str("Hello World!"))]
    );

    assert_eq!(
        apply_add::<RtfSchema>(
            &vec![DocChars(StyleSet::new(), DocString::from_str("Hello world!"))],
            &vec![
                AddSkip(10),
                AddChars(StyleSet::new(), DocString::from_str("dd49")),
                AddSkip(2),
            ],
        ),
        vec![DocChars(StyleSet::new(), DocString::from_str("Hello worldd49d!"))]
    );
}

#[test]
fn test_lib_op() {
    test_start();

    assert_eq!(
        apply_operation::<RtfSchema>(
            &vec![
                DocChars(StyleSet::new(), DocString::from_str("Heo")),
                DocGroup(Attrs::Text, vec![]),
                DocChars(StyleSet::new(), DocString::from_str("!")),
            ],
            &(
                vec![DelSkip(1), DelChars(1), DelSkip(2), DelSkip(1)],
                vec![AddSkip(3)],
            ),
        ),
        vec![
            DocChars(StyleSet::new(), DocString::from_str("Ho")),
            DocGroup(Attrs::Text, vec![]),
            DocChars(StyleSet::new(), DocString::from_str("!")),
        ]
    );
}

#[test]
fn apply_ghost() {
    test_start();

    assert_eq!(
        apply_operation::<RtfSchema>(
            &doc_span![DocChars(" stop crying, little hip hop")],
            &op_span![[], [AddChars("\u{01f47b}")]],
        ),
        doc_span![DocChars("\u{01f47b} stop crying, little hip hop")]
    );
}

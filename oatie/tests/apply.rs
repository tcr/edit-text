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
use oatie::style::OpaqueStyleMap;

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
        apply_delete(
            &vec![DocChars(DocString::from_str("Hello World!"), OpaqueStyleMap::new())],
            &vec![DelChars(6)],
        ),
        vec![DocChars(DocString::from_str("World!"), OpaqueStyleMap::new())]
    );

    assert_eq!(
        apply_add(
            &vec![DocChars(DocString::from_str("World!"), OpaqueStyleMap::new())],
            &vec![AddChars(DocString::from_str("Hello "), OpaqueStyleMap::new())],
        ),
        vec![DocChars(DocString::from_str("Hello World!"), OpaqueStyleMap::new())],
    );

    assert_eq!(
        apply_add(
            &vec![
                DocGroup(HashMap::new(), vec![]),
                DocChars(DocString::from_str("World!"), OpaqueStyleMap::new()),
            ],
            &vec![AddSkip(1), AddChars(DocString::from_str("Hello "), OpaqueStyleMap::new())],
        ),
        vec![
            DocGroup(HashMap::new(), vec![]),
            DocChars(DocString::from_str("Hello World!"), OpaqueStyleMap::new()),
        ]
    );

    assert_eq!(
        apply_delete(
            &vec![DocGroup(
                HashMap::new(),
                vec![DocChars(DocString::from_str("Hello Damned World!"), OpaqueStyleMap::new())],
            )],
            &vec![DelWithGroup(vec![DelSkip(6), DelChars(7)])],
        ),
        vec![DocGroup(
            HashMap::new(),
            vec![DocChars(DocString::from_str("Hello World!"), OpaqueStyleMap::new())],
        )]
    );

    assert_eq!(
        apply_add(
            &vec![DocGroup(
                HashMap::new(),
                vec![DocChars(DocString::from_str("Hello!"), OpaqueStyleMap::new())],
            )],
            &vec![AddWithGroup(vec![
                AddSkip(5),
                AddChars(DocString::from_str(" World"), OpaqueStyleMap::new()),
            ])],
        ),
        vec![DocGroup(
            HashMap::new(),
            vec![DocChars(DocString::from_str("Hello World!"), OpaqueStyleMap::new())],
        )]
    );

    assert_eq!(
        apply_operation(
            &vec![DocChars(DocString::from_str("Goodbye World!"), OpaqueStyleMap::new())],
            &(
                vec![DelChars(7)],
                vec![AddChars(DocString::from_str("Hello"), OpaqueStyleMap::new())],
            )
        ),
        vec![DocChars(DocString::from_str("Hello World!"), OpaqueStyleMap::new())]
    );

    assert_eq!(
        apply_add(
            &vec![DocChars(DocString::from_str("Hello world!"), OpaqueStyleMap::new())],
            &vec![
                AddSkip(10),
                AddChars(DocString::from_str("dd49"), OpaqueStyleMap::new()),
                AddSkip(2),
            ],
        ),
        vec![DocChars(DocString::from_str("Hello worldd49d!"), OpaqueStyleMap::new())]
    );
}

#[test]
fn test_lib_op() {
    test_start();

    assert_eq!(
        apply_operation(
            &vec![
                DocChars(DocString::from_str("Heo"), OpaqueStyleMap::new()),
                DocGroup(HashMap::new(), vec![]),
                DocChars(DocString::from_str("!"), OpaqueStyleMap::new()),
            ],
            &(
                vec![DelSkip(1), DelChars(1), DelSkip(2), DelSkip(1)],
                vec![AddSkip(3)],
            ),
        ),
        vec![
            DocChars(DocString::from_str("Ho"), OpaqueStyleMap::new()),
            DocGroup(HashMap::new(), vec![]),
            DocChars(DocString::from_str("!"), OpaqueStyleMap::new()),
        ]
    );
}

#[test]
fn apply_ghost() {
    test_start();

    assert_eq!(
        apply_operation(
            &doc_span![DocChars(" stop crying, little hip hop")],
            &op_span![[], [AddChars("\u{01f47b}")]],
        ),
        doc_span![DocChars("\u{01f47b} stop crying, little hip hop")]
    );
}

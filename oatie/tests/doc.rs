extern crate oatie;
extern crate term_painter;
#[macro_use] extern crate literator;
#[macro_use] extern crate log;
extern crate env_logger;
extern crate rand;

use std::collections::HashMap;

use oatie::*;
use oatie::doc::{DocSpan, Atom};
use oatie::doc::DocElement::*;
use oatie::doc::DelElement::*;
use oatie::doc::AddElement::*;

pub fn test_start() {
    if let Ok(_) = env_logger::init() {
        // good
    }
}

#[test]
fn try_this() {
    test_start();

    let source:DocSpan = vec![
        DocChars("Hello world!".to_owned()),
        DocGroup(HashMap::new(), vec![]),
    ];

    debug_span(&source);

    assert_eq!(iterate(&vec![
        DocChars("Hello world!".to_owned()),
        DocGroup(HashMap::new(), vec![]),
    ]), vec![
        Atom::Char('H'),
        Atom::Char('e'),
        Atom::Char('l'),
        Atom::Char('l'),
        Atom::Char('o'),
        Atom::Char(' '),
        Atom::Char('w'),
        Atom::Char('o'),
        Atom::Char('r'),
        Atom::Char('l'),
        Atom::Char('d'),
        Atom::Char('!'),
        Atom::Enter(HashMap::new()),
        Atom::Leave,
    ]);

    assert_eq!(apply_delete(&vec![
        DocChars("Hello world!".to_owned()),
        DocGroup(HashMap::new(), vec![]),
    ], &vec![
        DelChars(3),
        DelSkip(2),
        DelChars(1),
        DelSkip(1),
        DelChars(5),
        DelGroupAll,
    ]), vec![
        DocChars("low".to_owned()),
    ]);

    assert_eq!(apply_delete(&vec![
        DocChars("Hello World!".to_owned()),
    ], &vec![
        DelChars(6),
    ]), vec![
        DocChars("World!".to_owned()),
    ]);

    assert_eq!(apply_add(&vec![
        DocChars("World!".to_owned()),
    ], &vec![
        AddChars("Hello ".to_owned()),
    ]), vec![
        DocChars("Hello World!".to_owned()),
    ]);

    assert_eq!(apply_add(&vec![
        DocGroup(HashMap::new(), vec![]),
        DocChars("World!".to_owned()),
    ], &vec![
        AddSkip(1),
        AddChars("Hello ".to_owned()),
    ]), vec![
        DocGroup(HashMap::new(), vec![]),
        DocChars("Hello World!".to_owned()),
    ]);

    assert_eq!(apply_delete(&vec![
        DocGroup(HashMap::new(), vec![
            DocChars("Hello Damned World!".to_owned()),
        ]),
    ], &vec![
        DelWithGroup(vec![
            DelSkip(6),
            DelChars(7),
        ]),
    ]), vec![
        DocGroup(HashMap::new(), vec![
            DocChars("Hello World!".to_owned()),
        ]),
    ]);

    assert_eq!(apply_add(&vec![
        DocGroup(HashMap::new(), vec![
            DocChars("Hello!".to_owned()),
        ]),
    ], &vec![
        AddWithGroup(vec![
            AddSkip(5),
            AddChars(" World".to_owned()),
        ]),
    ]), vec![
        DocGroup(HashMap::new(), vec![
            DocChars("Hello World!".to_owned()),
        ]),
    ]);

    assert_eq!(apply_operation(&vec![
        DocChars("Goodbye World!".to_owned()),
    ], &(vec![
        DelChars(7),
    ], vec![
        AddChars("Hello".to_owned()),
    ])), vec![
        DocChars("Hello World!".to_owned()),
    ]);

    assert_eq!(apply_add(&vec![
        DocChars("Hello world!".to_owned())
    ],
    &vec![
        AddSkip(10), AddChars("dd49".to_owned()), AddSkip(2)
    ]),
    vec![
        DocChars("Hello worldd49d!".to_owned())
    ]);
}

#[test]
fn test_lib_op() {
    test_start();

    assert_eq!(apply_operation(&vec![
        DocChars("Heo".to_owned()), DocGroup(HashMap::new(), vec![]), DocChars("!".to_owned())
    ], &(vec![
        DelSkip(1), DelChars(1), DelSkip(2), DelSkip(1)
    ], vec![
        AddSkip(3)
    ])), vec![
        DocChars("Ho".to_owned()), DocGroup(HashMap::new(), vec![]), DocChars("!".to_owned())
    ]);
}

//! ```cargo
//! [dependencies]
//! oatie = { path = "/Users/trim/tcr/edit-text/oatie" }
//! ```

#[macro_use]
extern crate oatie;

use oatie::*;
use oatie::doc::*;

fn main() {
    let doc = 
doc_span![
    DocGroup({"tag": "p"}, [
        DocChars("1"),
    ]),
    DocGroup({"tag": "h2"}, [
        DocChars("1")
    ]),
];

    let pending =
op_span!([
    DelGroup([
        DelSkip(1),
    ]),
], [
    AddGroup({"tag": "bullet"}, [
        AddGroup({"tag": "p"}, [
            AddSkip(1),
        ])
    ]),
]);
    let local =
op_span!([
    DelSkip(1),
    DelWithGroup([
        DelSkip(1)
    ]),
], [
    AddSkip(2),
]);

// ([
//     DelGroup([
//         DelSkip(15)
//     ]),
//     DelSkip(2)
// ], [
//     AddGroup({"tag": "h3"}, [
//         AddSkip(6), AddGroup({"client": "b", "tag": "caret"}, []),
//         AddWithGroup([
//             AddSkip(5)
//         ]),
//         AddSkip(8)
//     ])
// ])

    println!("DOC\n{:?}\n\n", doc);

    let mut r = OT::apply(&Doc(doc.clone()), &pending);
    r = OT::apply(&r, &local);

    println!("HELP\n{:?}\n\n", r);

    println!("----> {:?}", OT::compose(&pending, &local));
    OT::apply(&Doc(doc.clone()), &OT::compose(&pending, &local));

    println!("lol");
}
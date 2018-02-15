//! ```cargo
//! [dependencies]
//! oatie = { path = "/Users/trim/tcr/edit-text/oatie" }
//! ```

#[macro_use]
extern crate oatie;

use oatie::*;
use oatie::doc::*;

fn main() {
    let doc = doc_span!(
DocGroup({"tag": "h3"}, [
    DocGroup({"client": "c", "tag": "caret"}, []),
    DocChars(" Ka"), DocGroup({"client": "b", "tag": "caret"}, []),
    DocChars("eRylC14 5 ")
]),
DocGroup({"tag": "pre"}, [
    DocChars(" 5 4"), DocGroup({"client": "a", "tag": "caret"}, [])
]),
DocGroup({"tag": "h1"}, [
    DocChars("mwis Mercutio, a rich text editor.")
]));
    let first = op_span!(
[
    DelWithGroup([
        DelSkip(6),
    ]),
    DelSkip(2),
], [
    AddWithGroup([
        AddSkip(6),
        AddGroup({"tag": "caret", "client": "b"}, [])
    ]),
    AddWithGroup([
        AddSkip(5)
    ]),
]);
    let next = op_span!(
[
    DelGroup([
        DelSkip(16)
    ])
], [
    AddGroup({"tag": "h3"}, [
        AddSkip(16)
    ]),
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


    let r = OT::apply(&Doc(doc.clone()), &first);
    OT::apply(&r, &next);

    println!("----> {:?}", OT::compose(&first, &next));
    OT::apply(&Doc(doc.clone()), &OT::compose(&first, &next));

    println!("lol");
}
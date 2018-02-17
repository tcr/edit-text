//! ```cargo
//! [dependencies]
//! oatie = { path = "/Users/trim/tcr/edit-text/oatie" }
//! ```

#[macro_use]
extern crate oatie;

use oatie::*;
use oatie::doc::*;
use oatie::schema::RtfSchema;

fn main() {
// let pending_op = op_span!([DelWithGroup([DelGroup([DelSkip(2)]), DelGroup([DelSkip(11), DelWithGroup([]), DelSkip(1)])]), DelGroup([DelWithGroup([DelChars(1), DelGroup([]), DelSkip(1)]), DelWithGroup([DelSkip(1)])]), DelWithGroup([DelSkip(1), DelWithGroup([])]), DelSkip(1)], [AddWithGroup([AddGroup({"tag": "h3"}, [AddChars(" ")]), AddGroup({"tag": "p"}, [AddChars("L"), AddGroup({"tag": "caret", "client": "b"}, []), AddChars(" "), AddSkip(13), AddWithGroup([]), AddSkip(1)])]), AddWithGroup([AddSkip(1), AddChars("b")]), AddWithGroup([AddSkip(1)]), AddWithGroup([AddSkip(1), AddWithGroup([])]), AddSkip(1)]);

// let local_op = op_span!([DelWithGroup([DelGroup([DelChars(1)]), DelGroup([DelChars(1), DelGroup([]), DelSkip(14), DelWithGroup([]), DelSkip(1)])]), DelWithGroup([DelSkip(2)]), DelWithGroup([DelSkip(1)]), DelSkip(2)], [AddWithGroup([AddGroup({"tag": "h3"}, [AddChars("C"), AddSkip(14), AddWithGroup([]), AddSkip(1), AddChars("p")])]), AddGroup({"tag": "bullet"}, [AddWithGroup([AddGroup({"client": "b", "tag": "caret"}, []), AddSkip(1), AddChars("2"), AddSkip(1)])]), AddWithGroup([AddSkip(1)]), AddSkip(2)]);

// let input_op = op_span!([DelWithGroup([DelGroup([DelSkip(2)]), DelGroup([DelSkip(2), DelChars(2), DelSkip(2), DelChars(2), DelSkip(1), DelChars(2), DelGroup([]), DelChars(1)])]), DelWithGroup([DelGroup([DelChars(1), DelGroup([]), DelChars(1)]), DelWithGroup([DelSkip(1)])]), DelWithGroup([DelSkip(1), DelWithGroup([])]), DelSkip(1)], [AddWithGroup([AddGroup({"tag": "pre"}, [AddChars("A"), AddSkip(1), AddGroup({"tag": "caret", "client": "a"}, []), AddSkip(6), AddChars("i ")])]), AddWithGroup([AddWithGroup([AddSkip(1)])]), AddWithGroup([AddSkip(1), AddWithGroup([])]), AddSkip(1)]);

// let doc = Doc(doc_span![DocGroup({"tag": "bullet"}, [DocGroup({"tag": "pre"}, [DocChars("Av"), DocGroup({"client": "a", "tag": "caret"}, []), DocChars("B20 W7i ")])]), DocGroup({"tag": "bullet"}, [DocGroup({"tag": "h3"}, [DocChars("2")])]), DocGroup({"tag": "h3"}, [DocChars("8"), DocGroup({"tag": "caret", "client": "c"}, [])]), DocGroup({"tag": "h1"}, [DocChars("0pos Mercutio, a rich text editor.")])]);



//     // I x P -> I', P'
//     let (pending_op_transform, input_op_transform) = Op::transform::<RtfSchema>(&input_op, &pending_op);
//     // P' x L -> P'', L'


    let input_op_transform = op_span!([
    DelGroup([
        DelChars(1)
    ]),
], [
]);
    let local_op = op_span!([
    DelSkip(1),
], [
    AddGroup({"tag": "bullet"}, [
        AddWithGroup([
            AddGroup({"tag": "caret", "client": "b"}, []),
            AddSkip(1),
        ])
    ]),
]);

    // println!("left {:?}", input_op_transform);
    // println!();
    // println!();
    // println!();
    // println!("against {:?}", local_op);

    println!();

    let (local_op_transform, r) = Op::transform::<RtfSchema>(&input_op_transform, &local_op);
    println!();
    println!();
    println!();
    println!("{:?}", local_op_transform);
    println!("{:?}", r);


    // // client_doc = input_doc : I' : P''
    // let client_op = Op::compose(&pending_op_transform, &local_op_transform);
    // // Reattach to doc.
    // self.doc = Op::apply(&new_doc, &pending_op_transform);
    // validate_doc(&self.doc).expect("Validation error after unrelated pending op");
    // self.doc = Op::apply(&self.doc, &local_op_transform);


// let pending = op_span!([DelWithGroup([DelGroup([DelSkip(2), DelWithGroup([]), DelSkip(8)])]), DelGroup([DelWithGroup([DelSkip(1)])]), DelWithGroup([DelSkip(1), DelWithGroup([])]), DelSkip(1)], [AddWithGroup([AddGroup({"tag": "pre"}, [AddSkip(1), AddChars(" ")]), AddGroup({"tag": "pre"}, [AddChars("L"), AddGroup({"tag": "caret", "client": "b"}, []), AddChars(" "), AddSkip(1), AddWithGroup([]), AddSkip(8)])]), AddWithGroup([AddSkip(1)]), AddWithGroup([AddSkip(1), AddWithGroup([])]), AddSkip(1)]);

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

    // println!("DOC\n{:?}\n\n", doc);

    // let mut r = OT::apply(&Doc(doc.clone()), &pending);
    // r = OT::apply(&r, &local);

    // println!("HELP\n{:?}\n\n", r);

    // println!("----> {:?}", OT::compose(&pending, &local));
    // OT::apply(&Doc(doc.clone()), &OT::compose(&pending, &local));

    // println!("lol");
}
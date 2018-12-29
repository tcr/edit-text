//! ```cargo
//! [dependencies]
//! oatie = { path = "/Users/trim/tcr/edit-text/oatie" }
//! failure = "*"
//! ```

#[macro_use]
extern crate oatie;
extern crate failure;

use oatie::*;
use oatie::doc::*;
use oatie::schema::RtfSchema;
use oatie::validate::validate_doc;
use failure::Error;

fn main() {
    run().unwrap();
}

fn run() -> Result<(), Error> {

    let doc = Doc(doc_span![
        DocGroup({"tag": "h2"}, [
            DocText("12")
        ]),
        DocGroup({"tag": "p"}, [
            DocText("1"),
        ]),
    ]);

    let a = op_span!([
        DelGroup([
            DelSkip(2)
        ]),
        DelWithGroup([
            DelText(1),
        ]),
    ],
    [
        AddGroup({"tag": "h3"}, [
            AddGroup({"tag": "caret", "client": "d"}, []),
        ]),
        AddGroup({"tag": "h3"}, [
            AddSkip(2),
        ]),
        AddWithGroup([
            AddText("6"),
        ]),
    ]);

    let b = op_span!([
        DelGroup([
            DelSkip(2),
        ]),
        DelGroup([
            DelSkip(1)
        ])
    ],
    [
        AddGroup({"tag": "h2"}, [
            AddSkip(3)
        ])
    ]);

    let (a_, b_) = Op::transform::<RtfSchema>(&a, &b);

    let mut doc_a = Op::apply(&doc, &a);
    doc_a = Op::apply(&doc_a, &a_);
    validate_doc(&doc_a)?;

    doc_a = Op::apply(&doc, &Op::compose(&a, &a_));
    validate_doc(&doc_a)?;

    let mut doc_b = Op::apply(&doc, &b);
    doc_b = Op::apply(&doc_b, &b_);
    validate_doc(&doc_b)?;
    doc_b = Op::apply(&doc, &Op::compose(&b, &b_));
    validate_doc(&doc_b)?;










let local_op = op_span!([
    DelWithGroup([
        DelText(1),
    ]),
], [
    AddGroup({"tag": "bullet"}, [
        AddWithGroup([
            AddGroup({"tag": "caret", "client": "b"}, [])
        ])
    ]),
]);


let input_transform = op_span!([
    DelGroup([
        DelSkip(1),
    ]),
], [
    AddGroup({"tag": "h3"}, [
        AddText("5"),
        AddSkip(1),
    ]),
]);

    // P' x L -> P'', L'
    let (local_transform, _) = Op::transform::<RtfSchema>(&input_transform, &local_op);

    println!("----> {:?}", local_transform);
        


    // // client_doc = input_doc : I' : P''
    // let client_op = Op::compose(&pending_op_transform, &local_op_transform);
    // // Reattach to doc.
    // self.doc = Op::apply(&new_doc, &pending_op_transform);
    // validate_doc(&self.doc).expect("Validation error after unrelated pending op");
    // self.doc = Op::apply(&self.doc, &local_op_transform);



    Ok(())
}









/*
fn main() {
// let pending_op = op_span!([DelWithGroup([DelGroup([DelSkip(2)]), DelGroup([DelSkip(11), DelWithGroup([]), DelSkip(1)])]), DelGroup([DelWithGroup([DelText(1), DelGroup([]), DelSkip(1)]), DelWithGroup([DelSkip(1)])]), DelWithGroup([DelSkip(1), DelWithGroup([])]), DelSkip(1)], [AddWithGroup([AddGroup({"tag": "h3"}, [AddText(" ")]), AddGroup({"tag": "p"}, [AddText("L"), AddGroup({"tag": "caret", "client": "b"}, []), AddText(" "), AddSkip(13), AddWithGroup([]), AddSkip(1)])]), AddWithGroup([AddSkip(1), AddText("b")]), AddWithGroup([AddSkip(1)]), AddWithGroup([AddSkip(1), AddWithGroup([])]), AddSkip(1)]);

// let local_op = op_span!([DelWithGroup([DelGroup([DelText(1)]), DelGroup([DelText(1), DelGroup([]), DelSkip(14), DelWithGroup([]), DelSkip(1)])]), DelWithGroup([DelSkip(2)]), DelWithGroup([DelSkip(1)]), DelSkip(2)], [AddWithGroup([AddGroup({"tag": "h3"}, [AddText("C"), AddSkip(14), AddWithGroup([]), AddSkip(1), AddText("p")])]), AddGroup({"tag": "bullet"}, [AddWithGroup([AddGroup({"client": "b", "tag": "caret"}, []), AddSkip(1), AddText("2"), AddSkip(1)])]), AddWithGroup([AddSkip(1)]), AddSkip(2)]);

// let input_op = op_span!([DelWithGroup([DelGroup([DelSkip(2)]), DelGroup([DelSkip(2), DelText(2), DelSkip(2), DelText(2), DelSkip(1), DelText(2), DelGroup([]), DelText(1)])]), DelWithGroup([DelGroup([DelText(1), DelGroup([]), DelText(1)]), DelWithGroup([DelSkip(1)])]), DelWithGroup([DelSkip(1), DelWithGroup([])]), DelSkip(1)], [AddWithGroup([AddGroup({"tag": "pre"}, [AddText("A"), AddSkip(1), AddGroup({"tag": "caret", "client": "a"}, []), AddSkip(6), AddText("i ")])]), AddWithGroup([AddWithGroup([AddSkip(1)])]), AddWithGroup([AddSkip(1), AddWithGroup([])]), AddSkip(1)]);

// let doc = Doc(doc_span![DocGroup({"tag": "bullet"}, [DocGroup({"tag": "pre"}, [DocText("Av"), DocGroup({"client": "a", "tag": "caret"}, []), DocText("B20 W7i ")])]), DocGroup({"tag": "bullet"}, [DocGroup({"tag": "h3"}, [DocText("2")])]), DocGroup({"tag": "h3"}, [DocText("8"), DocGroup({"tag": "caret", "client": "c"}, [])]), DocGroup({"tag": "h1"}, [DocText("0pos Mercutio, a rich text editor.")])]);



//     // I x P -> I', P'
//     let (pending_op_transform, input_op_transform) = Op::transform::<RtfSchema>(&input_op, &pending_op);
//     // P' x L -> P'', L'


    let input_op_transform = op_span!([
    DelGroup([
        DelText(1)
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


// let pending = op_span!([DelWithGroup([DelGroup([DelSkip(2), DelWithGroup([]), DelSkip(8)])]), DelGroup([DelWithGroup([DelSkip(1)])]), DelWithGroup([DelSkip(1), DelWithGroup([])]), DelSkip(1)], [AddWithGroup([AddGroup({"tag": "pre"}, [AddSkip(1), AddText(" ")]), AddGroup({"tag": "pre"}, [AddText("L"), AddGroup({"tag": "caret", "client": "b"}, []), AddText(" "), AddSkip(1), AddWithGroup([]), AddSkip(8)])]), AddWithGroup([AddSkip(1)]), AddWithGroup([AddSkip(1), AddWithGroup([])]), AddSkip(1)]);

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

    // let mut r = Op::apply(&Doc(doc.clone()), &pending);
    // r = Op::apply(&r, &local);

    // println!("HELP\n{:?}\n\n", r);

    // println!("----> {:?}", Op::compose(&pending, &local));
    // Op::apply(&Doc(doc.clone()), &Op::compose(&pending, &local));

    // println!("lol");
}
*/

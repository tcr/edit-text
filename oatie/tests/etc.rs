
// #[test]
// fn test_multiple_compose_1() {
//     println!("yeah");

//     let doc = doc_span![DocGroup({"tag": "ul"}, [DocGroup({"tag": "li"}, [DocGroup({"tag": "h1"}, [DocChars("Hello! "), DocGroup({"tag": "b"}, [DocChars("what\'s")]), DocChars(" up?")]), DocGroup({"tag": "p"}, [DocChars("World!")])])])];

//     let mut ops: Vec<Op> = vec![
//         op_span!(
//             [DelWithGroup([DelWithGroup([DelWithGroup([DelSkip(6), DelChars(1)])])])],
//             [],
//         ),
//         op_span!(
//             [DelWithGroup([DelWithGroup([DelGroup([DelSkip(11)])])])],
//             [AddWithGroup([AddWithGroup([AddGroup({"tag": "h1"}, [AddSkip(6)]), AddGroup({"tag": "p"}, [AddSkip(5)])])])],
//         ),
//         op_span!(
//             [],
//             [AddWithGroup([AddWithGroup([AddSkip(1), AddWithGroup([AddWithGroup([AddSkip(1), AddChars("W")])])])])],
//         ),
//         op_span!(
//             [DelWithGroup([DelWithGroup([DelSkip(1), DelWithGroup([DelWithGroup([DelChars(1)])])])])],
//             [],
//         ),
//     ];

//     let mut op = op_span!([], []);
//     for i in ops.into_iter() {

//         println!("compose: op_span!(");
//         println!("  {:?},", i.0);
//         println!("  {:?},", i.1);
//         println!(")");

//         op = compose(&op, &i);

//         println!("applying: op_span!(");
//         println!("  {:?},", op.0);
//         println!("  {:?},", op.1);
//         println!(")");
//         let out = apply_operation(&doc, &op);

//         println!("doc: {:?}", out);
//     }

// // CMP add [DelWithGroup([DelWithGroup([DelGroup([DelWithGroup([DelChars(1), DelSkip(1)]), DelSkip(5), DelChars(1), DelSkip(5)])])])]
// //     del [AddWithGroup([AddWithGroup([AddGroup({"tag": "h1"}, [AddSkip(6)]), AddGroup({"tag": "p"}, [AddWithGroup([AddChars("W")]), AddSkip(4)])])])]
// // start obj [DocGroup({"tag": "ul"}, [DocGroup({"tag": "li"}, [DocGroup({"tag": "h1"}, [DocChars("Hello! "), DocGroup({"tag": "b"}, [DocChars("what\'s")]), DocChars(" up?")]), DocGroup({"tag": "p"}, [DocChars("World!")])])])]
// }

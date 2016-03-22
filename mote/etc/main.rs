#![feature(trace_macros)]

use std::collections::HashMap;

macro_rules! doc_span {
    (@str_literal $e:expr) => { $e };
    ( @kind DocChars $b:expr $(,)* ) => {
        DocChars($b.to_owned())
    };
    ( @kind DocGroup { $( $e:tt : $b:expr ),+  $(,)* } , [ $( $v:tt )* ] $(,)* ) => {
        {
            let mut map = ::std::collections::HashMap::<String, String>::new();
            $( map.insert(doc_span!(@str_literal $e).to_owned(), ($b).to_owned()); )*
            DocGroup(map, doc_span![ $( $v )* ]) 
        }
    };
    ( $( $i:ident ( $( $b:tt )+ ) ),+ $(,)* ) => {
        vec![
            $( doc_span!(@kind $i $( $b )* , ) ),*
        ]
    };
}

#[derive(Debug)]
enum DocElement {
    DocGroup(HashMap<String, String>, Vec<DocElement>),
    DocChars(String),
}

fn main () {
    use DocElement::*;

    trace_macros!(true);
        let span = doc_span![DocGroup({"tag": "ul"}, [DocGroup({"tag": "li"}, [DocGroup({"tag": "p"}, [DocChars("Hello!")]), DocGroup({"tag": "p"}, [DocChars("World!")])])])];
    // let span = doc_span![DocGroup({"tag": "ul"}, ["b"])];

    println!("span: {:?}", span);
}

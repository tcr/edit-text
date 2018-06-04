//! Macros for more easily creating operations and documents in code.

#[macro_export]
macro_rules! doc_span {
    ( @str_literal $e:expr ) => { $e };
    ( @kind DocChars $b:expr $(,)* ) => {
        DocChars($crate::doc::DocString::from_str($b))
    };
    ( @kind DocChars $b:expr , { $( $e:expr => $c:expr ),+  $(,)* } $(,)* ) => {
        {
            let mut map = ::std::collections::HashMap::<Style, Option<String>>::new();
            $(
                map.insert($e, $c);
            )*
            DocChars($crate::doc::DocString::from_str_styled($b, map))
        }
    };
    ( @kind DocGroup { $( $e:tt : $b:expr ),+  $(,)* } , [ $( $v:tt )* ] $(,)* ) => {
        {
            let mut map = ::std::collections::HashMap::<String, String>::new();
            $(
                map.insert(doc_span!(@str_literal $e).to_owned(), ($b).to_owned());
            )*
            DocGroup(map, doc_span![ $( $v )* ])
        }
    };
    ( ) => {
        vec![]
    };
    ( $( $i:ident ( $( $b:tt )+ ) ),+ $(,)* ) => {
        vec![
            $( doc_span!(@kind $i $( $b )* , ) ),*
        ]
    };
}

#[macro_export]
macro_rules! add_span {
    ( @str_literal $e:expr ) => { $e };
    ( @kind AddSkip $b:expr $(,)* ) => {
        AddSkip($b)
    };
    ( @kind AddChars $b:expr , { $( $e:expr => $c:expr ),+  $(,)* } $(,)* ) => {
        {
            let mut map = ::std::collections::HashMap::<Style, Option<String>>::new();
            $(
                map.insert($e, $c);
            )*
            AddChars($crate::doc::DocString::from_str_styled($b, map))
        }
    };
    ( @kind AddChars $b:expr $(,)* ) => {
        AddChars($crate::doc::DocString::from_str($b))
    };
    ( @kind AddWithGroup [ $( $v:tt )* ] $(,)* ) => {
        AddWithGroup(add_span![ $( $v )* ])
    };
    ( @kind AddGroup { $( $e:tt : $b:expr ),+  $(,)* } , [ $( $v:tt )* ] $(,)* ) => {
        {
            let mut map = ::std::collections::HashMap::<String, String>::new();
            $(
                map.insert(add_span!(@str_literal $e).to_owned(), ($b).to_owned());
            )*
            AddGroup(map, add_span![ $( $v )* ])
        }
    };
    ( ) => {
        vec![]
    };
    ( $( $i:ident ( $( $b:tt )+ ) ),+ $(,)* ) => {
        vec![
            $( add_span!(@kind $i $( $b )* , ) ),*
        ]
    };
}

#[macro_export]
macro_rules! del_span {
    ( @str_literal $e:expr ) => { $e };
    ( @kind DelSkip $b:expr $(,)* ) => {
        DelSkip($b)
    };
    ( @kind DelChars $b:expr $(,)* ) => {
        DelChars($b.to_owned())
    };
    ( @kind DelWithGroup [ $( $v:tt )* ] $(,)* ) => {
        DelWithGroup(del_span![ $( $v )* ])
    };
    ( @kind DelGroup [ $( $v:tt )* ] $(,)* ) => {
        DelGroup(del_span![ $( $v )* ])
    };
    ( @kind DelGroupAll $(,)* ) => {
        DelGroupAll
    };
    ( ) => {
        vec![]
    };
    ( $( $i:ident ( $( $b:tt )* ) ),+ $(,)* ) => {
        vec![
            $( del_span!(@kind $i $( $b )* , ) ),*
        ]
    };
    ( $( $i:ident ),+ $(,)* ) => {
        vec![
            $( del_span!(@kind $i , ) ),*
        ]
    };
}

#[macro_export]
macro_rules! op_span {
    ( [ $( $d:tt )* ], [ $( $a:tt )* ] $(,)* ) => {
        (
            del_span![ $( $d )* ],
            add_span![ $( $a )* ],
        )
    };
}

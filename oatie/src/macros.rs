//! Macros for more easily creating operations and documents in code.

#[macro_export]
macro_rules! doc_span {
    ( @str_literal $e:expr ) => { $e };
    ( @kind DocText $b:expr $(,)* ) => {
        DocText($crate::rtf::StyleSet::new(), $crate::doc::DocString::from_str($b))
    };
    ( @kind DocText { $( $e:expr ),+  $(,)* } , $b:expr $(,)* ) => {
        {
            let mut map = ::std::collections::HashSet::new();
            $(
                map.insert($e);
            )*
            DocText($crate::rtf::StyleSet::from(map), $crate::doc::DocString::from_str($b))
        }
    };
    ( @kind DocGroup $b:expr , [ $( $v:tt )* ] $(,)* ) => {
        {
            DocGroup($b, doc_span![ $( $v )* ])
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
    ( @kind AddText { $( $e:expr => $c:expr ),+ , $b:expr  $(,)* } $(,)* ) => {
        {
            let mut map = ::std::collections::HashSet();
            $(
                map.insert($e, $c);
            )*
            AddText($crate::rtf::StyleSet::from(map), $crate::doc::DocString::from_str($b))
        }
    };
    ( @kind AddText $b:expr $(,)* ) => {
        AddText($crate::rtf::StyleSet::new(), $crate::doc::DocString::from_str($b))
    };
    ( @kind AddWithGroup [ $( $v:tt )* ] $(,)* ) => {
        AddWithGroup(add_span![ $( $v )* ])
    };
    ( @kind AddGroup $b:expr , [ $( $v:tt )* ] $(,)* ) => {
        {
            AddGroup($b, add_span![ $( $v )* ])
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
    ( @kind DelText $b:expr $(,)* ) => {
        DelText($b.to_owned())
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

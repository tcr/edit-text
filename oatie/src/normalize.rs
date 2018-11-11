use super::doc::*;
use std::collections::HashMap;

fn normalize_add_element(elem: AddElement) -> AddElement {
    match elem {
        AddGroup(attrs, span) => {
            let span = normalize_add_span(span, false);
            AddGroup(attrs, span)
        }
        AddWithGroup(span) => {
            let span = normalize_add_span(span, true);

            // Shortcut if the inner span is nothing but skips
            if span.is_empty() {
                AddSkip(1)
            } else {
                AddWithGroup(span)
            }
        }
        _ => elem,
    }
}

fn normalize_add_span(add: AddSpan, trim_last: bool) -> AddSpan {
    let mut ret: AddSpan = vec![];
    for elem in add.into_iter() {
        ret.place(&normalize_add_element(elem));
    }
    if trim_last {
        if let Some(&AddSkip(..)) = ret.last() {
            ret.pop();
        }
    }
    ret
}

fn normalize_del_element(elem: DelElement) -> DelElement {
    match elem {
        DelGroup(span) => {
            let span = normalize_del_span(span, false);
            DelGroup(span)
        }
        DelWithGroup(span) => {
            let span = normalize_del_span(span, true);

            // Shortcut if the inner span is nothing but skips
            if span.is_empty() {
                DelSkip(1)
            } else {
                DelWithGroup(span)
            }
        }
        _ => elem,
    }
}

fn normalize_del_span(del: DelSpan, trim_last: bool) -> DelSpan {
    let mut ret: DelSpan = vec![];
    for elem in del.into_iter() {
        ret.place(&normalize_del_element(elem));
    }
    if trim_last {
        if let Some(&DelSkip(..)) = ret.last() {
            ret.pop();
        }
    }
    ret
}

pub fn normalize(op: Op) -> Op {
    // TODO all
    (
        normalize_del_span(op.0, true),
        normalize_add_span(op.1, true),
    )
}

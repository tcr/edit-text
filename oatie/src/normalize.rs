use crate::doc::*;

fn normalize_add_element<S: Schema>(elem: AddElement<S>) -> AddElement<S> {
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

fn normalize_add_span<S: Schema>(add: AddSpan<S>, trim_last: bool) -> AddSpan<S> {
    let mut ret: AddSpan<S> = vec![];
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

fn normalize_del_element<S: Schema>(elem: DelElement<S>) -> DelElement<S> {
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

fn normalize_del_span<S: Schema>(del: DelSpan<S>, trim_last: bool) -> DelSpan<S> {
    let mut ret: DelSpan<S> = vec![];
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

pub fn normalize<S: Schema>(op: Op<S>) -> Op<S> {
    // TODO all
    (
        normalize_del_span(op.0, true),
        normalize_add_span(op.1, true),
    )
}

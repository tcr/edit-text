//! Methods to apply an operation to a document.

use super::doc::*;
use crate::stepper::*;

fn apply_add_inner<M: DocMutator<S>, S: Schema>(
    bc: &mut M,
    spanvec: &DocSpan<S>,
    addvec: &AddSpan<S>,
) -> (DocSpan<S>, DocSpan<S>) {
    let mut span = &spanvec[..];
    let mut del = &addvec[..];

    let mut first = None;
    if !span.is_empty() {
        first = Some(span[0].clone());
        span = &span[1..]
    }

    let mut res: DocSpan<S> = Vec::with_capacity(span.len());

    if del.is_empty() {
        return (vec![], spanvec.clone().to_vec());
    }

    let mut d = del[0].clone();
    del = &del[1..];

    let mut exhausted = first.is_none();

    trace!("ABOUT TO APPLY ADD {:?} {:?}", first, span);

    loop {
        // Flags for whether we have partially or fully consumed an element.
        let mut nextdel = true;
        let mut nextfirst = true;

        if exhausted {
            match d {
                AddSkip(..) | AddWithGroup(..) => {
                    panic!("exhausted document on {:?}", d);
                }
                _ => {}
            }
        }

        trace!("next {:?} {:?} {:?}", d, first, exhausted);

        match d.clone() {
            AddStyles(count, styles) => match first.clone().unwrap() {
                DocChars(mut chars_styles, value) => {
                    if value.char_len() < count {
                        d = AddStyles(count - value.char_len(), styles.clone());
                        chars_styles.extend(&styles);
                        bc.delete(1);
                        bc.InsertDocString(value.clone(), styles.clone());
                        // partial = false;
                        res.place(&DocChars(chars_styles, value));
                        nextdel = false;
                    } else if value.char_len() > count {
                        let (left, right) = value.split_at(count);
                        let mut left_styles = chars_styles.clone();
                        left_styles.extend(&styles);
                        bc.delete(1);
                        bc.InsertDocString(left.clone(), styles.clone());
                        // partial = false;
                        res.place(&DocChars(left_styles, left));
                        first = Some(DocChars(chars_styles, right));
                        nextfirst = false;
                    } else {
                        chars_styles.extend(&styles);
                        bc.delete(1);
                        bc.InsertDocString(value.clone(), styles.clone());
                        // partial = false;
                        res.place(&DocChars(chars_styles, value));
                    }
                }
                DocGroup(..) => {
                    panic!("Invalid AddStyles");
                }
            },
            AddSkip(count) => match first.clone().unwrap() {
                DocChars(chars_styles, value) => {
                    if value.char_len() < count {
                        // Consume and advance
                        d = AddSkip(count - value.char_len());
                        bc.AdvanceElements(1);
                        res.place(&DocChars(chars_styles, value));
                        nextdel = false;
                    } else if value.char_len() > count {
                        let (left, right) = value.split_at(count);
                        // Split text element, we assume
                        bc.skip(count);
                        res.place(&DocChars(chars_styles.clone(), left));
                        first = Some(DocChars(chars_styles, right));
                        nextfirst = false;
                    } else {
                        bc.AdvanceElements(1);
                        res.place(&DocChars(chars_styles, value));
                    }
                }
                DocGroup(..) => {
                    res.push(first.clone().unwrap());
                    bc.AdvanceElements(1);
                    if count > 1 {
                        d = AddSkip(count - 1);
                        nextdel = false;
                    }
                }
            },
            AddWithGroup(ref delspan) => match first.clone().unwrap() {
                DocGroup(ref attrs, ref span) => {
                    bc.Enter();
                    res.push(DocGroup(attrs.clone(), apply_add_outer(bc, span, delspan)));
                    bc.Exit();
                }
                _ => {
                    panic!("Invalid AddWithGroup");
                }
            },
            AddChars(styles, value) => {
                // TODO where do you skip anything, exactly
                // need to manifest the place issue externally as well
                bc.InsertDocString(value.clone(), styles.clone());
                res.place(&DocChars(styles, value));
                nextfirst = false;
            }
            AddGroup(attrs, innerspan) => {
                let mut subdoc = vec![];
                if !exhausted {
                    subdoc.push(first.clone().unwrap());
                    subdoc.extend_from_slice(span);
                }
                trace!("CALLING INNER {:?} {:?}", subdoc, innerspan);

                // Apply the inner AddSpan inside the group...
                let (inner, rest) = apply_add_inner(bc, &subdoc, &innerspan);
                res.place(&DocGroup(attrs.clone(), inner));

                // console_log!("partial A {:?}", partial);
                // console_log!("partial B {:?}", partial_inner);

                trace!("REST OF INNER {:?} {:?}", rest, del);

                // TODO not hardcode a random number.
                // Wrap previous elements in the inner span.
                bc.WrapPrevious(0, attrs);

                // Then apply it outside of the group.
                //TODO partial inner should be... something else
                let (inner, rest) = apply_add_inner(bc, &rest, &del.to_vec());
                // console_log!("partial B {:?} {:?}", inner, rest);
                res.place_all(&inner);
                // console_log!("partial C {:?}", partial);

                return (res, rest);
            }
        }

        if nextdel {
            if del.is_empty() {
                let mut remaining = vec![];
                trace!("nextfirst {:?} {:?} {:?}", nextfirst, first, exhausted);
                if !nextfirst && first.is_some() && !exhausted {
                    remaining.push(first.clone().unwrap());
                    // place_any(&mut res, &first.clone().unwrap());
                }
                remaining.extend_from_slice(span);
                return (res, remaining);
            }

            d = del[0].clone();
            del = &del[1..];
        }

        if nextfirst {
            if span.is_empty() {
                exhausted = true;
            } else {
                first = Some(span[0].clone());
                span = &span[1..];
            }
        }
    }
}

// TODO replace all occurances of this with apply_add_inner
fn apply_add_outer<M: DocMutator<S>, S: Schema>(
    bc: &mut M,
    spanvec: &DocSpan<S>,
    addvec: &AddSpan<S>,
) -> DocSpan<S> {
    let (mut res, remaining) = apply_add_inner(bc, spanvec, addvec);

    // TODO never accept unbalanced components?
    if !remaining.is_empty() {
        // if partial {
        //     let text = remaining.remove(0);
        //     bc.DeleteElements(1));
        //     match text {
        //         DocChars(text) => {
        //             console_log!("adding {:?}", text);
        //             bc.InsertDocString(text);
        //         }
        //         _ => unreachable!(),
        //     }
        // }
        res.place_all(&remaining);
        // panic!("Unbalanced apply_add");
    }
    res
}

pub fn apply_add<S: Schema>(
    spanvec: &DocSpan<S>,
    add: &AddSpan<S>,
) -> DocSpan<S> {
    let mut mutator = NullDocMutator {};
    let ret = apply_add_outer(&mut mutator, spanvec, add);
    ret
}

// TODO what does this do, why doe sit exist, for creating BC for frontend??
pub fn apply_add_bc<S: Schema>(
    spanvec: &DocSpan<S>,
    addvec: &AddSpan<S>,
) -> (Doc<S>, Program) {
    let mut mutator = RecordingDocMutator::new(DocStepper::new(spanvec));
    let output_doc = apply_add_outer(&mut mutator, spanvec, addvec);

    // Compare results.
    // let actual = ret.clone();
    let (_compare, bc) = mutator.result().unwrap();
    // if actual != compare {
    //     console_log!("\n\n\n✅✅✅ ADDITION: {:?}", add);
    //     for item in &bc.0 {
    //         console_log!("      -> {:?}", item);
    //     }
    //     console_log!("\ntest =====> [ {} ]\n\nactual:\n  {:?}\n\ncompare:\n  {:?}\n\n", actual == compare, actual, compare);
    // }

    (Doc(output_doc), bc)
}

fn apply_del_inner<M: DocMutator<S>, S: Schema>(
    bc: &mut M,
    spanvec: &DocSpan<S>,
    addvec: &DelSpan<S>,
) -> DocSpan<S> {
    let mut span = &spanvec[..];
    let mut del = &addvec[..];

    let mut res: DocSpan<S> = Vec::with_capacity(span.len());

    if del.is_empty() {
        return span.to_vec();
    }

    let mut first = span[0].clone();
    span = &span[1..];

    let mut d = del[0].clone();
    del = &del[1..];

    loop {
        let mut nextdel = true;
        let mut nextfirst = true;

        // println!("(d) del: {:?}\n    doc: {:?}", d, first);

        match d.clone() {
            DelStyles(count, styles) => match first.clone() {
                DocChars(mut chars_styles, value) => {
                    if value.char_len() < count {
                        d = DelStyles(count - value.char_len(), styles.clone());
                        chars_styles.remove(&styles);
                        bc.delete(1);
                        bc.InsertDocString(value.clone(), chars_styles.clone());
                        res.place(&DocChars(chars_styles, value));
                        nextdel = false;
                    } else if value.char_len() > count {
                        let (left, right) = value.split_at(count);
                        let mut left_styles = chars_styles.clone();
                        left_styles.remove(&styles);
                        bc.delete(1);
                        bc.InsertDocString(left.clone(), chars_styles.clone());
                        res.place(&DocChars(chars_styles.clone(), left));
                        first = DocChars(chars_styles, right);
                        nextfirst = false;
                    } else {
                        chars_styles.remove(&styles);
                        bc.delete(1);
                        bc.InsertDocString(value.clone(), chars_styles.clone());
                        res.place(&DocChars(chars_styles, value));
                    }
                }
                _ => {
                    panic!("Invalid DelStyles");
                }
            },
            DelSkip(count) => match first.clone() {
                DocChars(chars_styles, value) => {
                    if value.char_len() < count {
                        d = DelSkip(count - value.char_len());
                        bc.AdvanceElements(1);
                        res.place(&DocChars(chars_styles, value));
                        nextdel = false;
                    } else if value.char_len() > count {
                        let (left, right) = value.split_at(count);
                        // Assume this should be deleted from the left
                        bc.skip(count);
                        res.place(&DocChars(chars_styles.clone(), left));
                        first = DocChars(chars_styles, right);
                        nextfirst = false;
                    } else {
                        bc.AdvanceElements(1);
                        res.place(&DocChars(chars_styles, value));
                        nextdel = true;
                    }
                }
                DocGroup(..) => {
                    res.push(first.clone());
                    bc.AdvanceElements(1);
                    if count > 1 {
                        d = DelSkip(count - 1);
                        nextdel = false;
                    }
                }
            },
            DelWithGroup(ref delspan) => match first.clone() {
                DocGroup(ref attrs, ref span) => {
                    bc.Enter();
                    res.push(DocGroup(attrs.clone(), apply_del_inner(bc, span, delspan)));
                    bc.Exit();
                }
                _ => {
                    panic!("Invalid DelWithGroup");
                }
            },
            DelGroup(ref delspan) => match first.clone() {
                DocGroup(_, ref span) => {
                    bc.Enter();
                    res.place_all(&apply_del_inner(bc, span, delspan)[..]);
                    bc.UnwrapSelf();
                }
                _ => {
                    panic!("Invalid DelGroup");
                }
            },
            DelChars(count) => match first.clone() {
                DocChars(chars_styles, value) => {
                    if value.char_len() > count {
                        let (_, right) = value.split_at(count);
                        first = DocChars(chars_styles, right);
                        nextfirst = false;
                    } else if value.char_len() < count {
                        d = DelChars(count - value.char_len());
                        nextdel = false;
                    } else {
                        // noop
                        bc.DeleteElements(1);
                    }
                }
                _ => {
                    panic!("Invalid DelChars");
                }
            },
        }

        if nextdel {
            if del.is_empty() {
                if !nextfirst {
                    res.place(&first)
                    // TODO res place
                }
                if !span.is_empty() {
                    res.place(&span[0]);
                    res.extend_from_slice(&span[1..]);
                }
                break;
            }

            d = del[0].clone();
            del = &del[1..];
        }

        if nextfirst {
            if span.is_empty() {
                panic!(
                    "exhausted document in apply_delete\n -->{:?}\n -->{:?}",
                    first, span
                );
            }

            first = span[0].clone();
            span = &span[1..];
        }
    }

    res
}

pub fn apply_delete<S: Schema>(
    spanvec: &DocSpan<S>,
    delvec: &DelSpan<S>,
) -> DocSpan<S> {
    let mut mutator = NullDocMutator {};
    let ret = apply_del_inner(&mut mutator, spanvec, delvec);
    ret
}

// TODO what does this do, why doe sit exist, for creating BC for frontend??
pub fn apply_del_bc<S: Schema>(
    spanvec: &DocSpan<S>,
    del: &DelSpan<S>,
) -> (DocSpan<S>, Program) {
    let mut mutator = RecordingDocMutator::new(DocStepper::new(spanvec));
    let output_doc = apply_del_inner(&mut mutator, spanvec, del);

    let (compare, bc) = mutator.result().unwrap();

    // Compare results.
    if cfg!(feature = "validate_bytecode") {
        let actual = output_doc.clone();
        if actual != compare {
            console_log!("\n\n\n🚫🚫🚫 DELETION: {:?}", del);
            for _item in &bc.0 {
                console_log!("      -> {:?}", _item);
            }
            console_log!(
                "\ntest =====> [ {} ]\n\nactual:\n  {:?}\n\ncompare:\n  {:?}\n\n",
                actual == compare,
                actual,
                compare
            );
        }
    }

    (output_doc, bc)
}

pub fn apply_op_bc<S: Schema>(
    spanvec: &DocSpan<S>,
    op: &Op<S>,
) -> Vec<Program> {
    // console_log!("\n\n\n\n\naaaaaaa\n\n\n\n\n\n\n");
    let &(ref delvec, ref addvec) = op;
    // console_log!("👻👻  1  👻👻");
    let (postdel, del_program) = apply_del_bc(spanvec, delvec);
    // console_log!("👻👻  2  👻👻");
    let (_postadd, add_program) = apply_add_bc(&postdel, addvec);
    // console_log!("👻👻  3  👻👻");
    // console_log!("👻👻👻👻👻 {:?}", del_program);
    vec![del_program, add_program]
}

pub fn apply_operation<S: Schema>(
    spanvec: &DocSpan<S>,
    op: &Op<S>,
) -> DocSpan<S> {
    let &(ref delvec, ref addvec) = op;
    // println!("------> @1 {:?}", spanvec);
    // println!("------> @2 {:?}", delvec);
    let postdel = apply_delete(spanvec, delvec);
    // println!("------> @3 {:?}", postdel);
    // println!("------> @4 {:?}", addvec);
    apply_add(&postdel, addvec)
}

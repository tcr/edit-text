//! Composes two operations together.

use super::doc::*;
use std::cmp;
use crate::stepper::*;

fn compose_del_del_inner(res: &mut DelSpan, a: &mut DelStepper, b: &mut DelStepper) {
    while !a.is_done() && !b.is_done() {
        match a.get_head() {
            DelSkip(acount) => {
                match b.head.clone() {
                    Some(DelSkip(bcount)) => {
                        res.place(&DelSkip(cmp::min(acount, bcount)));
                        if acount > bcount {
                            a.head = Some(DelSkip(acount - bcount));
                            b.next();
                        } else if acount < bcount {
                            b.head = Some(DelSkip(bcount - acount));
                            a.next();
                        } else {
                            a.next();
                            b.next();
                        }
                    }
                    // Some(DelObject) |
                    Some(DelWithGroup(..)) | Some(DelGroup(..)) => {
                        if acount > 1 {
                            a.head = Some(DelSkip(acount - 1));
                        } else {
                            a.next();
                        }
                        res.place(&b.next().unwrap());
                    }
                    Some(DelChars(bcount)) => {
                        res.place(&DelChars(cmp::min(acount, bcount)));
                        if acount > bcount {
                            a.head = Some(DelSkip(acount - bcount));
                            b.next();
                        } else if acount < bcount {
                            b.head = Some(DelChars(bcount - acount));
                            a.next();
                        } else {
                            a.next();
                            b.next();
                        }
                    }
                    Some(DelStyles(b_count, b_styles)) => {
                        res.place(&DelChars(cmp::min(acount, b_count)));
                        if acount > b_count {
                            a.head = Some(DelSkip(acount - b_count));
                            b.next();
                        } else if acount < b_count {
                            b.head = Some(DelStyles(b_count - acount, b_styles));
                            a.next();
                        } else {
                            a.next();
                            b.next();
                        }
                    }
                    None => {
                        res.place(&a.next().unwrap());
                    } // Some(DelMany(bcount)) => {
                      //     res.place(&DelMany(cmp::min(acount, bcount)));
                      //     if acount > bcount {
                      //         a.head = Some(DelSkip(acount - bcount));
                      //         b.next();
                      //     } else if acount < bcount {
                      //         b.head = Some(DelMany(bcount - acount));
                      //         a.next();
                      //     } else {
                      //         a.next();
                      //         b.next();
                      //     }
                      // }
                      // Some(DelGroupAll) => {
                      //     if acount > 1 {
                      //         a.head = Some(DelSkip(acount - 1));
                      //     } else {
                      //         a.next();
                      //     }
                      //     res.place(&b.next().unwrap());
                      // }
                }
            }
            DelStyles(a_count, a_styles) => match b.head.clone() {
                Some(DelStyles(b_count, b_styles)) => {
                    let mut both_styles = b_styles.clone();
                    both_styles.extend(a_styles.clone());
                    res.push(DelStyles(cmp::min(a_count, b_count), both_styles));
                    if a_count > b_count {
                        b.head = Some(DelStyles(a_count - b_count, a_styles));
                        b.next();
                    } else if a_count < b_count {
                        a.head = Some(DelStyles(b_count - a_count, b_styles));
                        a.next();
                    } else {
                        a.next();
                        b.next();
                    }
                }
                Some(DelSkip(b_count)) => {
                    res.push(DelStyles(cmp::min(a_count, b_count), a_styles.clone()));
                    if a_count > b_count {
                        b.head = Some(DelSkip(a_count - b_count));
                        b.next();
                    } else if a_count < b_count {
                        a.head = Some(DelStyles(b_count - a_count, a_styles));
                        a.next();
                    } else {
                        a.next();
                        b.next();
                    }
                }
                Some(DelWithGroup(..)) | Some(DelGroup(..)) => {
                    unreachable!();
                }
                Some(DelChars(b_count)) => {
                    res.place(&DelChars(cmp::min(a_count, b_count)));
                    if a_count > b_count {
                        a.head = Some(DelStyles(a_count - b_count, a_styles));
                        b.next();
                    } else if a_count < b_count {
                        b.head = Some(DelChars(b_count - a_count));
                        a.next();
                    } else {
                        a.next();
                        b.next();
                    }
                }
                None => {
                    res.place(&a.next().unwrap());
                }
            },
            DelWithGroup(ref span) => {
                match b.head.clone() {
                    Some(DelSkip(bcount)) => {
                        if bcount > 1 {
                            b.head = Some(DelSkip(bcount - 1));
                        } else {
                            b.next();
                        }
                        res.place(&a.next().unwrap());
                    }
                    Some(DelStyles(..)) => {
                        panic!("DelWithGroup vs DelStyles is bad");
                    }
                    Some(DelWithGroup(ref bspan)) => {
                        res.place(&DelWithGroup(compose_del_del(span, bspan)));
                        a.next();
                        b.next();
                    }
                    Some(DelGroup(ref bspan)) => {
                        res.place(&DelGroup(compose_del_del(span, bspan)));
                        a.next();
                        b.next();
                    }
                    Some(DelChars(..)) => {
                        panic!("DelWithGroup vs DelChars is bad");
                    }
                    None => {
                        res.place(&a.next().unwrap());
                    } // Some(DelMany(bcount)) => {
                      //     if bcount > 1 {
                      //         b.head = Some(DelMany(bcount - 1));
                      //     } else {
                      //         b.next();
                      //     }
                      //     a.next();
                      //     res.place(&DelMany(1));
                      // }
                      // Some(DelObject) => {
                      //     panic!("DelWithGroup vs DelObject is bad");
                      // }
                      // Some(DelGroupAll) => {
                      //     a.next();
                      //     res.place(&b.next().unwrap());
                      // }
                }
            }
            DelGroup(ref span) => {
                let mut c = DelStepper::new(span);
                let mut inner: DelSpan = vec![];
                compose_del_del_inner(&mut inner, &mut c, b);
                if !c.is_done() {
                    inner.place(&c.head.unwrap());
                    inner.place_all(&c.rest);
                }
                res.place(&DelGroup(inner));
                a.next();
            }
            DelChars(count) => {
                res.place(&DelChars(count));
                a.next();
            } // DelObject => {
              //     match b.head.clone() {
              //         Some(DelObject) => {
              //             res.place(&DelObject);
              //             a.next();
              //             b.next();
              //         }
              //         None => {
              //             res.place(&DelObject);
              //             a.next();
              //         }
              //         _ => {
              //             panic!("Invalid compose against DelObject");
              //         }
              //     }
              // }
              // DelMany(count) => {
              //     res.place(&DelMany(count));
              //     a.next();
              // }
              // DelGroupAll => {
              //     res.place(&DelGroupAll);
              //     a.next();
              // }
        }
    }
}

pub fn compose_del_del(avec: &DelSpan, bvec: &DelSpan) -> DelSpan {
    let mut res = Vec::with_capacity(avec.len() + bvec.len());

    let mut a = DelStepper::new(avec);
    let mut b = DelStepper::new(bvec);

    compose_del_del_inner(&mut res, &mut a, &mut b);

    if !a.is_done() {
        res.place(&a.get_head());
        res.place_all(&a.rest);
    }

    if !b.is_done() {
        res.place(&b.get_head());
        res.place_all(&b.rest);
    }

    res
}

fn compose_add_add_inner(res: &mut AddSpan, a: &mut AddStepper, b: &mut AddStepper) {
    while !b.is_done() && !a.is_done() {
        match b.get_head() {
            AddChars(..) => {
                res.place(&b.next().unwrap());
            }
            AddStyles(b_count, b_styles) => match a.get_head() {
                AddStyles(a_count, a_styles) => {
                    let mut both_styles = b_styles.clone();
                    both_styles.extend(a_styles.clone());
                    res.push(AddStyles(cmp::min(a_count, b_count), both_styles));
                    if a_count > b_count {
                        b.head = Some(AddStyles(a_count - b_count, a_styles));
                        b.next();
                    } else if a_count < b_count {
                        a.head = Some(AddStyles(b_count - a_count, b_styles));
                        a.next();
                    } else {
                        a.next();
                        b.next();
                    }
                }
                AddChars(mut value) => {
                    if b_count < value.char_len() {
                        let (mut a_left, a_right) = value.split_at(b_count);
                        a_left.extend_styles(&b_styles);
                        res.place(&AddChars(a_left));
                        a.head = Some(AddChars(a_right));
                        b.next();
                    } else if b_count > value.char_len() {
                        value.extend_styles(&b_styles);
                        b.head = Some(AddStyles(b_count - value.char_len(), b_styles));
                        res.place(&AddChars(value));
                        a.next();
                    } else {
                        value.extend_styles(&b_styles);
                        res.place(&AddChars(value));
                        a.next();
                        b.next();
                    }
                }
                AddSkip(acount) => {
                    res.push(AddStyles(cmp::min(acount, b_count), b_styles.clone()));
                    if acount > b_count {
                        b.head = Some(AddSkip(acount - b_count));
                        b.next();
                    } else if acount < b_count {
                        a.head = Some(AddStyles(b_count - acount, b_styles));
                        a.next();
                    } else {
                        a.next();
                        b.next();
                    }
                }
                AddWithGroup(..) => {
                    res.push(a.next().unwrap());
                    if b_count == 1 {
                        b.next();
                    } else {
                        b.head = Some(AddSkip(b_count - 1));
                    }
                }
                AddGroup(..) => {
                    res.push(a.next().unwrap());
                    if b_count == 1 {
                        b.next();
                    } else {
                        b.head = Some(AddSkip(b_count - 1));
                    }
                }
            },
            AddSkip(bcount) => match a.get_head() {
                AddStyles(acount, a_styles) => {
                    res.push(AddStyles(cmp::min(acount, bcount), a_styles.clone()));
                    if acount > bcount {
                        a.head = Some(AddStyles(acount - bcount, a_styles));
                        b.next();
                    } else if acount < bcount {
                        b.head = Some(AddSkip(bcount - acount));
                        a.next();
                    } else {
                        a.next();
                        b.next();
                    }
                }
                AddChars(value) => {
                    if bcount < value.char_len() {
                        let (a_left, a_right) = value.split_at(bcount);
                        res.place(&AddChars(a_left));
                        a.head = Some(AddChars(a_right));
                        b.next();
                    } else if bcount > value.char_len() {
                        res.place(&a.next().unwrap());
                        b.head = Some(AddSkip(bcount - value.char_len()));
                    } else {
                        res.place(&a.get_head());
                        a.next();
                        b.next();
                    }
                }
                AddSkip(acount) => {
                    res.push(AddSkip(cmp::min(acount, bcount)));
                    if acount > bcount {
                        a.head = Some(AddSkip(acount - bcount));
                        b.next();
                    } else if acount < bcount {
                        b.head = Some(AddSkip(bcount - acount));
                        a.next();
                    } else {
                        a.next();
                        b.next();
                    }
                }
                AddWithGroup(_span) => {
                    res.push(a.next().unwrap());
                    if bcount == 1 {
                        b.next();
                    } else {
                        b.head = Some(AddSkip(bcount - 1));
                    }
                }
                AddGroup(..) => {
                    res.push(a.next().unwrap());
                    if bcount == 1 {
                        b.next();
                    } else {
                        b.head = Some(AddSkip(bcount - 1));
                    }
                }
            },
            AddGroup(attrs, bspan) => {
                let mut c = AddStepper::new(&bspan);
                let mut inner = vec![];
                compose_add_add_inner(&mut inner, a, &mut c);
                if !c.is_done() {
                    inner.place(&c.get_head());
                    inner.place_all(&c.rest);
                }
                res.push(AddGroup(attrs.clone(), inner));
                b.next();
            }
            AddWithGroup(ref bspan) => match a.get_head() {
                AddChars(..) => {
                    panic!("Cannot compose AddWithGroup with AddChars");
                }
                AddStyles(..) => {
                    panic!("Cannot compose AddWithGroup with AddStyles");
                }
                AddSkip(acount) => {
                    if acount == 1 {
                        a.next();
                    } else {
                        a.head = Some(AddSkip(acount - 1));
                    }
                    res.push(b.next().unwrap());
                }
                AddWithGroup(ref aspan) => {
                    res.push(AddWithGroup(compose_add_add(aspan, bspan)));
                    a.next();
                    b.next();
                }
                AddGroup(ref attrs, ref aspan) => {
                    res.push(AddGroup(attrs.clone(), compose_add_add(aspan, bspan)));
                    a.next();
                    b.next();
                }
            },
        }
    }
}

pub fn compose_add_add(avec: &AddSpan, bvec: &AddSpan) -> AddSpan {
    let mut res = Vec::with_capacity(avec.len() + bvec.len());

    let mut a = AddStepper::new(avec);
    let mut b = AddStepper::new(bvec);

    compose_add_add_inner(&mut res, &mut a, &mut b);

    if !b.is_done() {
        res.place(&b.get_head());
        res.place_all(&b.rest);
    }

    if !a.is_done() {
        res.place(&a.get_head());
        res.place_all(&a.rest);
    }

    res
}

pub fn compose_add_del(avec: &AddSpan, bvec: &DelSpan) -> Op {
    let mut delres: DelSpan = Vec::with_capacity(avec.len() + bvec.len());
    let mut addres: AddSpan = Vec::with_capacity(avec.len() + bvec.len());

    let mut a = AddStepper::new(avec);
    let mut b = DelStepper::new(bvec);

    compose_add_del_inner(&mut delres, &mut addres, &mut a, &mut b);

    if !b.is_done() {
        let rest = b.into_span();
        if rest.skip_post_len() > 0 {
            addres.place(&AddSkip(rest.skip_post_len()));
        }
        delres.place_all(&rest);
    }

    if !a.is_done() {
        let rest = a.into_span();
        if rest.skip_pre_len() > 0 {
            delres.place(&DelSkip(rest.skip_pre_len()));
        }
        addres.place_all(&rest);
    }

    (delres, addres)
}

fn compose_add_del_inner(
    delres: &mut DelSpan,
    addres: &mut AddSpan,
    a: &mut AddStepper,
    b: &mut DelStepper,
) {
    while !b.is_done() && !a.is_done() {
        match b.get_head() {
            DelChars(bcount) => match a.get_head() {
                AddChars(avalue) => {
                    if bcount < avalue.char_len() {
                        let (_a_left, a_right) = avalue.split_at(bcount);
                        a.head = Some(AddChars(a_right));
                        b.next();
                    } else if bcount > avalue.char_len() {
                        a.next();
                        b.head = Some(DelChars(bcount - avalue.char_len()));
                    } else {
                        a.next();
                        b.next();
                    }
                }
                AddSkip(acount) => {
                    if bcount < acount {
                        a.head = Some(AddSkip(acount - bcount));
                        delres.place(&b.next().unwrap());
                    } else if bcount > acount {
                        a.next();
                        delres.place(&DelChars(acount));
                        b.head = Some(DelChars(bcount - acount));
                    } else {
                        a.next();
                        delres.place(&b.next().unwrap());
                    }
                }
                _ => {
                    panic!("Unimplemented or Unexpected");
                }
            },
            DelStyles(b_count, b_styles) => match a.get_head() {
                AddChars(mut a_value) => {
                    if b_count < a_value.char_len() {
                        let (mut a_left, a_right) = a_value.split_at(b_count);
                        a_left.remove_styles(&b_styles);
                        addres.place(&AddChars(a_left));
                        a.head = Some(AddChars(a_right));
                        b.next();
                    } else if b_count > a_value.char_len() {
                        a_value.remove_styles(&b_styles);
                        b.head = Some(DelSkip(b_count - a_value.char_len()));
                        addres.place(&AddChars(a_value));
                    } else {
                        a_value.remove_styles(&b_styles);
                        addres.place(&AddChars(a_value));
                        a.next();
                        b.next();
                    }
                }
                AddStyles(a_count, a_styles) => {
                    // a_styles - b_styles
                    let combined_styles = a_styles
                        .clone()
                        .drain()
                        .filter(|(k, _)| !b_styles.contains(k))
                        .collect();

                    // res.push(AddStyles(cmp::min(a_count, b_count), both_styles));
                    if a_count > b_count {
                        a.head = Some(AddStyles(a_count - b_count, a_styles));
                        a.next();
                        addres.place(&AddStyles(b_count, combined_styles));
                    } else if a_count < b_count {
                        b.head = Some(DelStyles(b_count - a_count, b_styles));
                        a.next();
                        addres.place(&AddStyles(b_count, combined_styles));
                    } else {
                        a.next();
                        b.next();
                        addres.place(&AddStyles(a_count, combined_styles));
                    }
                    delres.place(&b.next().unwrap());
                }
                AddSkip(a_count) => {
                    addres.place(&AddSkip(cmp::min(a_count, b_count)));
                    delres.place(&DelStyles(cmp::min(a_count, b_count), b_styles.clone()));
                    if a_count > b_count {
                        a.head = Some(AddSkip(a_count - b_count));
                        b.next();
                    } else if a_count < b_count {
                        a.next();
                        b.head = Some(DelStyles(b_count - a_count, b_styles.clone()));
                    } else {
                        a.next();
                        b.next();
                    }
                }
                AddWithGroup(..) => {
                    panic!("DelStyles by AddWithGroup is ILLEGAL");
                }
                AddGroup(..) => {
                    panic!("DelStyles by AddGroup is ILLEGAL");
                }
            },
            DelSkip(bcount) => match a.get_head() {
                AddChars(avalue) => {
                    if bcount < avalue.char_len() {
                        let (a_left, a_right) = avalue.split_at(bcount);
                        addres.place(&AddChars(a_left));
                        a.head = Some(AddChars(a_right));
                        b.next();
                    } else if bcount > avalue.char_len() {
                        addres.place(&a.next().unwrap());
                        b.head = Some(DelSkip(bcount - avalue.char_len()));
                    } else {
                        addres.place(&a.get_head());
                        a.next();
                        b.next();
                    }
                }
                AddStyles(a_count, a_styles) => {
                    addres.place(&AddStyles(cmp::min(a_count, bcount), a_styles.clone()));
                    delres.place(&DelSkip(cmp::min(a_count, bcount)));
                    if a_count > bcount {
                        a.head = Some(AddStyles(a_count - bcount, a_styles));
                        b.next();
                    } else if a_count < bcount {
                        a.next();
                        b.head = Some(DelSkip(bcount - a_count));
                    } else {
                        a.next();
                        b.next();
                    }
                }
                AddSkip(acount) => {
                    addres.place(&AddSkip(cmp::min(acount, bcount)));
                    delres.place(&DelSkip(cmp::min(acount, bcount)));
                    if acount > bcount {
                        a.head = Some(AddSkip(acount - bcount));
                        b.next();
                    } else if acount < bcount {
                        a.next();
                        b.head = Some(DelSkip(bcount - acount));
                    } else {
                        a.next();
                        b.next();
                    }
                }
                AddWithGroup(..) => {
                    addres.place(&a.next().unwrap());
                    delres.place(&DelSkip(1));
                    if bcount == 1 {
                        b.next();
                    } else {
                        b.head = Some(DelSkip(bcount - 1));
                    }
                }
                AddGroup(_, aspan) => {
                    addres.place(&a.next().unwrap());
                    if aspan.skip_pre_len() > 0 {
                        delres.place(&DelSkip(aspan.skip_pre_len()));
                    }
                    if bcount == 1 {
                        b.next();
                    } else {
                        b.head = Some(DelSkip(bcount - 1));
                    }
                }
            },
            DelWithGroup(span) => match a.get_head() {
                AddChars(..) => {
                    panic!("DelWithGroup by AddChars is ILLEGAL");
                }
                AddStyles(..) => {
                    panic!("DelWithGroup by AddStyles is ILLEGAL");
                }
                AddSkip(acount) => {
                    delres.place(&b.next().unwrap());
                    addres.place(&AddSkip(1));
                    if acount > 1 {
                        a.head = Some(AddSkip(acount - 1));
                    } else {
                        a.next();
                    }
                }
                AddWithGroup(insspan) => {
                    a.next();
                    b.next();

                    let (del, ins) = compose_add_del(&insspan, &span);
                    delres.place(&DelWithGroup(del));
                    addres.place(&AddWithGroup(ins));
                }
                AddGroup(attr, insspan) => {
                    a.next();
                    b.next();

                    let (del, ins) = compose_add_del(&insspan, &span);
                    addres.place(&AddGroup(attr, ins));
                    delres.place_all(&del);
                }
            },
            DelGroup(span) => {
                match a.get_head() {
                    AddChars(..) => {
                        panic!("DelGroup by AddChars is ILLEGAL");
                    }
                    AddStyles(..) => {
                        panic!("DelGroup by AddStyles is ILLEGAL");
                    }
                    AddSkip(acount) => {
                        delres.place(&b.next().unwrap());
                        if span.skip_post_len() > 0 {
                            addres.place(&AddSkip(span.skip_post_len()));
                        }
                        if acount > 1 {
                            a.head = Some(AddSkip(acount - 1));
                        } else {
                            a.next();
                        }
                    }
                    AddWithGroup(insspan) => {
                        a.next();
                        b.next();

                        let (del, ins) = compose_add_del(&insspan, &span);
                        delres.place(&DelGroup(del));
                        addres.place_all(&ins[..]);

                        // let mut a_stepper = AddStepper::new(&insspan);
                        // let mut b_stepper = DelStepper::new(&span);

                        // let mut del_inner = vec![];
                        // compose_add_del_inner(&mut del_inner, addres, &mut a_stepper, &mut b_stepper);

                        // if !b_stepper.is_done() {
                        //     del_inner.place_all(&b_stepper.into_span());
                        // }

                        // if !a_stepper.is_done() {
                        //     let rest = a_stepper.into_span();
                        //     if rest.skip_pre_len() > 0 {
                        //         del_inner.place(&DelSkip(rest.skip_pre_len()));
                        //     }
                        //     addres.place_all(&rest);
                        // }

                        // delres.place(&DelGroup(del_inner));

                        // if b_stepper.clone().into_span().skip_post_len() > 0 {
                        //     addres.place(&AddSkip(b_stepper.into_span().skip_post_len()));
                        // }
                    }
                    AddGroup(_, insspan) => {
                        a.next();
                        b.next();

                        let (del, ins) = compose_add_del(&insspan, &span);
                        delres.place_all(&del[..]);
                        addres.place_all(&ins[..]);
                    }
                }
            } // DelObject => {
              //     match a.get_head() {
              //         AddSkip(acount) => {
              //             if acount > 1 {
              //                 a.head = Some(AddSkip(acount - 1));
              //                 delres.place(&b.next().unwrap());
              //             } else {
              //                 a.next();
              //                 delres.place(&b.next().unwrap());
              //             }
              //         }
              //         _ => {
              //             panic!("Bad");
              //         }
              //     }
              // }
              // DelMany(bcount) => {
              //     match a.get_head() {
              //         AddChars(avalue) => {
              //             let alen = avalue.chars().count();
              //             if bcount < alen {
              //                 a.head = Some(AddChars(avalue.chars().skip(bcount).collect()));
              //                 b.next();
              //             } else if bcount > alen {
              //                 a.next();
              //                 b.head = Some(DelMany(bcount - alen));
              //             } else {
              //                 a.next();
              //                 b.next();
              //             }
              //         }
              //         AddSkip(acount) => {
              //             if bcount < acount {
              //                 a.head = Some(AddSkip(acount - bcount));
              //                 delres.place(&b.next().unwrap());
              //             } else if bcount > acount {
              //                 a.next();
              //                 delres.place(&DelMany(acount));
              //                 b.head = Some(DelMany(bcount - acount));
              //             } else {
              //                 a.next();
              //                 b.next();
              //             }
              //         }
              //         AddGroup(attr, ins_span) => {
              //             if bcount > 1 {
              //                 a.next();
              //                 delres.place(&DelMany(ins_span.skip_len()));
              //                 b.head = Some(DelMany(bcount - 1));
              //             } else {
              //                 a.next();
              //                 b.next();
              //             }
              //         }
              //         AddWithGroup(insspan) => {
              //             if bcount > 1 {
              //                 a.next();
              //                 b.head = Some(DelMany(bcount - 1));
              //             } else {
              //                 a.next();
              //                 b.next();
              //             }
              //             delres.place(&DelMany(1));
              //         }
              //     }
              // }
              // DelGroupAll => {
              //     match a.get_head() {
              //         AddChars(avalue) => {
              //             panic!("DelGroupAll by AddChars is ILLEGAL");
              //         }
              //         AddSkip(acount) => {
              //             delres.place(&b.next().unwrap());
              //             if acount > 1 {
              //                 a.head = Some(AddSkip(acount - 1));
              //             } else {
              //                 a.next();
              //             }
              //         }
              //         AddWithGroup(insspan) => {
              //             a.next();
              //             delres.place(&b.next().unwrap());
              //         }
              //         AddGroup(attr, insspan) => {
              //             a.next();
              //             b.next();
              //         }
              //     }
              // }
        }
    }
}

pub fn compose(a: &Op, b: &Op) -> Op {
    let &(ref adel, ref ains) = a;
    let &(ref bdel, ref bins) = b;

    log_compose!("`````````````` >(compose)<");
    log_compose!("``````````````a_ins {:?}", ains);
    log_compose!("``````````````b_del {:?}", bdel);
    let (mdel, mins) = compose_add_del(ains, bdel);
    log_compose!("``````````````  a=> {:?}", mdel);
    log_compose!("``````````````  b=> {:?}", mins);

    log_compose!("``````````````a_del {:?}", adel);
    log_compose!("``````````````  a=>  {:?}", mdel);
    let a_ = compose_del_del(adel, &mdel);
    log_compose!("``````````````  del' {:?}", a_);

    log_compose!("``````````````  b=> {:?}", mins);
    log_compose!("`````````````b_ins {:?}", bins);
    let b_ = compose_add_add(&mins, bins);
    log_compose!("`````````````` ins' {:?}", b_);
    log_compose!();
    log_compose!();

    (a_, b_)
}

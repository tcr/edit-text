//! Composes two operations together.

use super::doc::*;
use crate::stepper::*;
use std::cmp;

fn compose_del_del_inner<S: Schema>(
    res: &mut DelSpan<S>,
    a: &mut DelStepper<S>,
    b: &mut DelStepper<S>,
) {
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
                    Some(DelText(bcount)) => {
                        res.place(&DelText(cmp::min(acount, bcount)));
                        if acount > bcount {
                            a.head = Some(DelSkip(acount - bcount));
                            b.next();
                        } else if acount < bcount {
                            b.head = Some(DelText(bcount - acount));
                            a.next();
                        } else {
                            a.next();
                            b.next();
                        }
                    }
                    Some(DelStyles(b_count, b_styles)) => {
                        res.place(&DelText(cmp::min(acount, b_count)));
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
                    both_styles.extend(&a_styles);
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
                Some(DelText(b_count)) => {
                    res.place(&DelText(cmp::min(a_count, b_count)));
                    if a_count > b_count {
                        a.head = Some(DelStyles(a_count - b_count, a_styles));
                        b.next();
                    } else if a_count < b_count {
                        b.head = Some(DelText(b_count - a_count));
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
                    Some(DelText(..)) => {
                        panic!("DelWithGroup vs DelText is bad");
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
                let mut inner: DelSpan<S> = vec![];
                compose_del_del_inner(&mut inner, &mut c, b);
                if !c.is_done() {
                    inner.place(&c.head.unwrap());
                    inner.place_all(&c.rest);
                }
                res.place(&DelGroup(inner));
                a.next();
            }
            DelText(count) => {
                res.place(&DelText(count));
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

pub fn compose_del_del<S: Schema>(avec: &DelSpan<S>, bvec: &DelSpan<S>) -> DelSpan<S> {
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

fn compose_add_add_inner<S: Schema>(
    res: &mut AddSpan<S>,
    a: &mut AddStepper<S>,
    b: &mut AddStepper<S>,
) {
    while !b.is_done() && !a.is_done() {
        match b.get_head() {
            AddText(..) => {
                res.place(&b.next().unwrap());
            }
            AddStyles(b_count, b_styles) => match a.get_head() {
                AddStyles(a_count, a_styles) => {
                    let mut both_styles = b_styles.clone();
                    both_styles.extend(&a_styles);
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
                AddText(mut styles, value) => {
                    if b_count < value.char_len() {
                        let (a_left, a_right) = value.split_at(b_count);
                        let mut left_styles = styles.clone();
                        left_styles.extend(&b_styles);
                        res.place(&AddText(left_styles, a_left));
                        a.head = Some(AddText(styles, a_right));
                        b.next();
                    } else if b_count > value.char_len() {
                        styles.extend(&b_styles);
                        b.head = Some(AddStyles(b_count - value.char_len(), b_styles));
                        res.place(&AddText(styles, value));
                        a.next();
                    } else {
                        styles.extend(&b_styles);
                        res.place(&AddText(styles, value));
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
                AddText(styles, value) => {
                    if bcount < value.char_len() {
                        let (a_left, a_right) = value.split_at(bcount);
                        res.place(&AddText(styles.clone(), a_left));
                        a.head = Some(AddText(styles, a_right));
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
                AddText(..) => {
                    panic!("Cannot compose AddWithGroup with AddText");
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

pub fn compose_add_add<S: Schema>(avec: &AddSpan<S>, bvec: &AddSpan<S>) -> AddSpan<S> {
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

pub fn compose_add_del<S: Schema>(avec: &AddSpan<S>, bvec: &DelSpan<S>) -> Op<S> {
    let mut delres: DelSpan<S> = Vec::with_capacity(avec.len() + bvec.len());
    let mut addres: AddSpan<S> = Vec::with_capacity(avec.len() + bvec.len());

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

    Op(delres, addres)
}

fn compose_add_del_inner<S: Schema>(
    delres: &mut DelSpan<S>,
    addres: &mut AddSpan<S>,
    a: &mut AddStepper<S>,
    b: &mut DelStepper<S>,
) {
    while !b.is_done() && !a.is_done() {
        match b.get_head() {
            DelText(bcount) => match a.get_head() {
                AddText(a_styles, avalue) => {
                    if bcount < avalue.char_len() {
                        let (_a_left, a_right) = avalue.split_at(bcount);
                        a.head = Some(AddText(a_styles, a_right));
                        b.next();
                    } else if bcount > avalue.char_len() {
                        a.next();
                        b.head = Some(DelText(bcount - avalue.char_len()));
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
                        delres.place(&DelText(acount));
                        b.head = Some(DelText(bcount - acount));
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
                AddText(mut a_styles, a_value) => {
                    if b_count < a_value.char_len() {
                        let (a_left, a_right) = a_value.split_at(b_count);
                        let mut a_left_styles = a_styles.clone();
                        a_left_styles.remove(&b_styles);
                        addres.place(&AddText(a_left_styles, a_left));
                        a.head = Some(AddText(a_styles, a_right));
                        b.next();
                    } else if b_count > a_value.char_len() {
                        a_styles.remove(&b_styles);
                        b.head = Some(DelSkip(b_count - a_value.char_len()));
                        addres.place(&AddText(a_styles, a_value));
                    } else {
                        a_styles.remove(&b_styles);
                        addres.place(&AddText(a_styles, a_value));
                        a.next();
                        b.next();
                    }
                }
                AddStyles(a_count, a_styles) => {
                    // a_styles - b_styles
                    let mut combined_styles = a_styles.clone();
                    combined_styles.remove(&b_styles);

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
                AddText(a_styles, avalue) => {
                    if bcount < avalue.char_len() {
                        let (a_left, a_right) = avalue.split_at(bcount);
                        addres.place(&AddText(a_styles.clone(), a_left));
                        a.head = Some(AddText(a_styles, a_right));
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
                AddText(..) => {
                    panic!("DelWithGroup by AddText is ILLEGAL");
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

                    let Op(del, ins) = compose_add_del(&insspan, &span);
                    delres.place(&DelWithGroup(del));
                    addres.place(&AddWithGroup(ins));
                }
                AddGroup(attr, insspan) => {
                    a.next();
                    b.next();

                    let Op(del, ins) = compose_add_del(&insspan, &span);
                    addres.place(&AddGroup(attr, ins));
                    delres.place_all(&del);
                }
            },
            DelGroup(span) => {
                match a.get_head() {
                    AddText(..) => {
                        panic!("DelGroup by AddText is ILLEGAL");
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

                        let Op(del, ins) = compose_add_del(&insspan, &span);
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

                        let Op(del, ins) = compose_add_del(&insspan, &span);
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
              //         AddText(avalue) => {
              //             let alen = avalue.chars().count();
              //             if bcount < alen {
              //                 a.head = Some(AddText(avalue.chars().skip(bcount).collect()));
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
              //         AddText(avalue) => {
              //             panic!("DelGroupAll by AddText is ILLEGAL");
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

pub fn compose<S: Schema>(a: &Op<S>, b: &Op<S>) -> Op<S> {
    let &Op(ref adel, ref ains) = a;
    let &Op(ref bdel, ref bins) = b;

    log_compose!("`````````````` >(compose)<");
    log_compose!("``````````````a_ins {:?}", ains);
    log_compose!("``````````````b_del {:?}", bdel);
    let Op(mdel, mins) = compose_add_del(ains, bdel);
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

    Op(a_, b_)
}

#![allow(unused_imports)]

extern crate env_logger;
#[macro_use]
extern crate log;
#[macro_use]
extern crate oatie;
extern crate rand;
extern crate term_painter;

use oatie::*;
use oatie::compose::*;
use oatie::doc::*;
use oatie::random::*;
use rand::{thread_rng, Rng};
use std::collections::HashMap;

fn test_start() {
    let _ = env_logger::init();
}

#[test]
fn monkey_add_add() {
    test_start();

    for _ in 0..1000 {
        let start = vec![DocChars("Hello world!".to_owned())];

        trace!("start {:?}", start);

        let a = random_add_span(&start);
        trace!("Random A: {:?}", a);

        let middle = apply_add(&start, &a);
        let b = random_add_span(&middle);
        trace!("Random B: {:?}", a);
        let end = apply_add(&middle, &b);

        let composed = compose_add_add(&a, &b);
        trace!("Composed: {:?}", composed);
        let otherend = apply_add(&start, &composed);

        trace!("middle {:?}", middle);
        trace!("b {:?}", b);
        trace!("end {:?}", end);

        trace!("composed {:?}", composed);
        trace!("otherend {:?}", otherend);

        assert_eq!(end, otherend);

        trace!("-----");
    }
}

#[test]
fn monkey_del_del() {
    test_start();

    for _ in 0..1000 {
        let start = vec![DocChars("Hello world!".to_owned())];

        trace!("start {:?}", start);

        let a = random_del_span(&start);
        trace!("a {:?}", a);

        let middle = apply_delete(&start, &a);
        let b = random_del_span(&middle);
        let end = apply_delete(&middle, &b);

        let composed = compose_del_del(&a, &b);
        let otherend = apply_delete(&start, &composed);

        trace!("middle {:?}", middle);
        trace!("b {:?}", b);
        trace!("end {:?}", end);

        trace!("composed {:?}", composed);
        trace!("otherend {:?}", otherend);

        assert_eq!(end, otherend);
    }
}

#[test]
fn monkey_add_del() {
    test_start();

    for _ in 0..1000 {
        let start = vec![DocChars("Hello world!".to_owned())];

        trace!("start {:?}", start);

        let a = random_add_span(&start);
        trace!("a {:?}", a);

        let middle = apply_add(&start, &a);
        let b = random_del_span(&middle);
        let end = apply_delete(&middle, &b);

        trace!("middle {:?}", middle);
        trace!("b {:?}", b);
        trace!("end {:?}", end);

        let (dela, addb) = compose_add_del(&a, &b);
        trace!("dela {:?}", dela);
        trace!("addb {:?}", addb);

        let middle2 = apply_delete(&start, &dela);
        trace!("middle2 {:?}", middle2);
        let otherend = apply_add(&middle2, &addb);
        trace!("otherend {:?}", otherend);

        assert_eq!(end, otherend);
    }
}

fn random_op(input: &DocSpan) -> Op {
    trace!("random_op: input {:?}", input);
    let del = random_del_span(input);
    trace!("random_op: del {:?}", del);
    let middle = apply_delete(input, &del);
    let ins = random_add_span(&middle);
    (del, ins)
}

#[test]
fn monkey_compose() {
    test_start();

    let mut start = vec![DocChars("Hello world!".to_owned())];

    for _ in 0..100 {
        trace!("start {:?}", start);

        let a = random_op(&start);
        trace!("a {:?}", a);

        let middle = apply_operation(&start, &a);
        trace!("middle {:?}", middle);

        let b = random_op(&middle);
        trace!("b {:?}", b);

        let end = apply_operation(&middle, &b);
        trace!("end {:?}", end);

        let composed = compose(&a, &b);
        trace!("composed {:?}", composed);

        let otherend = apply_operation(&start, &composed);
        trace!("otherend {:?}", otherend);

        assert_eq!(end, otherend);

        start = end;
    }
}

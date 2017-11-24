extern crate oatie;
extern crate serde_json;
#[macro_use] extern crate maplit;

use std::io;
use std::io::prelude::*;
use serde_json::Value;
use std::collections::HashMap;

use oatie::compose;
use oatie::doc::*;
use oatie::normalize;
use oatie::transform::*;

macro_rules! comma_seq {
    ($strval: expr, $target: expr, $fnv: expr) => (
        let mut inner = $strval;
        while inner.len() > 0 {
            let mut item = None;
            for (comma, _) in {
                let mut a: Vec<_> = inner.match_indices(",").collect();
                a.push((inner.len() as usize, ""));
                a
            } {
                let take = &inner[0..comma];
                match $fnv(take) {
                    Ok(arg) => {
                        item = Some(arg);
                        if comma >= inner.len() {
                            inner = &inner[inner.len()..];
                            break;
                        }
                        inner = &inner[comma + 1..];
                        break;
                    }
                    Err(..) => {
                        continue;
                    }
                }
            }
            if item.is_none() {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "Group has no readable data"));
            }
            $target.push(item.unwrap());
        }
    )
}

fn parse_del(mut value: &str) -> io::Result<DelElement> {
    value = value.trim();

    if value == "DelGroupAll" {
        return Ok(DelElement::DelGroupAll);
    }

    let mut cap = None;
    if value.starts_with("DelSkip(") && value.ends_with(")") {
        cap = Some(("DelSkip", &value["DelSkip(".len()..value.len()-1]));
    }
    if value.starts_with("DelChars(") && value.ends_with(")") {
        cap = Some(("DelChars", &value["DelChars(".len()..value.len()-1]));
    }

    if let Some((key, segment)) = cap {
        let inner = try!(segment.parse::<usize>()
            .or(Err(io::Error::new(io::ErrorKind::InvalidData, "Bad parsing of number"))));
        return Ok(match key {
            "DelSkip" => DelElement::DelSkip(inner),
            "DelChars" => DelElement::DelChars(inner),
            _ => unreachable!("why"),
        });
    }

    let mut cap = None;
    if value.starts_with("DelWithGroup([") && value.ends_with("])") {
        cap = Some(("DelWithGroup", &value["DelWithGroup([".len()..value.len()-2]));
    }
    if value.starts_with("DelGroup([") && value.ends_with("])") {
        cap = Some(("DelGroup", &value["DelGroup([".len()..value.len()-2]));
    }

    if let Some((key, inner)) = cap {
        let mut args = vec![];
        comma_seq!(inner, args, parse_del);

        return Ok(match key {
            "DelGroup" => DelElement::DelGroup(args),
            "DelWithGroup" => DelElement::DelWithGroup(args),
            _ => unreachable!("why 2"),
        });
    }
    Err(malformed("Invalid data"))
}

fn parse_add(mut value: &str) -> io::Result<AddElement> {
    value = value.trim();

    //if value == "DelGroupAll" {
    //    return Ok(DelElement::DelGroupAll);
    //}

    let mut cap = None;
    if value.starts_with("AddSkip(") && value.ends_with(")") {
        cap = Some(("AddSkip", &value["DelSkip(".len()..value.len()-1]));
    }
    if value.starts_with("AddChars(") && value.ends_with(")") {
        cap = Some(("AddChars", &value["DelChars(".len()..value.len()-1]));
    }

    if let Some((key, segment)) = cap {
        return Ok(match key {
            "AddSkip" => AddElement::AddSkip(try!(segment.parse::<usize>()
                .or(Err(io::Error::new(io::ErrorKind::InvalidData, "Bad parsing of number"))))),
            "AddChars" => {
                if segment.len() < 2 || !segment.starts_with("\"") || !segment.ends_with("\"") {
                    return Err(malformed("Expected full quoted string"));
                }
                let segment = &segment[1..segment.len()-1];
                AddElement::AddChars(segment.to_string())
            }
            _ => unreachable!(),
        });
    }

    // let mut cap = None;
    if value.starts_with("AddWithGroup([") && value.ends_with("])") {
        let inner = &value["AddWithGroup([".len()..value.len()-2];

        let mut args = vec![];
        comma_seq!(inner, args, parse_add);

        return Ok(AddElement::AddWithGroup(args));
    }

    if value.starts_with("AddGroup({") && value.ends_with("])") {
        println!("value {:?}", value);
        // panic!("this doesnt work yet");

        let right_index = value.find('}').unwrap();
        let body = &value["AddGroup(".len()..right_index+1];

        // Find ending } then subslice then
        let v: Value = serde_json::from_str(body).unwrap();
        let mut map: HashMap<String, String> = hashmap![];
        for (key, value) in v.as_object().unwrap() {
            map.insert(key.to_string(), value.as_str().unwrap().to_string());
        }

        let next_index = value[right_index+1..].find('[').unwrap();
        let inner = &value[right_index+1+next_index+1..value.len()-2];
        println!("what {:?}", inner);
        let mut args = vec![];
        println!("------>");
        comma_seq!(inner, args, parse_add);

        return Ok(AddElement::AddGroup(map, args));
    }

    // if let Some((key, inner)) = cap {
    //     let mut args = vec![];
    //     comma_seq!(inner, args, parse_add);

    //     return Ok(match key {
    //         //"DelGroup" => DelElement::DelGroup(args),
    //         "AddWithGroup" => AddElement::AddWithGroup(args),
    //         _ => unreachable!(),
    //     });
    // }

    Err(malformed("Invalid data"))
}

fn malformed(reason: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, reason)
}


fn run(input: &str) -> io::Result<()> {
//    let input = r#"
//[DelSkip(1)]
//[AddWithGroup([AddWithGroup([AddWithGroup([AddChars("1234"), AddSkip(6)])])])]
//
//[DelWithGroup([DelWithGroup([DelWithGroup([DelChars(4), DelSkip(2)])])])]
//[]
//"#;

    let four = input.lines().filter(|x| x.len() > 0).collect::<Vec<_>>();
    if four.len() != 4 && four.len() != 6 {
        return Err(malformed("Needed four or six lines as input"));
    }

    let a = four[0].clone();
    if !(a.starts_with("[") && a.ends_with("]")) {
        return Err(malformed("Expected array"));
    }
    let inner = &a[1..a.len()-1];
    let mut del_a = vec![];
    comma_seq!(inner, del_a, parse_del);

    let a = four[1].clone();
    if !(a.starts_with("[") && a.ends_with("]")) {
        return Err(malformed("Expected array"));
    }
    let inner = &a[1..a.len()-1];
    let mut add_a = vec![];
    comma_seq!(inner, add_a, parse_add);

    let a = four[2].clone();
    if !(a.starts_with("[") && a.ends_with("]")) {
        return Err(malformed("Expected array"));
    }
    let inner = &a[1..a.len()-1];
    let mut del_b = vec![];
    comma_seq!(inner, del_b, parse_del);

    let a = four[3].clone();
    if !(a.starts_with("[") && a.ends_with("]")) {
        return Err(malformed("Expected array"));
    }
    let inner = &a[1..a.len()-1];
    let mut add_b = vec![];
    comma_seq!(inner, add_b, parse_add);

    println!("op {:?} {:?}", del_a, del_b);
    println!("okay!");

    let a = (del_a, add_a);
    let b = (del_b, add_b);
    let confirm = op_transform_compare(a, b);

    // Check validating lines
    if four.len() == 6 {
        let a = four[4].clone();
        if !(a.starts_with("[") && a.ends_with("]")) {
            return Err(malformed("Expected array"));
        }
        let inner = &a[1..a.len()-1];
        let mut confirm_del = vec![];
        comma_seq!(inner, confirm_del, parse_del);

        let a = four[5].clone();
        if !(a.starts_with("[") && a.ends_with("]")) {
            return Err(malformed("Expected array"));
        }
        let inner = &a[1..a.len()-1];
        let mut confirm_add = vec![];
        comma_seq!(inner, confirm_add, parse_add);

        println!("Validating...");
        assert_eq!(confirm, (confirm_del, confirm_add));
        println!("Valid!");
    }

    Ok(())
}

fn op_transform_compare(a: Op, b: Op) -> Op {
    let (a_, b_) = transform(&a, &b);

    let a_res = normalize(compose::compose(&a, &a_));
    let b_res = normalize(compose::compose(&b, &b_));

    println!("");
    println!("A' {:?}", a_res);
    println!("B' {:?}", b_res);
    println!("");

    assert_eq!(a_res, b_res);

    a_res
}

//fn main() {
//    run(r#"
//[DelSkip(1)]
//[AddWithGroup([AddWithGroup([AddGroup([AddChars("1234"), AddSkip(6)])])])]
//
//[DelWitqGroup([De([Del?ithGroup([(4), Del(2֢)])])]
//[]
//"#).unwrap();
//}

// Copyright 2015 Keegan McAllister.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// See `LICENSE` in this repository.

fn main() {
    let mut input = String::new();
    let stdin = io::stdin();
    stdin.lock().read_to_string(&mut input).expect("Could not read stdin");

    match run(&input) {
        Ok(..) => {
            println!("all set!");
        }
        Err(err) => {
            println!("transform error: {}", err);
            ::std::process::exit(1);
        }
    }
}

use std::io;
use std::io::prelude::*;
use serde_json::Value;
use std::collections::HashMap;

use super::compose;
use super::doc::*;
use super::normalize;
use super::transform::*;
use serde_json;
use failure::Error;

#[derive(Debug, Fail)]
#[fail(display = "Group has no readable data")]
struct NoReadableData;

#[derive(Debug, Fail)]
#[fail(display = "Malformed data: {}", _0)]
struct MalformedData(String);

#[derive(Debug, Fail)]
#[fail(display = "Exhausted array")]
struct ExhaustedArray;

fn comma_seq<T, F>(mut inner: &str, fnv: F) -> Result<Vec<T>, NoReadableData>
where
    F: Fn(&str) -> Result<T, Error>,
{
    let mut target = vec![];
    while !inner.is_empty() {
        let mut item = None;
        for (comma, _) in {
            let mut a: Vec<_> = inner.match_indices(",").collect();
            a.push((inner.len() as usize, ""));
            a
        }
        {
            let take = &inner[0..comma];
            match fnv(take) {
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
            return Err(NoReadableData);
        }
        target.push(item.unwrap());
    }
    Ok(target)
}

fn parse_del(mut value: &str) -> Result<DelElement, Error> {
    value = value.trim();

    if value == "DelGroupAll" {
        return Ok(DelElement::DelGroupAll);
    }

    let cap = if value.starts_with("DelSkip(") && value.ends_with(')') {
        Some(("DelSkip", &value["DelSkip(".len()..value.len() - 1]))
    } else if value.starts_with("DelChars(") && value.ends_with(')') {
        Some(("DelChars", &value["DelChars(".len()..value.len() - 1]))
    } else {
        None
    };

    if let Some((key, segment)) = cap {
        let inner = segment.parse::<usize>()?;
        return Ok(match key {
            "DelSkip" => DelElement::DelSkip(inner),
            "DelChars" => DelElement::DelChars(inner),
            _ => unreachable!("why"),
        });
    }

    let cap = if value.starts_with("DelWithGroup([") && value.ends_with("])") {
        Some((
            "DelWithGroup",
            &value["DelWithGroup([".len()..value.len() - 2],
        ))
    } else if value.starts_with("DelGroup([") && value.ends_with("])") {
        Some(("DelGroup", &value["DelGroup([".len()..value.len() - 2]))
    } else {
        None
    };

    if let Some((key, inner)) = cap {
        let args = comma_seq(inner, parse_del)?;

        return Ok(match key {
            "DelGroup" => DelElement::DelGroup(args),
            "DelWithGroup" => DelElement::DelWithGroup(args),
            _ => unreachable!("why 2"),
        });
    }
    Err(MalformedData("Invalid data".into()))?
}

fn parse_add(mut value: &str) -> Result<AddElement, Error> {
    value = value.trim();

    //if value == "DelGroupAll" {
    //    return Ok(DelElement::DelGroupAll);
    //}

    let cap = if value.starts_with("AddSkip(") && value.ends_with(')') {
        Some(("AddSkip", &value["DelSkip(".len()..value.len() - 1]))
    } else if value.starts_with("AddChars(") && value.ends_with(')') {
        Some(("AddChars", &value["DelChars(".len()..value.len() - 1]))
    } else {
        None
    };

    if let Some((key, segment)) = cap {
        return Ok(match key {
            "AddSkip" => AddElement::AddSkip(try!(segment.parse::<usize>().or_else(|_| {
                Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Bad parsing of number",
                ))
            }))),
            "AddChars" => {
                if segment.len() < 2 || !segment.starts_with("\"") || !segment.ends_with("\"") {
                    Err(MalformedData("Expected full quoted string".into()))?;
                }
                let segment = &segment[1..segment.len() - 1];
                AddElement::AddChars(segment.to_string())
            }
            _ => unreachable!(),
        });
    }

    // let mut cap = None;
    if value.starts_with("AddWithGroup([") && value.ends_with("])") {
        let inner = &value["AddWithGroup([".len()..value.len() - 2];

        let args = comma_seq(inner, parse_add)?;

        return Ok(AddElement::AddWithGroup(args));
    }

    if value.starts_with("AddGroup({") && value.ends_with("])") {
        // panic!("this doesnt work yet");

        let right_index = value.find('}').unwrap();
        let body = &value["AddGroup(".len()..right_index + 1];

        // Find ending } then subslice then
        let v: Value = serde_json::from_str(body).unwrap();
        let mut map: HashMap<String, String> = hashmap![];
        for (key, value) in v.as_object().unwrap() {
            map.insert(key.to_string(), value.as_str().unwrap().to_string());
        }

        let next_index = value[right_index + 1..].find('[').unwrap();
        let inner = &value[right_index + 1 + next_index + 1..value.len() - 2];
        let args = comma_seq(inner, parse_add)?;

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

    Err(MalformedData("Invalid data".into()))?
}

fn extract_array(input: &str) -> Result<String, ExhaustedArray> {
    if !(input.starts_with("[") && input.ends_with("]")) {
        Err(ExhaustedArray)
    } else {
        Ok(input[1..input.len() - 1].to_string())
    }
}

pub fn run_transform_test(input: &str) -> Result<(), Error> {
    let four = input.lines().filter(|x| !x.is_empty()).collect::<Vec<_>>();
    if four.len() != 4 && four.len() != 6 {
        Err(MalformedData("Needed four or six lines as input".into()))?;
    }

    let del_a = comma_seq(&extract_array(four[0])?, parse_del)?;
    let add_a = comma_seq(&extract_array(four[1])?, parse_add)?;

    let del_b = comma_seq(&extract_array(four[2])?, parse_del)?;
    let add_b = comma_seq(&extract_array(four[3])?, parse_add)?;

    let a = (del_a, add_a);
    let b = (del_b, add_b);

    println!("transform start!");
    let confirm = op_transform_compare(&a, &b);

    // Check validating lines
    if four.len() == 6 {
        let confirm_del = comma_seq(&extract_array(four[4])?, parse_del)?;
        let confirm_add = comma_seq(&extract_array(four[5])?, parse_add)?;

        println!("Validating...");
        assert_eq!(confirm, (confirm_del, confirm_add));
        println!("Valid!");
    }

    // ALSO CHECK THE REVERSE
    // The result may be different, so we don't care it to
    // that, but we can check that the transform is at least normalized.
    let _ = op_transform_compare(&b, &a);

    Ok(())
}

fn op_transform_compare(a: &Op, b: &Op) -> Op {
    let (a_, b_) = transform(a, b);

    println!();
    println!(" --> a : {:?}", a);
    println!(" --> a': {:?}", a_);
    println!(" --> b : {:?}", b);
    println!(" --> b': {:?}", b_);
    println!();

    let a_res = normalize(compose::compose(a, &a_));
    let b_res = normalize(compose::compose(b, &b_));

    println!();
    println!("A' {:?}", a_res);
    println!("B' {:?}", b_res);
    println!();

    assert_eq!(a_res, b_res);

    a_res
}

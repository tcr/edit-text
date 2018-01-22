//! Parse doc_span, add_span, del_span like strings.

use super::OT;
use super::compose;
use super::doc::*;
use super::normalize;
use super::transform::*;
use super::validate::{validate_doc_span, ValidateContext};
use failure::Error;
use regex::Regex;
use serde_json;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Debug;
use std::io;
use std::io::prelude::*;
use yansi::Paint;

#[derive(Debug, Fail)]
#[fail(display = "Group has no readable data")]
pub struct NoReadableData;

#[derive(Debug, Fail)]
#[fail(display = "Malformed data: {}", _0)]
pub struct MalformedData(pub String);

#[derive(Debug, Fail)]
#[fail(display = "Exhausted array")]
pub struct ExhaustedArray;

fn comma_seq<T, F>(mut inner: &str, fnv: F) -> Result<Vec<T>, NoReadableData>
where
    F: Fn(&str) -> Result<T, Error>,
{
    let mut target = vec![];
    if inner.chars().all(|x| x.is_whitespace()) {
        return Ok(target);
    }

    let re = Regex::new(r"^\s*(,\s*)*$").unwrap();
    while !inner.is_empty() && !re.is_match(inner) {
        let mut item = None;
        for (comma, _) in {
            let mut a: Vec<_> = inner.match_indices(",").collect();
            a.push((inner.len() as usize, ""));
            a
        } {
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

fn extract_array(input: &str) -> Result<String, ExhaustedArray> {
    let input = input.trim();
    if !(input.starts_with("[") && input.ends_with("]")) {
        Err(ExhaustedArray)
    } else {
        Ok(input[1..input.len() - 1].to_string())
    }
}

fn parse_doc_elem(mut value: &str) -> Result<DocElement, Error> {
    value = value.trim();

    let cap = if value.starts_with("DocChars(") && value.ends_with(')') {
        Some(("DocChars", &value["DocChars(".len()..value.len() - 1]))
    } else {
        None
    };

    if let Some((key, segment)) = cap {
        return Ok(match key {
            "DocChars" => {
                if segment.len() < 2 || !segment.starts_with("\"") || !segment.ends_with("\"") {
                    Err(MalformedData("Expected full quoted string".into()))?;
                }
                let segment: Value = serde_json::from_str(&segment.replace("\\'", "'")).unwrap();
                DocElement::DocChars(segment.as_str().unwrap().to_string())
            }
            _ => unreachable!(),
        });
    }

    if value.starts_with("DocGroup({") && value.ends_with("])") {
        // panic!("this doesnt work yet");

        let right_index = value.find('}').unwrap();
        let body = &value["DocGroup(".len()..right_index + 1];

        // Find ending } then subslice then
        let v: Value = serde_json::from_str(body).unwrap();
        let mut map: HashMap<String, String> = hashmap![];
        for (key, value) in v.as_object().unwrap() {
            map.insert(key.to_string(), value.as_str().unwrap().to_string());
        }

        let next_index = value[right_index + 1..].find('[').unwrap();
        let inner = &value[right_index + 1 + next_index + 1..value.len() - 2];
        let args = comma_seq(inner, parse_doc_elem)?;

        return Ok(DocElement::DocGroup(map, args));
    }

    Err(MalformedData("Invalid data".into()))?
}

pub fn parse_doc_span(input: &str) -> Result<DocSpan, Error> {
    Ok(comma_seq(&extract_array(input)?, parse_doc_elem)?)
}

fn parse_del_elem(mut value: &str) -> Result<DelElement, Error> {
    value = value.trim();

    // if value == "DelGroupAll" {
    //     return Ok(DelElement::DelGroupAll);
    // }

    let cap = if value.starts_with("DelSkip(") && value.ends_with(')') {
        Some(("DelSkip", &value["DelSkip(".len()..value.len() - 1]))
    } else if value.starts_with("DelChars(") && value.ends_with(')') {
        Some(("DelChars", &value["DelChars(".len()..value.len() - 1]))
    // } else if value.starts_with("DelMany(") && value.ends_with(')') {
    //     Some(("DelMany", &value["DelMany(".len()..value.len() - 1]))
    } else {
        None
    };

    if let Some((key, segment)) = cap {
        let inner = segment.parse::<usize>()?;
        return Ok(match key {
            "DelSkip" => DelElement::DelSkip(inner),
            "DelChars" => DelElement::DelChars(inner),
            // "DelMany" => DelElement::DelMany(inner),
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
        let args = comma_seq(inner, parse_del_elem)?;

        return Ok(match key {
            "DelGroup" => DelElement::DelGroup(args),
            "DelWithGroup" => DelElement::DelWithGroup(args),
            _ => unreachable!("why 2"),
        });
    }
    Err(MalformedData("Invalid data".into()))?
}

pub fn parse_del_span(input: &str) -> Result<DelSpan, Error> {
    Ok(comma_seq(&extract_array(input)?, parse_del_elem)?)
}

fn parse_add_elem(mut value: &str) -> Result<AddElement, Error> {
    value = value.trim();

    let cap = if value.starts_with("AddSkip(") && value.ends_with(')') {
        Some(("AddSkip", &value["AddSkip(".len()..value.len() - 1]))
    } else if value.starts_with("AddChars(") && value.ends_with(')') {
        Some(("AddChars", &value["AddChars(".len()..value.len() - 1]))
    } else {
        None
    };

    if let Some((key, segment)) = cap {
        return Ok(match key {
            "AddSkip" => AddElement::AddSkip(try!(segment.parse::<usize>().or_else(|_| Err(
                io::Error::new(io::ErrorKind::InvalidData, "Bad parsing of number",)
            )))),
            "AddChars" => {
                if segment.len() < 2 || !segment.starts_with("\"") || !segment.ends_with("\"") {
                    Err(MalformedData("Expected full quoted string".into()))?;
                }
                let segment: Value = serde_json::from_str(&segment.replace("\\'", "'")).unwrap();
                AddElement::AddChars(segment.as_str().unwrap().to_string())
            }
            _ => unreachable!(),
        });
    }

    // let mut cap = None;
    if value.starts_with("AddWithGroup([") && value.ends_with("])") {
        let inner = &value["AddWithGroup([".len()..value.len() - 2];

        let args = comma_seq(inner, parse_add_elem)?;

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
        let args = comma_seq(inner, parse_add_elem)?;

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

pub fn parse_add_span(input: &str) -> Result<AddSpan, Error> {
    Ok(comma_seq(&extract_array(input)?, parse_add_elem)?)
}

// TODO move this somewhere better
pub fn debug_pretty<D: Debug>(input: &D) -> String {
    let input = format!("{:?}", input);

    let mut out = String::new();
    let mut len = "".to_string();
    let mut chars = input.chars().peekable();
    loop {
        let mut c = if let Some(c) = chars.next() {
            c
        } else {
            break;
        };
        if c == '[' {
            out.push(c);

            while chars.peek().unwrap().is_whitespace() {
                let _ = chars.next();
            }
            if chars.peek() == Some(&']') {
                c = chars.next().unwrap();
            } else {
                out.push_str("\n");

                len.push_str("    ");
                out.push_str(&len);
            }
        } else if c == ']' {
            len = len[0..len.len() - 4].to_string();
            out.push_str("\n");
            out.push_str(&len);
        } else if c == '\n' {
            out.push(c);
            out.push_str(&len);
        } else {
            out.push(c);
        }

        if c == ']' {
            out.push(c);
            if chars.peek() == Some(&')') {
                out.push(chars.next().unwrap());
                if chars.peek() == Some(&',') {
                    out.push(chars.next().unwrap());
                    while chars.peek().unwrap().is_whitespace() {
                        let _ = chars.next();
                    }
                    out.push_str("\n");
                    out.push_str(&len);
                }
            }
        }
    }
    out
}

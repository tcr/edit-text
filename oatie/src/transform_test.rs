use std::io;
use std::io::prelude::*;
use serde_json::Value;
use std::collections::HashMap;

use super::compose;
use super::doc::*;
use super::normalize;
use super::{OT, Operation};
use super::schema::{ValidateContext, validate_doc_span};
use super::transform::*;
use serde_json;
use regex::Regex;
use failure::Error;
use debug_pretty;
use yansi::Paint;

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
    if inner.chars().all(|x| x.is_whitespace()) {
        return Ok(target)
    }

    let re = Regex::new(r"^\s*(,\s*)*$").unwrap();
    while !inner.is_empty() && !re.is_match(inner) {
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

fn op_transform_compare(a: &Op, b: &Op) -> (Op, Op, Op, Op) {
    let (a_, b_) = transform(a, b);

    println!();
    println!(" --> a \n{:?}", a);
    println!(" --> a'\n{:?}", a_);
    println!(" --> b \n{:?}", b);
    println!(" --> b'\n{:?}", b_);
    println!();

    let a_res = normalize(compose::compose(a, &a_));
    let b_res = normalize(compose::compose(b, &b_));

    // a_res.0 = normalize_delgroupall(a_res.0);
    // b_res.0 = normalize_delgroupall(b_res.0);

    println!();
    println!("(!) validating composed ops are equivalent:");
    println!(" --> a : a'\n{:?}", a_res);
    println!(" --> b : b'\n{:?}", b_res);
    println!();

    assert_eq!(a_res, b_res);

    (a_, b_, a_res, b_res)
}

pub fn run_transform_test(input: &str) -> Result<(), Error> {
    let re = Regex::new(r"(\n|^)(\w+):([\n\w\W]+?)(\n(?:\w)|(\n\]))").unwrap();
    let mut test: HashMap<_, _> = re.captures_iter(&input)
        .map(|cap| {
            let name = cap[2].to_string();
            let end_cap = cap.get(5).map(|x| x.as_str()).unwrap_or("");
            let body = [&cap[3], end_cap].join("");
            (name, body)
        })
        .collect();

    // Attempt old-style transform test which matches by line.
    if test.len() == 0 {
        let four = input.lines().filter(|x| !x.is_empty()).collect::<Vec<_>>();
        if four.len() != 4 && four.len() != 6 {
            Err(MalformedData("Needed four or six lines as input".into()))?;
        }

        test.insert("a_del".into(), four[0].into());
        test.insert("a_add".into(), four[1].into());

        test.insert("b_del".into(), four[2].into());
        test.insert("b_add".into(), four[3].into());

        // Check validating lines
        if four.len() == 6 {
            test.insert("op_del".into(), four[4].into());
            test.insert("op_add".into(), four[5].into());
        }
    }

    println!("entries {:?}", test.keys().collect::<Vec<_>>());

    // Extract test entries.
    let a = (parse_del_span(&test["a_del"])?, parse_add_span(&test["a_add"])?);
    let b = (parse_del_span(&test["b_del"])?, parse_add_span(&test["b_add"])?);
    let check = if test.contains_key("op_del") || test.contains_key("op_add") {
        Some((parse_del_span(&test["op_del"])?, parse_add_span(&test["op_add"])?))
    } else {
        None
    };

    // Check that transforms produce identical operations when composed.
    println!("{}", Paint::red("(!) comparing transform operation results..."));
    let (a_, b_, a_res, b_res) = op_transform_compare(&a, &b);
    println!("ok");
    println!();

    // Check validating lines.
    if let Some(check) = check {
        println!("{}", Paint::red("(!) validating client A against provided 'check' op..."));
        assert_eq!(a_res, check);
        println!("ok");
        println!();
    }

    // Check against provided document.
    if let Some(doc) = test.get("doc") {
        println!("{}", Paint::red("(!) validating docs..."));

        let doc = Doc(parse_doc_span(doc)?);
        println!("original document: {:?}", doc);
        validate_doc_span(&mut ValidateContext::new(), &doc.0)?;
        println!();

        // First test original operations can be applied against the doc.
        // (This should always pass.)
        println!("{}", Paint::red("(!) applying first ops..."));
        println!(" ---> doc a : a");
        let doc_a = OT::apply(&doc, &a);
        println!("{:?}", doc_a);
        println!();
        println!(" ---> doc b : b");
        let doc_b = OT::apply(&doc, &b);
        println!("{:?}", doc_b);
        println!();
        println!("ok");
        println!();

        // Next apply the transformed ops.
        println!("{}", Paint::red("(!) applying transformed ops..."));
        println!(" ---> doc a : a : a'");
        let doc_a = OT::apply(&doc_a, &a_);
        println!("{:?}", doc_a);
        validate_doc_span(&mut ValidateContext::new(), &doc_a.0)?;
        println!(" ---> doc b : b : b'");
        let doc_b = OT::apply(&doc_b, &b_);
        println!("{:?}", doc_b);
        validate_doc_span(&mut ValidateContext::new(), &doc_b.0)?;
        println!();
        println!("ok");
        println!();

        // Next test them composed.
        println!("{}", Paint::red("(!) testing op composed (double check)..."));
        println!(" ---> doc a : (a : a')");
        let doc_a_cmp = OT::apply(&doc, &Operation::compose(&a, &a_));
        println!("{}", debug_pretty(&doc));
        println!();
        println!("{}", debug_pretty(&Operation::compose(&a, &a_)));
        println!("{}", debug_pretty(&doc_a_cmp));
        validate_doc_span(&mut ValidateContext::new(), &doc_a_cmp.0)?;
        println!(" ---> doc b : (b : b')");
        let doc_b_cmp = OT::apply(&doc, &Operation::compose(&a, &a_));
        println!("{}", debug_pretty(&doc_b_cmp));
        validate_doc_span(&mut ValidateContext::new(), &doc_b_cmp.0)?;
        println!();
        println!("ok");
        println!();

        // Next test transforms can produce identical documents.
        // TODO
    }

    // ALSO CHECK THE REVERSE
    // The result may be different, so we don't care it to
    // that, but we can check that the transform is at least normalized.
    // let _ = op_transform_compare(&b, &a);

    println!("{}", Paint::green("(!) done."));

    Ok(())
}
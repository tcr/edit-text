//! Helper methods for performing transform tests.

extern crate ron;

use serde_json::Value;
use std::collections::HashMap;
use std::io;
use std::io::prelude::*;

use super::compose;
use super::doc::*;
use super::normalize::*;
use super::parse::debug_pretty;
use super::transform::*;
use super::validate::{
    validate_doc_span,
    ValidateContext,
};
use super::OT;
use failure::Error;
use parse::*;
use regex::Regex;
use serde_json;
use yansi::Paint;

fn op_transform_compare<T: Schema>(a: &Op, b: &Op) -> (Op, Op, Op, Op) {
    let (a_, b_) = transform::<T>(a, b);

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

#[derive(Serialize, Deserialize, Debug)]
enum TestSpec {
    TransformTest { doc: DocSpan, a: Op, b: Op },
}

pub fn run_transform_test<T: Schema>(input: &str) -> Result<(), Error> {
    let mut test: HashMap<String, String> = HashMap::new();

    // ron-defined test specs
    if input.find("TransformTest").is_some() {
        match ron::de::from_str::<TestSpec>(input)? {
            TestSpec::TransformTest {
                ref doc,
                ref a,
                ref b,
            } => {
                test.insert("doc".into(), ron::ser::to_string(&doc)?);
                test.insert("a_del".into(), ron::ser::to_string(&a.0)?);
                test.insert("a_add".into(), ron::ser::to_string(&a.1)?);
                test.insert("b_del".into(), ron::ser::to_string(&b.0)?);
                test.insert("b_add".into(), ron::ser::to_string(&b.1)?);
            }
        }
    // line by line
    } else {
        let re = Regex::new(r"(\n|^)(\w+):([\n\w\W]+?)(\n(?:\w)|(\n\]))").unwrap();
        let res: HashMap<String, String> = re
            .captures_iter(&input)
            .map(|cap| {
                let name = cap[2].to_string();
                let end_cap = cap.get(5).map(|x| x.as_str()).unwrap_or("");
                let body = [&cap[3], end_cap].join("");
                (name, body)
            })
            .collect();
        test.extend(res);
    }

    // Attempt old-style transform test which matches by line.
    if test.len() == 0 {
        let four = input.lines().filter(|x| !x.is_empty()).collect::<Vec<_>>();
        if four.len() != 4 && four.len() != 6 {
            bail!("Needed four or six lines as input");
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
    let a = (
        ron::de::from_str::<DelSpan>(&test["a_del"])?,
        ron::de::from_str::<AddSpan>(&test["a_add"])?,
    );
    let b = (
        ron::de::from_str::<DelSpan>(&test["b_del"])?,
        ron::de::from_str::<AddSpan>(&test["b_add"])?,
    );
    let check = if test.contains_key("op_del") || test.contains_key("op_add") {
        Some((
            ron::de::from_str::<DelSpan>(&test["op_del"])?,
            ron::de::from_str::<AddSpan>(&test["op_add"])?,
        ))
    } else {
        None
    };

    // Check that transforms produce identical operations when composed.
    println!(
        "{}",
        Paint::red("(!) comparing transform operation results...")
    );
    let (a_, b_, a_res, b_res) = op_transform_compare::<T>(&a, &b);
    println!("ok");
    println!();

    // Check validating lines.
    if let Some(check) = check {
        println!(
            "{}",
            Paint::red("(!) validating client A against provided 'check' op...")
        );
        assert_eq!(a_res, check);
        println!("ok");
        println!();
    }

    // Check against provided document.
    if let Some(doc) = test.get("doc") {
        println!("{}", Paint::red("(!) validating docs..."));

        let doc = Doc(ron::de::from_str::<DocSpan>(doc)?);
        println!("original document: {:?}", doc);
        validate_doc_span(&mut ValidateContext::new(), &doc.0)?;
        println!();

        // First test original operations can be applied against the doc.
        // (This should always pass.)
        println!("{}", Paint::red("(!) applying first ops..."));
        println!(" ---> doc a : a");
        let doc_a = Op::apply(&doc, &a);
        println!("{:?}", doc_a);
        println!();
        println!(" ---> doc b : b");
        let doc_b = Op::apply(&doc, &b);
        println!("{:?}", doc_b);
        println!();
        println!("ok");
        println!();

        // Next apply the transformed ops.
        println!("{}", Paint::red("(!) applying transformed ops..."));
        println!(" ---> doc a : a : a'");
        let doc_a = Op::apply(&doc_a, &a_);
        println!("{:?}", doc_a);
        validate_doc_span(&mut ValidateContext::new(), &doc_a.0)?;
        println!(" ---> doc b : b : b'");
        let doc_b = Op::apply(&doc_b, &b_);
        println!("{:?}", doc_b);
        validate_doc_span(&mut ValidateContext::new(), &doc_b.0)?;
        println!();
        println!("ok");
        println!();

        // Next test them composed.
        println!(
            "{}",
            Paint::red("(!) testing op composed (double check)...")
        );
        println!(" ---> doc a : (a : a')");
        let doc_a_cmp = Op::apply(&doc, &Op::compose(&a, &a_));
        println!("{}", debug_pretty(&doc));
        println!();
        println!("{}", debug_pretty(&Op::compose(&a, &a_)));
        println!("{}", debug_pretty(&doc_a_cmp));
        validate_doc_span(&mut ValidateContext::new(), &doc_a_cmp.0)?;
        println!(" ---> doc b : (b : b')");
        let doc_b_cmp = Op::apply(&doc, &Op::compose(&a, &a_));
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

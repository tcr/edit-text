//! ```cargo
//! [dependencies]
//! ron = "*"
//! serde = "*"
//! serde_derive = "*"
//! maplit = "*"
//! failure = "*"
//! serde_with = "*"
//! ```

#![feature(extern_in_paths)]

use extern::{
    ron,
    failure::Error,
    serde_derive::*,
    serde::{self, Deserialize, Serialize, Serializer, Deserializer},
    maplit::*,
    std::{
        collections::HashMap,
    },
    std::hash::Hash,
};


#[repr(u64)]
#[derive(Serialize, Hash, PartialEq, Eq)]
enum Style {
    Bold,
    Italic,
}

#[derive(Serialize)]
struct DocString(
    String,

    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "serialize_option")]
    Option<HashMap<Style, Option<String>>>,
);

impl DocString {
    fn from_str(value: &str) -> DocString {
        DocString(
            value.to_string(),
            None,
        )
    }

    fn from_str_styles(
        value: &str,
        mut styles: HashMap<Style, Option<String>>,
    ) -> DocString {
        DocString(
            value.to_string(),
            // Some(styles.drain().map(|(k, v)| (k.into(), v)).collect()),
            Some(styles),
        )
    }
}

fn serialize_option<T, S>(option: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
where
    T: Serialize,
    S: Serializer,
{
    option.as_ref().unwrap().serialize(serializer)
}

fn to_ron_string(value: &DocString) -> Result<String, Error> {
    Ok(ron::ser::to_string(value)?)
}

fn main() {
    use self::Style::*;

    let val = DocString::from_str("hi");
    let a = ron::ser::to_string(&val).unwrap();
    eprintln!("{}", a);

    let val = DocString::from_str_styles("hi", hashmap![
        Bold => None,
    ]);
    let a = to_ron_string(&val).unwrap();
    eprintln!("{}", a);
}
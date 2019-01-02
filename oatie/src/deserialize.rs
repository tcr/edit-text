//! Update legacy serialization formats to a modern format.

use crate::doc::*;
use failure::Error;

// ISSUE Deduplicate v1 and v2 parsing logic
//
// The logic governing these two formats is almost identical, except that in v1
// styles are represented as a hashmap (that supports a value Link(String)) and
// in v2 they are represented as a set. This might be possible to dynamically
// generate using serde parsing logic (e.g. implemented on a visitor) or maybe
// it's possible to specify which version of the code to try and just change the
// behavior of the one function that parses styles.

/// Decoding "v1" docs, which used an ad-hoc serde format as the data types
/// evolved. There's no specification for this, just compatibility with old
/// code (test cases and database storage).
pub mod v1 {
    use failure::*;
    use serde::de::{
        self,
        SeqAccess,
        Visitor,
    };
    use serde::{
        Deserialize,
        Deserializer,
    };
    use std::collections::HashMap;
    use std::fmt;

    #[derive(Deserialize, PartialEq, Eq, Clone, Debug, Hash)]
    pub enum Style {
        Normie,
        Bold,
        Italic,
        Link,
    }

    pub struct DocString(String, HashMap<Style, Option<String>>);

    // DocString can be serialized as `String` or `[String]`
    impl<'de> Deserialize<'de> for DocString {
        fn deserialize<D>(deserializer: D) -> Result<DocString, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct FieldVisitor;

            impl<'de> Visitor<'de> for FieldVisitor {
                type Value = DocString;

                fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                    formatter.write_str("docstring")
                }

                fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
                where
                    V: SeqAccess<'de>,
                {
                    // Deserialize the one we care about.
                    let ret: String = seq
                        .next_element()?
                        .ok_or_else(|| de::Error::invalid_length(1, &self))?;

                    let styles: HashMap<Style, Option<String>> = seq
                        .next_element()?
                        .ok_or_else(|| de::Error::invalid_length(1, &self))?;

                    Ok(DocString(ret, styles))
                }

                fn visit_str<E>(self, value: &str) -> Result<DocString, E>
                where
                    E: de::Error,
                {
                    Ok(DocString(value.to_owned(), HashMap::new()))
                }
            }

            deserializer.deserialize_any(FieldVisitor)
        }
    }

    pub type DocSpan = Vec<DocElement>;

    #[derive(Deserialize)]
    pub enum DocElement {
        DocGroup(HashMap<String, String>, DocSpan),
        DocChars(DocString),
    }

    pub type DelSpan = Vec<DelElement>;

    #[derive(Deserialize)]
    pub enum DelElement {
        DelSkip(usize),
        DelWithGroup(DelSpan),
        DelChars(usize),
        DelGroup(DelSpan),
        DelStyles(usize, Vec<Style>),
    }

    pub type AddSpan = Vec<AddElement>;

    #[derive(Deserialize)]
    pub enum AddElement {
        AddSkip(usize),
        AddWithGroup(AddSpan),
        AddChars(DocString),
        AddGroup(HashMap<String, String>, AddSpan),
        AddStyles(usize, Vec<Style>),
    }

    fn update_attrs(input: HashMap<String, String>) -> Result<crate::rtf::Attrs, Error> {
        Ok(
            match input
                .get("tag")
                .ok_or(format_err!("No tag found in unpacked group"))?
                .as_str()
            {
                "h1" => crate::rtf::Attrs::Header(1),
                "h2" => crate::rtf::Attrs::Header(2),
                "h3" => crate::rtf::Attrs::Header(3),
                "h4" => crate::rtf::Attrs::Header(4),
                "h5" => crate::rtf::Attrs::Header(5),
                "h6" => crate::rtf::Attrs::Header(6),
                "pre" => crate::rtf::Attrs::Code,
                "html" => crate::rtf::Attrs::Html,
                "hr" => crate::rtf::Attrs::Rule,
                "bullet" => crate::rtf::Attrs::ListItem,
                "caret" => crate::rtf::Attrs::Caret {
                    client_id: input
                        .get("client")
                        .map(|x| x.to_owned())
                        .unwrap_or("unnamed".to_string()),
                    focus: input.get("focus").map(|x| x == "true").unwrap_or(true),
                },
                "p" | _ => crate::rtf::Attrs::Para,
            },
        )
    }

    fn update_styles(styles: Vec<Style>) -> Result<crate::rtf::StyleSet, Error> {
        let mut set = std::collections::HashSet::new();
        for style in styles {
            match style {
                // Normie is no longer used; it was a sentinel that styles were
                // working by visually rendering non-Normie styled text in
                // a bizarre and noticeable way.
                Style::Normie => {}
                Style::Bold => {
                    set.insert(crate::rtf::RtfStyle::Bold);
                }
                Style::Italic => {
                    set.insert(crate::rtf::RtfStyle::Italic);
                }
                Style::Link => {} // No link info included
            }
        }
        Ok(crate::rtf::StyleSet::from(set))
    }

    fn update_docspan(input: DocSpan) -> Result<crate::doc::DocSpan<crate::rtf::RtfSchema>, Error> {
        let mut output = vec![];
        for item in input {
            output.push(match item {
                DocElement::DocGroup(attrs, span) => {
                    crate::doc::DocGroup(update_attrs(attrs)?, update_docspan(span)?)
                }
                DocElement::DocChars(string) => crate::doc::DocText(
                    update_styles(string.1.into_iter().map(|(k, _v)| k).collect())?,
                    crate::doc::DocString::from_string(string.0),
                ),
            });
        }
        Ok(output)
    }

    fn update_addspan(input: AddSpan) -> Result<crate::doc::AddSpan<crate::rtf::RtfSchema>, Error> {
        let mut output = vec![];
        for item in input {
            output.push(match item {
                AddElement::AddSkip(skip) => crate::doc::AddSkip(skip),
                AddElement::AddWithGroup(span) => crate::doc::AddWithGroup(update_addspan(span)?),
                AddElement::AddChars(string) => crate::doc::AddText(
                    update_styles(string.1.into_iter().map(|(k, _v)| k).collect())?,
                    crate::doc::DocString::from_string(string.0),
                ),
                AddElement::AddGroup(attrs, span) => {
                    crate::doc::AddGroup(update_attrs(attrs)?, update_addspan(span)?)
                }
                AddElement::AddStyles(skip, styles) => {
                    crate::doc::AddStyles(skip, update_styles(styles)?)
                }
            });
        }
        Ok(output)
    }

    fn update_delspan(input: DelSpan) -> Result<crate::doc::DelSpan<crate::rtf::RtfSchema>, Error> {
        let mut output = vec![];
        for item in input {
            output.push(match item {
                DelElement::DelSkip(skip) => crate::doc::DelSkip(skip),
                DelElement::DelWithGroup(span) => crate::doc::DelWithGroup(update_delspan(span)?),
                DelElement::DelChars(skip) => crate::doc::DelText(skip),
                DelElement::DelGroup(span) => crate::doc::DelGroup(update_delspan(span)?),
                DelElement::DelStyles(skip, styles) => {
                    crate::doc::DelStyles(skip, update_styles(styles)?)
                }
            });
        }
        Ok(output)
    }

    pub fn docspan_ron(input: &str) -> Result<crate::doc::DocSpan<crate::rtf::RtfSchema>, Error> {
        update_docspan(ron::de::from_str(input)?)
    }

    pub fn docspan_json(input: &str) -> Result<crate::doc::DocSpan<crate::rtf::RtfSchema>, Error> {
        update_docspan(serde_json::from_str(input)?)
    }

    pub fn delspan_ron(input: &str) -> Result<crate::doc::DelSpan<crate::rtf::RtfSchema>, Error> {
        update_delspan(ron::de::from_str(input)?)
    }

    pub fn delspan_json(input: &str) -> Result<crate::doc::DelSpan<crate::rtf::RtfSchema>, Error> {
        update_delspan(serde_json::from_str(input)?)
    }

    pub fn addspan_ron(input: &str) -> Result<crate::doc::AddSpan<crate::rtf::RtfSchema>, Error> {
        update_addspan(ron::de::from_str(input)?)
    }

    pub fn addspan_json(input: &str) -> Result<crate::doc::AddSpan<crate::rtf::RtfSchema>, Error> {
        update_addspan(serde_json::from_str(input)?)
    }
}

/// Decoding "v2" docs, which used an ad-hoc serde format as the data types
/// evolved. There's no specification for this, just compatibility with old
/// code (test cases and database storage).
pub mod v2 {
    use failure::*;
    use serde::de::{
        self,
        SeqAccess,
        Visitor,
    };
    use serde::{
        Deserialize,
        Deserializer,
    };
    use std::collections::HashMap;
    use std::fmt;

    #[derive(Deserialize)]
    pub enum Style {
        Normie,
        Bold,
        Italic,
        Link,
    }

    pub struct DocString(String);

    // DocString can be serialized as `String` or `[String]`
    impl<'de> Deserialize<'de> for DocString {
        fn deserialize<D>(deserializer: D) -> Result<DocString, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct FieldVisitor;

            impl<'de> Visitor<'de> for FieldVisitor {
                type Value = DocString;

                fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                    formatter.write_str("docstring")
                }

                fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
                where
                    V: SeqAccess<'de>,
                {
                    // Deserialize the one we care about.
                    let ret: String = seq
                        .next_element()?
                        .ok_or_else(|| de::Error::invalid_length(1, &self))?;

                    Ok(DocString(ret))
                }

                fn visit_str<E>(self, value: &str) -> Result<DocString, E>
                where
                    E: de::Error,
                {
                    Ok(DocString(value.to_owned()))
                }
            }

            deserializer.deserialize_any(FieldVisitor)
        }
    }

    pub type DocSpan = Vec<DocElement>;

    #[derive(Deserialize)]
    pub enum DocElement {
        DocGroup(HashMap<String, String>, DocSpan),
        DocChars(DocString, #[serde(default)] Vec<Style>),
    }

    pub type DelSpan = Vec<DelElement>;

    #[derive(Deserialize)]
    pub enum DelElement {
        DelSkip(usize),
        DelWithGroup(DelSpan),
        DelChars(usize),
        DelGroup(DelSpan),
        DelStyles(usize, Vec<Style>),
    }

    pub type AddSpan = Vec<AddElement>;

    #[derive(Deserialize)]
    pub enum AddElement {
        AddSkip(usize),
        AddWithGroup(AddSpan),
        AddChars(DocString, #[serde(default)] Vec<Style>),
        AddGroup(HashMap<String, String>, AddSpan),
        AddStyles(usize, Vec<Style>),
    }

    fn update_attrs(input: HashMap<String, String>) -> Result<crate::rtf::Attrs, Error> {
        Ok(
            match input
                .get("tag")
                .ok_or(format_err!("No tag found in unpacked group"))?
                .as_str()
            {
                "h1" => crate::rtf::Attrs::Header(1),
                "h2" => crate::rtf::Attrs::Header(2),
                "h3" => crate::rtf::Attrs::Header(3),
                "h4" => crate::rtf::Attrs::Header(4),
                "h5" => crate::rtf::Attrs::Header(5),
                "h6" => crate::rtf::Attrs::Header(6),
                "pre" => crate::rtf::Attrs::Code,
                "html" => crate::rtf::Attrs::Html,
                "hr" => crate::rtf::Attrs::Rule,
                "bullet" => crate::rtf::Attrs::ListItem,
                "caret" => crate::rtf::Attrs::Caret {
                    client_id: input
                        .get("client")
                        .map(|x| x.to_owned())
                        .unwrap_or("unnamed".to_string()),
                    focus: input.get("focus").map(|x| x == "true").unwrap_or(true),
                },
                "p" | _ => crate::rtf::Attrs::Para,
            },
        )
    }

    fn update_styles(styles: Vec<Style>) -> Result<crate::rtf::StyleSet, Error> {
        let mut set = std::collections::HashSet::new();
        for style in styles {
            match style {
                // Normie is no longer used; it was a sentinel that styles were
                // working by visually rendering non-Normie styled text in
                // a bizarre and noticeable way.
                Style::Normie => {}
                Style::Bold => {
                    set.insert(crate::rtf::RtfStyle::Bold);
                }
                Style::Italic => {
                    set.insert(crate::rtf::RtfStyle::Italic);
                }
                Style::Link => {} // No link info included
            }
        }
        Ok(crate::rtf::StyleSet::from(set))
    }

    fn update_docspan(input: DocSpan) -> Result<crate::doc::DocSpan<crate::rtf::RtfSchema>, Error> {
        let mut output = vec![];
        for item in input {
            output.push(match item {
                DocElement::DocGroup(attrs, span) => {
                    crate::doc::DocGroup(update_attrs(attrs)?, update_docspan(span)?)
                }
                DocElement::DocChars(string, styles) => crate::doc::DocText(
                    update_styles(styles)?,
                    crate::doc::DocString::from_string(string.0),
                ),
            });
        }
        Ok(output)
    }

    fn update_addspan(input: AddSpan) -> Result<crate::doc::AddSpan<crate::rtf::RtfSchema>, Error> {
        let mut output = vec![];
        for item in input {
            output.push(match item {
                AddElement::AddSkip(skip) => crate::doc::AddSkip(skip),
                AddElement::AddWithGroup(span) => crate::doc::AddWithGroup(update_addspan(span)?),
                AddElement::AddChars(string, styles) => crate::doc::AddText(
                    update_styles(styles)?,
                    crate::doc::DocString::from_string(string.0),
                ),
                AddElement::AddGroup(attrs, span) => {
                    crate::doc::AddGroup(update_attrs(attrs)?, update_addspan(span)?)
                }
                AddElement::AddStyles(skip, styles) => {
                    crate::doc::AddStyles(skip, update_styles(styles)?)
                }
            });
        }
        Ok(output)
    }

    fn update_delspan(input: DelSpan) -> Result<crate::doc::DelSpan<crate::rtf::RtfSchema>, Error> {
        let mut output = vec![];
        for item in input {
            output.push(match item {
                DelElement::DelSkip(skip) => crate::doc::DelSkip(skip),
                DelElement::DelWithGroup(span) => crate::doc::DelWithGroup(update_delspan(span)?),
                DelElement::DelChars(skip) => crate::doc::DelText(skip),
                DelElement::DelGroup(span) => crate::doc::DelGroup(update_delspan(span)?),
                DelElement::DelStyles(skip, styles) => {
                    crate::doc::DelStyles(skip, update_styles(styles)?)
                }
            });
        }
        Ok(output)
    }

    pub fn docspan_ron(input: &str) -> Result<crate::doc::DocSpan<crate::rtf::RtfSchema>, Error> {
        update_docspan(ron::de::from_str(input)?)
    }

    pub fn docspan_json(input: &str) -> Result<crate::doc::DocSpan<crate::rtf::RtfSchema>, Error> {
        update_docspan(serde_json::from_str(input)?)
    }

    pub fn delspan_ron(input: &str) -> Result<crate::doc::DelSpan<crate::rtf::RtfSchema>, Error> {
        update_delspan(ron::de::from_str(input)?)
    }

    pub fn delspan_json(input: &str) -> Result<crate::doc::DelSpan<crate::rtf::RtfSchema>, Error> {
        update_delspan(serde_json::from_str(input)?)
    }

    pub fn addspan_ron(input: &str) -> Result<crate::doc::AddSpan<crate::rtf::RtfSchema>, Error> {
        update_addspan(ron::de::from_str(input)?)
    }

    pub fn addspan_json(input: &str) -> Result<crate::doc::AddSpan<crate::rtf::RtfSchema>, Error> {
        update_addspan(serde_json::from_str(input)?)
    }
}

pub fn doc_ron(input: &str) -> Result<crate::doc::Doc<crate::rtf::RtfSchema>, Error> {
    // V3
    match ron::de::from_str::<crate::doc::Doc<crate::rtf::RtfSchema>>(input) {
        Ok(value) => Ok(value),
        Err(err) => {
            // V3a Possibly a DocSpan not a Doc?
            if let Ok(value) = ron::de::from_str::<crate::doc::DocSpan<crate::rtf::RtfSchema>>(input) {
                return Ok(Doc(value));
            }

            // Try V2 encoding.
            if let Ok(value) = v2::docspan_ron(&input) {
                return Ok(Doc(value));
            }

            // Try V1 encoding.
            if let Ok(value) = v1::docspan_ron(&input) {
                return Ok(Doc(value));
            }

            // Throw original ron error if error is encountered.
            Err(err.into())
        }
    }
}

pub fn doc_json(input: &str) -> Result<crate::doc::Doc<crate::rtf::RtfSchema>, Error> {
    match serde_json::from_str::<crate::doc::Doc<crate::rtf::RtfSchema>>(&input) {
        Ok(value) => Ok(value),
        Err(err) => {
            // Try V2 encoding.
            if let Ok(value) = v2::docspan_json(&input) {
                return Ok(Doc(value));
            }

            // Try V1 encoding.
            if let Ok(value) = v1::docspan_json(&input) {
                return Ok(Doc(value));
            }
            
            // Throw original ron error if error is encountered.
            Err(err.into())
        }
    }
}

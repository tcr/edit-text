use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use std::ops::Range;
use serde::ser::SerializeSeq;
use serde::de::{self, SeqAccess, Visitor};
use std::fmt;

#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Style {
    Fancy,
    Selected, // never used except on the client
    Bold,
}

impl fmt::Display for Style {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

pub type StyleMap = HashMap<Style, Option<String>>;

/// Abstraction for String that allows a limited set of operations
/// with good optimization. (Or that's the idea.)
#[derive(Clone, Debug)]
pub struct DocString(
    Arc<String>,
    Option<Range<usize>>,
    Option<StyleMap>,
);

impl DocString {
    pub fn from_string(input: String) -> DocString {
        DocString(Arc::new(input), None, None)
    }

    pub fn from_str(input: &str) -> DocString {
        DocString(Arc::new(input.to_owned()), None, None)
    }

    pub fn from_string_styled(input: String, styles: StyleMap) -> DocString {
        DocString(Arc::new(input), None, Some(styles))
    }

    pub fn from_str_styled(input: &str, styles: StyleMap) -> DocString {
        DocString(Arc::new(input.to_owned()), None, Some(styles))
    }

    // TODO audit use of this
    pub fn as_str(&self) -> &str {
        if let Some(ref range) = self.1 {
            &self.0[range.clone()]
        } else {
            &self.0
        }
    }

    pub fn styles(&self) -> &Option<StyleMap> {
        &self.2
    }

    pub fn push_str(&mut self, input: &str) {
        let mut value = self.to_string();
        value.push_str(input);
        self.0 = Arc::new(value);
        self.1 = None;
    }

    pub fn push_doc_string(&mut self, input: &DocString) {
        self.push_str(input.as_str());
    }

    // TODO consume self?
    pub fn split_at(&self, char_boundary: usize) -> (DocString, DocString) {
        let (byte_index, _) = self.as_str().char_indices().skip(char_boundary).next().unwrap();
        let mut start = 0;
        let mut end = self.0.len();
        if let Some(ref range) = self.1 {
            start = range.start;
            end = range.end;
        }
        (
            DocString(self.0.clone(), Some((start + 0)..(start + byte_index)), self.2.clone()),
            DocString(self.0.clone(), Some((start + byte_index)..end), self.2.clone()),
        )
    }

    pub fn to_string(&self) -> String {
        self.as_str().to_owned()
    }

    pub fn is_empty(&self) -> bool {
        self.as_str().is_empty()
    }

    pub fn into_string(self) -> String {
        // TODO make this faster by deconstructing the Rc?
        self.to_string()
    }

    pub fn char_len(&self) -> usize {
        self.as_str().chars().count()
    }
}

impl PartialEq for DocString {
    fn eq(&self, other: &DocString) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for DocString {}

impl Serialize for DocString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let &Some(ref value) = &self.2 {
            let mut s = serializer.serialize_seq(Some(2))?;
            s.serialize_element(self.as_str())?;
            s.serialize_element(value)?;
            s.end()
        } else {
            serializer.serialize_str(self.as_str())
        }
    }
}

impl<'de> Deserialize<'de> for DocString {
    fn deserialize<D>(deserializer: D) -> Result<DocString, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct FieldVisitor;

        impl<'de> Visitor<'de> for FieldVisitor {
            type Value = DocString;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("docstring")
            }

            fn visit_str<E>(self, value: &str) -> Result<DocString, E>
            where
                E: de::Error,
            {
                Ok(DocString::from_str(value))
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<DocString, A::Error>
            where
                A: SeqAccess<'de>,
            {
                if let Some(inner) = seq.next_element::<String>()? {
                    if let Some(styles) = seq.next_element::<StyleMap>()? {
                        Ok(DocString::from_string_styled(inner, styles))
                    } else {
                        Err(de::Error::unknown_field("1", FIELDS))
                    }
                } else {
                    Err(de::Error::unknown_field("0", FIELDS))
                }
            }
        }

        const FIELDS: &'static [&'static str] = &["docstring"];
        deserializer.deserialize_any(FieldVisitor)
    }
}

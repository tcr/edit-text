use serde::de::{
    self,
    SeqAccess,
    Visitor,
};
use serde::{
    Deserialize,
    Deserializer,
    Serialize,
    Serializer,
};
use std::fmt;
use std::ops::Range;
use std::sync::Arc;

/// Abstraction for String that has better performance by restricting its API.
/// It can also be styled using the Style enum.
#[derive(Clone, Debug)]
pub struct DocString {
    string: Arc<String>,
    range: Range<usize>,
}

impl DocString {
    pub fn from_string(input: String) -> DocString {
        let range = 0..input.len();
        DocString {
            string: Arc::new(input),
            range,
        }
    }

    pub fn from_str(input: &str) -> DocString {
        let range = 0..input.len();
        DocString {
            string: Arc::new(input.to_owned()),
            range,
        }
    }

    // TODO audit use of this
    pub fn as_str(&self) -> &str {
        &self.string[self.range.clone()]
    }

    // Add text (with the same styling) to the end of this string.
    pub fn push_str(&mut self, input: &str) {
        let mut value = self.to_string();
        value.push_str(input);
        self.range = 0..value.len();
        self.string = Arc::new(value);
    }

    // TODO Should DocString::split_at consume self instead of &mut self?
    pub fn split_at(&self, char_boundary: usize) -> (DocString, DocString) {
        let start = self.range.start;
        let end = self.range.end;

        let byte_index = &self.string[start..]
            .char_indices()
            .nth(char_boundary)
            .unwrap()
            .0;

        (
            DocString {
                string: self.string.clone(),
                range: (start + 0)..(start + byte_index),
            },
            DocString {
                string: self.string.clone(),
                range: (start + byte_index)..end,
            },
        )
    }

    pub unsafe fn seek_start_forward(&mut self, add: usize) {
        let (start, end) = (self.range.start, self.range.end);
        let add_bytes = self.string[start..]
            .char_indices()
            .map(|(index, _)| index)
            .chain(::std::iter::once(end))
            .nth(add)
            .expect("Moved beyond end of string");
        self.range = start + add_bytes..end;
    }

    pub unsafe fn seek_start_backward(&mut self, sub: usize) {
        let (start, end) = (self.range.start, self.range.end);
        let mut start_bytes = start;
        if sub > 0 {
            start_bytes = self.string[..start]
                .char_indices()
                .map(|(index, _)| index)
                .rev()
                .nth(sub - 1)
                .expect("Moved beyond start of string");
        }
        self.range = start_bytes..end;
    }

    pub unsafe fn try_byte_range(&self) -> Option<&Range<usize>> {
        Some(&self.range)
    }

    pub unsafe fn byte_range_mut(&mut self) -> &mut Range<usize> {
        &mut self.range
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
        serializer.serialize_str(self.as_str())
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

                Ok(DocString::from_string(ret))
            }

            fn visit_str<E>(self, value: &str) -> Result<DocString, E>
            where
                E: de::Error,
            {
                Ok(DocString::from_str(value))
            }
        }

        deserializer.deserialize_any(FieldVisitor)
    }
}

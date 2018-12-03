use serde::de::{
    self,
    SeqAccess,
    Visitor,
};
use serde::{
    ser::SerializeSeq,
    Deserialize,
    Deserializer,
    Serialize,
    Serializer,
};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::ops::Range;
use std::sync::Arc;
use enumset::*;

// Possible model for moving this out of core: provide an API for a resident "style service" or just
// bless a particular attribute in your codebase with #[oatie_style].

#[repr(u8)]
#[derive(Debug, Serialize, Deserialize, EnumSetType)]
pub enum Style {
    Normie,   // Sentinel (if this isn't present on a DocString, something went wrong somewhere)
    Selected, // Never used in server, added on client to show selected text
    Bold,
    Italic,
    Link,     // Needs attached link data
}

impl fmt::Display for Style {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Use the Debug implementation for Display.
        fmt::Debug::fmt(self, f)
    }
}

pub type StyleMap = HashMap<Style, Option<String>>;
pub type StyleSet = HashSet<Style>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpaqueStyleMap(EnumSet<Style>, Option<Arc<String>>);

impl OpaqueStyleMap {
    pub fn new() -> Self {
        OpaqueStyleMap(EnumSet::new(), None)
    }

    pub fn from(mut map: StyleMap) -> Self {
        let link = map.remove_entry(&Style::Link)
            .map(|(_k, link)| {
                Arc::new(link.unwrap())
            });
        let mut set = EnumSet::new();
        map.keys().for_each(|k| { set.insert(*k); });
        OpaqueStyleMap(set, link)
    }

    pub fn styles(&self) -> StyleSet {
        self.0.iter().collect()
    }

    pub fn contains(&self, style: Style) -> bool {
        self.0.contains(style)
    }

    pub fn to_map(&self) -> StyleMap {
        let mut hashmap: StyleMap = self.0.iter().map(|s| (s.to_owned(), None)).collect();
        if let Some(ref string) = self.1 {
            hashmap.insert(Style::Link, Some((*string).to_string()));
        }
        hashmap
    }

    pub fn iter(&self) -> impl Iterator<Item=(Style, Option<String>)> {
        // TODO OpaqueStyleMap::iter needs to support Link values (self.1)
        self.0.iter().map(|k| (k, None))
    }

    pub fn insert(&mut self, map: &StyleMap) {
        for (k, v) in map {
            self.0.insert(*k);
            if *k == Style::Link {
                self.1 = v.to_owned().map(|s| Arc::new(s));
            }
        }
    }

    pub fn remove(&mut self, set: &StyleSet) {
        for item in set {
            self.0.remove(*item);
            if item == &Style::Link {
                self.1 = None;
            }
        }
    }
}

/// Abstraction for String that has better performance by restricting its API.
/// It can also be styled using the Style enum.
#[derive(Clone, Debug)]
pub struct DocString {
    string: Arc<String>,
    range: Range<usize>,
    styles: OpaqueStyleMap,
}

impl DocString {
    pub fn from_string(input: String) -> DocString {
        let range = 0..input.len();
        DocString{ string: Arc::new(input), range, styles: OpaqueStyleMap::new() }
    }

    pub fn from_str(input: &str) -> DocString {
        let range = 0..input.len();
        DocString { string: Arc::new(input.to_owned()), range, styles: OpaqueStyleMap::new() }
    }

    pub fn from_string_styled(input: String, styles: StyleMap) -> DocString {
        let range = 0..input.len();
        DocString{ string: Arc::new(input), range, styles: OpaqueStyleMap::from(styles) }
    }

    pub fn from_str_styled(input: &str, styles: StyleMap) -> DocString {
        let range = 0..input.len();
        DocString { string: Arc::new(input.to_owned()), range, styles: OpaqueStyleMap::from(styles) }
    }

    // TODO audit use of this
    pub fn as_str(&self) -> &str {
        &self.string[self.range.clone()]
    }

    pub fn styles(&self) -> Option<OpaqueStyleMap> {
        Some(self.styles.clone())
    }

    pub fn remove_styles(&mut self, styles: &StyleSet) {
        self.styles.remove(styles);
    }

    pub fn extend_styles(&mut self, styles: &StyleMap) {
        self.styles.insert(styles);
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

        let byte_index = &self.string[start..].char_indices().nth(char_boundary).unwrap().0;

        (
            DocString {
                string: self.string.clone(),
                range: (start + 0)..(start + byte_index),
                styles: self.styles.clone(),
            },
            DocString {
                string: self.string.clone(),
                range: (start + byte_index)..end,
                styles: self.styles.clone(),
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
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // if let &Some(ref styles) = &self.styles {
            let mut s = serializer.serialize_seq(Some(2))?;
            s.serialize_element(self.as_str())?;
            s.serialize_element(&self.styles.to_map())?;
            s.end()
        // } else {
        //     serializer.serialize_str(self.as_str())
        // }
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

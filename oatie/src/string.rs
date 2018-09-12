use serde::{
    de::{
        self,
        SeqAccess,
        Visitor,
    },
    ser::SerializeSeq,
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::{
    collections::{
        HashMap,
        HashSet,
    },
    fmt,
    ops::Range,
    sync::{
        atomic::AtomicUsize,
        Arc,
    },
};
use core::{
    char,
};

#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Style {
    Normie,   // sentinel
    Selected, // never used except on the client
    Bold,
    Italic,
    Link,
}

impl fmt::Display for Style {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

pub type StyleMap = HashMap<Style, Option<String>>;
pub type StyleSet = HashSet<Style>;

/// Abstraction for String that allows a limited set of operations
/// with good optimization. (Or that's the idea.)
#[derive(Clone, Debug)]
pub struct DocString(Arc<String>, pub Option<Range<usize>>, Option<Arc<StyleMap>>);

impl DocString {
    pub fn from_string(input: String) -> DocString {
        DocString(Arc::new(input), None, None)
    }

    pub fn from_str(input: &str) -> DocString {
        DocString(Arc::new(input.to_owned()), None, None)
    }

    pub fn from_string_styled(input: String, styles: StyleMap) -> DocString {
        DocString(Arc::new(input), None, Some(Arc::new(styles)))
    }

    pub fn from_str_styled(input: &str, styles: StyleMap) -> DocString {
        DocString(Arc::new(input.to_owned()), None, Some(Arc::new(styles)))
    }

    // TODO audit use of this
    pub fn as_str(&self) -> &str {
        if let Some(ref range) = self.1 {
            &self.0[range.clone()]
        } else {
            &self.0
        }
    }

    pub fn styles(&self) -> Option<Arc<StyleMap>> {
        self.2.clone()
    }

    pub fn remove_styles(&mut self, styles: &StyleSet) {
        if let &mut Some(ref mut self_styles) = &mut self.2 {
            let mut new_styles: StyleMap = (**self_styles).clone();
            *self_styles = Arc::new(new_styles
                .drain()
                .filter(|(ref x, _)| !styles.contains(x))
                .collect());
        } else {
            // no-op
        }
    }

    pub fn extend_styles(&mut self, styles: &StyleMap) {
        if let &mut Some(ref self_styles) = &mut self.2 {
            let mut new_styles: StyleMap = (**self_styles).clone();
            new_styles.extend(styles.iter().map(|(a, b)| (a.to_owned(), b.to_owned())));
            self.2 = Some(Arc::new(new_styles));
        } else {
            self.2 = Some(Arc::new(styles.to_owned()));
        }
    }

    // Add text (with the same styling) to the end of this string.
    pub fn push_str(&mut self, input: &str) {
        let mut value = self.to_string();
        value.push_str(input);
        self.0 = Arc::new(value);
        self.1 = None;
    }

    

    // TODO consume self?
    pub fn split_at(&self, char_boundary: usize) -> (DocString, DocString) {
        let mut start = 0;
        let mut end = self.0.len();
        if let Some(ref range) = self.1 {
            start = range.start;
            end = range.end;
        }

        let byte_index = &self.0[start..].char_indices().nth(char_boundary).unwrap().0;

        (
            DocString(
                self.0.clone(),
                Some((start + 0)..(start + byte_index)),
                self.2.clone(),
            ),
            DocString(
                self.0.clone(),
                Some((start + byte_index)..end),
                self.2.clone(),
            ),
        )
    }

    pub unsafe fn seek_start_forward(&mut self, add: usize) {
        let (start, end) = if let Some(ref range) = self.1 {
            (range.start, range.end)
        } else {
            (0, self.0.len())
        };
        let add_bytes = self.0[start..]
            .char_indices()
            .map(|(index, _)| index)
            .chain(::std::iter::once(end))
            .nth(add)
            .expect("Moved beyond end of string");
        self.1 = Some(start + add_bytes..end);
    }

    pub unsafe fn seek_start_backward(&mut self, sub: usize) {
        let (start, end) = if let Some(ref range) = self.1 {
            (range.start, range.end)
        } else {
            (0, self.0.len())
        };
        let mut start_bytes = start;
        if sub > 0 {
            start_bytes = self.0[..start]
                .char_indices()
                .map(|(index, _)| index)
                .rev()
                .nth(sub - 1)
                .expect("Moved beyond start of string");
        }
        self.1 = Some(start_bytes..end);
    }

    pub unsafe fn try_byte_range(&self) -> Option<&Range<usize>> {
        self.1.as_ref()
    }

    pub unsafe fn byte_range_mut(&mut self) -> &mut Range<usize> {
        if self.1.is_none() {
            self.1 = Some(0..(self.0.len()));
        }
        self.1.as_mut().unwrap()
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
            s.serialize_element(Arc::as_ref(value))?;
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


/// Indexes into a DocString, tracking two owned DocStrings left() and right() which
/// can be retrieved by reference. Because indexing into the string is 
/// performed on DocString internals, this makes scanning a Unicode string
/// much faster than split_at().
#[derive(Clone, Debug, PartialEq)]
pub struct CharCursor {
    right_string: DocString,
    left_string: DocString,
    index: usize,
    //TODO add str_len: usize, and do more checking that index doesn't go out of range
}

impl CharCursor {
    pub fn left<'a>(&'a self) -> Option<&'a DocString> {
        if unsafe {
            self.left_string.try_byte_range().unwrap().len() == 0
        } {
            None
        } else {
            Some(&self.left_string)
        }
    }

    pub fn right<'a>(&'a self) -> Option<&'a DocString> {
        if unsafe {
            self.right_string.try_byte_range().unwrap().len() == 0
        } {
            None
        } else {
            Some(&self.right_string)
        }
    }

    // TODO rename this to index(), value_add to seek_add, value_sub to seek_sub
    pub fn value(&self) -> usize {
        self.index
    }

    pub fn index_from_end(&self) -> usize {
        unsafe {
            // TODO this is incorrect (unwrap_or should be str len),
            // try_byte_range really needs to be replaced
            // with something that guarantees a range
            self.right_string.try_byte_range().map(|x| x.len()).unwrap_or(0)
        }
    }

    pub fn value_add(&mut self, add: usize) {
        self.index += add;
        unsafe {
            self.right_string.seek_start_forward(add);
            self.left_string.byte_range_mut().end = self.right_string.byte_range_mut().start;
        }
    }

    pub fn value_sub(&mut self, sub: usize) {
        self.index -= sub;
        unsafe {
            self.right_string.seek_start_backward(sub);
            self.left_string.byte_range_mut().end = self.right_string.byte_range_mut().start;
        }
    }

    pub fn from_docstring(text: &DocString) -> CharCursor {
        let mut left_string = text.clone();
        let mut right_string = text.clone();

        // Collapse the left string's range to the start of the string and its length to 0.
        // (A zero-length range is usually invalid, so we need to be careful
        // not to call functions that depend on that being true. Hence the unsafe.)
        unsafe {
            left_string.byte_range_mut().end = right_string.byte_range_mut().start;
        }

        CharCursor {
            left_string,
            right_string,
            index: 0,
        }
    }

    pub fn from_docstring_end(text: &DocString) -> CharCursor {
        let left_string = text.clone();
        let mut right_string = text.clone();

        // Collapse the left string's range to the start of the string and its length to 0.
        // (A zero-length range is usually invalid, so we need to be careful
        // not to call functions that depend on that being true. Hence the unsafe.)
        unsafe {
            right_string.byte_range_mut().start = right_string.byte_range_mut().end;
        }

        CharCursor {
            left_string,
            right_string,
            index: text.char_len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let mut ds = CharCursor::from_docstring(&DocString::from_str("Welcome!"));
        ds.value_add(6);
        assert_eq!(ds.right().unwrap().as_str(), "e!");
    }

    #[test]
    #[should_panic]
    fn seek_too_far() {
        let mut ds = CharCursor::from_docstring(&DocString::from_str("Welcome!"));
        ds.value_add(11);
    }

    #[test]
    #[should_panic]
    fn seek_negative() {
        let mut ds = CharCursor::from_docstring(&DocString::from_str("Welcome!"));
        ds.value_add(4);
        ds.value_sub(10);
    }

    #[test]
    fn option_ends() {
        let mut ds = CharCursor::from_docstring(&DocString::from_str("Welcome!"));
        assert_eq!(ds.left(), None);
        assert_eq!(ds.right().is_some(), true);
        ds.value_add("Welcome!".len());
        assert_eq!(ds.left().is_some(), true);
        assert_eq!(ds.right(), None);
        ds.value_sub(1);
        assert_eq!(ds.left().is_some(), true);
        assert_eq!(ds.right().is_some(), true);
    }
}
//! TODO Move this to src/core ?

use super::DocString;
use crate::doc::*;

/// Indexes into a DocString, tracking two owned DocStrings left() and right() which
/// can be retrieved by reference. Because indexing into the string is
/// performed on DocString internals, this makes scanning a Unicode string
/// much faster than split_at().
#[derive(Clone, Debug, PartialEq)]
pub struct CharCursor<S: Schema> {
    left_string: DocElement<S>,
    right_string: DocElement<S>,
    index: usize,   // in chars
    str_len: usize, // in chars
}

impl<S: Schema> CharCursor<S> {
    #[inline(always)]
    pub fn from_docstring(text: &DocString, styles: S::CharsProperties) -> CharCursor<S> {
        let mut left_string = text.clone();
        let mut right_string = text.clone();

        // Collapse the left string's range to the start of the string and its length to 0.
        // (A zero-length range is usually invalid, so we need to be careful
        // not to call functions that depend on that being true. Hence the unsafe.)
        unsafe {
            left_string.byte_range_mut().end = right_string.byte_range_mut().start;
        }

        CharCursor {
            left_string: DocElement::DocChars(styles.clone(), left_string),
            right_string: DocElement::DocChars(styles, right_string),
            index: 0,
            str_len: text.char_len(),
        }
    }

    #[inline(always)]
    pub fn from_docstring_end(text: &DocString, styles: S::CharsProperties) -> CharCursor<S> {
        let left_string = text.clone();
        let mut right_string = text.clone();

        // Collapse the left string's range to the start of the string and its length to 0.
        // (A zero-length range is usually invalid, so we need to be careful
        // not to call functions that depend on that being true. Hence the unsafe.)
        unsafe {
            right_string.byte_range_mut().start = right_string.byte_range_mut().end;
        }

        CharCursor {
            left_string: DocElement::DocChars(styles.clone(), left_string),
            right_string: DocElement::DocChars(styles, right_string),
            index: text.char_len(),
            str_len: text.char_len(),
        }
    }

    pub fn update_from_docstring(&mut self, text: &DocString, styles: S::CharsProperties) {
        self.left_string = DocElement::DocChars(styles.clone(), text.clone());
        self.right_string = DocElement::DocChars(styles, text.clone());
        self.index = text.char_len();
    }

    pub fn style<'a>(&'a self) -> &'a S::CharsProperties {
        if let DocElement::DocChars(ref styles, _) = &self.left_string {
            styles
        } else {
            unreachable!();
        }
    }

    pub fn left<'a>(&'a self) -> Option<&'a DocString> {
        if let DocElement::DocChars(_, ref left_text) = &self.left_string {
            if unsafe { left_text.try_byte_range().unwrap().len() == 0 } {
                None
            } else {
                Some(left_text)
            }
        } else {
            unreachable!();
        }
    }

    pub fn right<'a>(&'a self) -> Option<&'a DocString> {
        if let DocElement::DocChars(_, ref right_text) = &self.right_string {
            if unsafe { right_text.try_byte_range().unwrap().len() == 0 } {
                None
            } else {
                Some(right_text)
            }
        } else {
            unreachable!();
        }
    }

    pub fn left_element<'a>(&'a self) -> Option<&'a DocElement<S>> {
        if let DocElement::DocChars(_, ref left_text) = &self.left_string {
            if unsafe { left_text.try_byte_range().unwrap().len() == 0 } {
                None
            } else {
                Some(&self.left_string)
            }
        } else {
            unreachable!();
        }
    }

    pub fn right_element<'a>(&'a self) -> Option<&'a DocElement<S>> {
        if let DocElement::DocChars(_, ref right_text) = &self.right_string {
            if unsafe { right_text.try_byte_range().unwrap().len() == 0 } {
                None
            } else {
                Some(&self.right_string)
            }
        } else {
            unreachable!();
        }
    }

    fn left_text_mut<'a>(&'a mut self) -> &'a mut DocString {
        if let DocElement::DocChars(_, ref mut left_text) = self.left_string {
            left_text
        } else {
            unreachable!();
        }
    }

    fn right_text_mut<'a>(&'a mut self) -> &'a mut DocString {
        if let DocElement::DocChars(_, ref mut right_text) = self.right_string {
            right_text
        } else {
            unreachable!();
        }
    }

    // TODO rename this to index(), value_add to seek_add, value_sub to seek_sub
    pub fn value(&self) -> usize {
        self.index
    }

    pub fn index_from_end(&self) -> usize {
        self.str_len - self.index
    }

    pub fn value_add(&mut self, add: usize) {
        self.index += add;
        unsafe {
            self.right_text_mut().seek_start_forward(add);
            self.left_text_mut().byte_range_mut().end =
                self.right_text_mut().byte_range_mut().start;
        }
    }

    pub fn value_sub(&mut self, sub: usize) {
        self.index -= sub;
        unsafe {
            self.right_text_mut().seek_start_backward(sub);
            self.left_text_mut().byte_range_mut().end =
                self.right_text_mut().byte_range_mut().start;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rtf::*;

    #[test]
    fn basic() {
        let mut ds = CharCursor::<RtfSchema>::from_docstring(&DocString::from_str("Welcome!"), StyleSet::new());
        ds.value_add(6);
        assert_eq!(ds.right().unwrap().as_str(), "e!");
    }

    #[test]
    #[should_panic]
    fn seek_too_far() {
        let mut ds = CharCursor::<RtfSchema>::from_docstring(&DocString::from_str("Welcome!"), StyleSet::new());
        ds.value_add(11);
    }

    #[test]
    #[should_panic]
    fn seek_negative() {
        let mut ds = CharCursor::<RtfSchema>::from_docstring(&DocString::from_str("Welcome!"), StyleSet::new());
        ds.value_add(4);
        ds.value_sub(10);
    }

    #[test]
    fn option_ends() {
        let mut ds = CharCursor::<RtfSchema>::from_docstring(&DocString::from_str("Welcome!"), StyleSet::new());
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

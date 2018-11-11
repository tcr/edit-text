use super::doc::*;

pub(crate) fn can_element_join(left: &DocElement, right: &DocElement) -> bool {
    match (left, right) {
        (&DocChars(ref prefix), &DocChars(ref suffix)) => {
            if prefix.styles() == suffix.styles() {
                return true;
            }
        }
        _ => {},
    }
    false
}

pub(crate) fn try_element_join(left: &mut DocElement, right: &DocElement) -> bool {
    match (left, right) {
        (&mut DocChars(ref mut prefix), &DocChars(ref suffix)) => {
            if prefix.styles() == suffix.styles() {
                prefix.push_str(suffix.as_str());
                return true
            }
        }
        _ => {},
    }
    false
}

pub trait DocPlaceable {
    fn skip_len(&self) -> usize;
    fn place_all(&mut self, all: &[DocElement]);
    fn place(&mut self, value: &DocElement);
}

impl DocPlaceable for DocSpan {
    fn place(&mut self, elem: &DocElement) {
        match *elem {
            DocChars(ref text) => {
                assert!(text.char_len() > 0);

                // If the most recent element is text, we may want to just
                // append our text to it to cut down on new elements.
                if let Some(element) = self.last_mut() {
                    if try_element_join(element, elem) {
                        return;
                    }
                    // // Check if they're equal and we can push it directly.
                    // if prefix.styles() == text.styles() {
                    //     prefix.push_str(text.as_str());
                    //     return;
                    // }
                }

                // Otherwise, push the new entry
                self.push(DocChars(text.to_owned()));
            }
            DocGroup(..) => {
                self.push(elem.clone());
            }
        }
    }

    fn place_all(&mut self, all: &[DocElement]) {
        for i in all {
            self.place(i);
        }
    }

    fn skip_len(&self) -> usize {
        let mut ret = 0;
        for item in self {
            ret += match *item {
                DocChars(ref value) => value.char_len(),
                DocGroup(..) => 1,
            };
        }
        ret
    }
}

pub trait DelPlaceable {
    fn place_all(&mut self, all: &[DelElement]);
    fn place(&mut self, value: &DelElement);
    fn skip_pre_len(&self) -> usize;
    fn skip_post_len(&self) -> usize;

    /// Optimization for depth-first code to recursively return skips up
    /// the walker.
    fn is_continuous_skip(&self) -> bool;
}

impl DelPlaceable for DelSpan {
    fn place_all(&mut self, all: &[DelElement]) {
        for i in all {
            self.place(i);
        }
    }

    fn place(&mut self, elem: &DelElement) {
        match *elem {
            DelChars(count) => {
                assert!(count > 0);
                if let Some(&mut DelChars(ref mut value)) = self.last_mut() {
                    *value += count;
                } else {
                    self.push(DelChars(count));
                }
            }
            DelStyles(count, ref styles) => {
                assert!(count > 0);
                if let Some(&mut DelStyles(ref mut prefix_count, ref prefix_styles)) =
                    self.last_mut()
                {
                    if prefix_styles == styles {
                        *prefix_count += count;
                        return;
                    }
                }

                self.push(DelStyles(count, styles.to_owned()));
            }
            DelSkip(count) => {
                assert!(count > 0);
                if let Some(&mut DelSkip(ref mut value)) = self.last_mut() {
                    *value += count;
                } else {
                    self.push(DelSkip(count));
                }
            }
            DelGroup(..) | DelWithGroup(..) => {
                self.push(elem.clone());
            } // DelGroupAll | DelObject => {
              //     unimplemented!();
              // }
              // DelMany(count) => {
              //     unimplemented!();
              // }
        }
    }

    fn skip_pre_len(&self) -> usize {
        let mut ret = 0;
        for item in self {
            ret += match *item {
                DelSkip(len) | DelChars(len) | DelStyles(len, _) => len,
                DelGroup(..) | DelWithGroup(..) => 1,
                // DelMany(len) => len,
                // DelObject | DelGroupAll  => 1,
            };
        }
        ret
    }

    fn skip_post_len(&self) -> usize {
        let mut ret = 0;
        for item in self {
            ret += match *item {
                DelSkip(len) | DelStyles(len, _) => len,
                DelChars(..) => 0,
                DelWithGroup(..) => 1,
                DelGroup(ref span) => span.skip_post_len(),
                // DelObject | DelMany(..) | DelGroupAll => 0,
            };
        }
        ret
    }

    fn is_continuous_skip(&self) -> bool {
        if self.len() > 1 {
            // Will never be a continuous skip
            false
        } else if self.is_empty() {
            // is []
            true
        } else if let DelSkip(_) = self[0] {
            // is [DelSkip(n)]
            true
        } else {
            // is [DelSomething(n)]
            false
        }
    }
}

pub trait AddPlaceable {
    fn place_all(&mut self, all: &[AddElement]);
    fn place(&mut self, value: &AddElement);
    fn skip_pre_len(&self) -> usize;
    fn skip_post_len(&self) -> usize;

    /// Optimization for depth-first code to recursively return skips up
    /// the walker.
    fn is_continuous_skip(&self) -> bool;
}

impl AddPlaceable for AddSpan {
    fn place_all(&mut self, all: &[AddElement]) {
        for i in all {
            self.place(i);
        }
    }

    fn place(&mut self, elem: &AddElement) {
        match *elem {
            AddChars(ref text) => {
                assert!(text.char_len() > 0);

                // If the most recent element is text, we may want to just
                // append our text to it to cut down on new elements.
                if let Some(&mut AddChars(ref mut prefix)) = self.last_mut() {
                    // Check if they're equal and we can push it directly.
                    if prefix.styles() == text.styles() {
                        prefix.push_str(text.as_str());
                        return;
                    }
                }

                // Otherwise, push the new entry
                self.push(AddChars(text.to_owned()));
            }
            AddStyles(count, ref styles) => {
                assert!(count > 0);
                if let Some(&mut AddStyles(ref mut prefix_count, ref prefix_styles)) =
                    self.last_mut()
                {
                    if prefix_styles == styles {
                        *prefix_count += count;
                        return;
                    }
                }

                self.push(AddStyles(count, styles.to_owned()));
            }
            AddSkip(count) => {
                assert!(count > 0);
                if let Some(&mut AddSkip(ref mut value)) = self.last_mut() {
                    *value += count;
                } else {
                    self.push(AddSkip(count));
                }
            }
            AddGroup(..) | AddWithGroup(..) => {
                self.push(elem.clone());
            }
        }
    }

    fn skip_pre_len(&self) -> usize {
        let mut ret = 0;
        for item in self {
            ret += match *item {
                AddSkip(len) | AddStyles(len, _) => len,
                AddChars(ref chars) => 0,
                AddGroup(_, ref span) => span.skip_pre_len(),
                AddWithGroup(..) => 1,
            };
        }
        ret
    }

    fn skip_post_len(&self) -> usize {
        let mut ret = 0;
        for item in self {
            ret += match *item {
                AddSkip(len) | AddStyles(len, _) => len,
                AddChars(ref chars) => chars.char_len(),
                AddGroup(..) | AddWithGroup(..) => 1,
            };
        }
        ret
    }

    fn is_continuous_skip(&self) -> bool {
        if self.len() > 1 {
            // Will never be a continuous skip
            false
        } else if self.is_empty() {
            // is []
            true
        } else if let AddSkip(_) = self[0] {
            // is [DelSkip(n)]
            true
        } else {
            // is [DelSomething(n)]
            false
        }
    }
}

pub trait CurPlaceable {
    fn place_all(&mut self, all: &[CurElement]);
    fn place(&mut self, value: &CurElement);
}

impl CurPlaceable for CurSpan {
    fn place_all(&mut self, all: &[CurElement]) {
        for i in all {
            self.place(i);
        }
    }

    fn place(&mut self, elem: &CurElement) {
        match *elem {
            CurSkip(count) => {
                assert!(count > 0);
                if let Some(&mut CurSkip(ref mut value)) = self.last_mut() {
                    *value += count;
                } else {
                    self.push(CurSkip(count));
                }
            }
            CurGroup | CurChar | CurWithGroup(..) => {
                self.push(elem.clone());
            }
        }
    }
}

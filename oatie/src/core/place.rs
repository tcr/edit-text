use super::doc::*;

pub(crate) fn can_element_join<S: Schema>(left: &DocElement<S>, right: &DocElement<S>) -> bool {
    match (left, right) {
        (&DocChars(ref _prefix, ref prefix_styles), &DocChars(ref _suffix, ref suffix_styles)) => {
            if prefix_styles == suffix_styles {
                return true;
            }
        }
        _ => {}
    }
    false
}

pub(crate) fn try_element_join<S: Schema>(left: &mut DocElement<S>, right: &DocElement<S>) -> bool {
    match (left, right) {
        (&mut DocChars(ref prefix_styles, ref mut prefix), &DocChars(ref suffix_styles, ref suffix)) => {
            if prefix_styles == suffix_styles {
                prefix.push_str(suffix.as_str());
                return true;
            }
        }
        _ => {}
    }
    false
}

pub trait DocPlaceable<S: Schema> {
    fn skip_len(&self) -> usize;
    fn place_all(&mut self, all: &[DocElement<S>]);
    fn place(&mut self, value: &DocElement<S>);
}

impl<S: Schema> DocPlaceable<S> for DocSpan<S> {
    fn place(&mut self, elem: &DocElement<S>) {
        match *elem {
            DocChars(ref _styles, ref text) => {
                assert!(text.char_len() > 0);

                // If the most recent element is text, we may want to just
                // append our text to it to cut down on new elements.
                if let Some(element) = self.last_mut() {
                    if try_element_join(element, elem) {
                        return;
                    }
                }

                // Otherwise, push the new entry
                self.push(elem.clone());
            }
            DocGroup(..) => {
                self.push(elem.clone());
            }
        }
    }

    fn place_all(&mut self, all: &[DocElement<S>]) {
        for i in all {
            self.place(i);
        }
    }

    fn skip_len(&self) -> usize {
        let mut ret = 0;
        for item in self {
            ret += match *item {
                DocChars(_, ref value) => value.char_len(),
                DocGroup(..) => 1,
            };
        }
        ret
    }
}

pub trait DelPlaceable<S: Schema> {
    fn place_all(&mut self, all: &[DelElement<S>]);
    fn place(&mut self, value: &DelElement<S>);
    fn skip_pre_len(&self) -> usize;
    fn skip_post_len(&self) -> usize;

    /// Optimization for depth-first code to recursively return skips up
    /// the walker.
    fn is_continuous_skip(&self) -> bool;
}

impl<S: Schema> DelPlaceable<S> for DelSpan<S> {
    fn place_all(&mut self, all: &[DelElement<S>]) {
        for i in all {
            self.place(i);
        }
    }

    fn place(&mut self, elem: &DelElement<S>) {
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

pub trait AddPlaceable<S: Schema> {
    fn place_all(&mut self, all: &[AddElement<S>]);
    fn place(&mut self, value: &AddElement<S>);
    fn skip_pre_len(&self) -> usize;
    fn skip_post_len(&self) -> usize;

    /// Optimization for depth-first code to recursively return skips up
    /// the walker.
    fn is_continuous_skip(&self) -> bool;
}

impl<S: Schema> AddPlaceable<S> for AddSpan<S> {
    fn place_all(&mut self, all: &[AddElement<S>]) {
        for i in all {
            self.place(i);
        }
    }

    fn place(&mut self, elem: &AddElement<S>) {
        match *elem {
            AddChars(ref styles, ref text) => {
                assert!(text.char_len() > 0);

                // If the most recent element is text, we may want to just
                // append our text to it to cut down on new elements.
                if let Some(&mut AddChars(ref prefix_styles, ref mut prefix)) = self.last_mut() {
                    // Check if they're equal and we can push it directly.
                    if styles == prefix_styles {
                        prefix.push_str(text.as_str());
                        return;
                    }
                }

                // Otherwise, push the new entry
                self.push(AddChars(styles.clone(), text.to_owned()));
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
                AddChars(ref _chars, _) => 0,
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
                AddChars(_, ref chars) => chars.char_len(),
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

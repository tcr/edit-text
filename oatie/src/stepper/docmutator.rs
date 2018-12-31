use super::*;
use serde_json::json;
use wasm_bindgen::prelude::*;

impl Program {
    pub fn new() -> Program {
        Program(vec![])
    }

    // Collapse trivial operations together.
    pub fn place(&mut self, mut code: Bytecode) {
        use self::Bytecode::*;
        // console_log!("       ❎ {:?}", code);
        match (self.0.last_mut(), &mut code) {
            (Some(&mut AdvanceElements(ref mut last_n)), AdvanceElements(n)) => {
                *last_n += *n;
            }
            (Some(&mut DeleteElements(ref mut last_n)), DeleteElements(n)) => {
                *last_n += *n;
            }
            // (Some(&mut InsertString(ref mut last_str)), InsertString(ref mut new_n)) => {
            //     *last_str = format!("{}{}", last_str.as_str(), new_n.as_str());
            // }
            _ => self.0.push(code.clone()),
        }
    }

    // fn place_all(&mut self, mut codes: Vec<Bytecode>) {
    //     if codes.len() > 0 {
    //         self.place(&codes.remove(0));
    //         self.0.extend(codes.into_iter());
    //     }

    // }
}

#[allow(non_snake_case)]
pub trait DocMutator<S: Schema> {
    fn flush_chars(&mut self) -> bool {
        unimplemented!();
    }

    fn flush(&mut self) {
        unimplemented!();
    }

    fn Enter(&mut self) {
        unimplemented!();
    }

    fn Exit(&mut self) {
        unimplemented!();
    }

    /// TODO rename to advance
    fn AdvanceElements(&mut self, _count: usize) {
        unimplemented!();
    }

    fn delete(&mut self, _count: usize) {
        unimplemented!();
    }

    fn DeleteElements(&mut self, _count: usize) {
        unimplemented!();
    }

    fn InsertDocString(&mut self, _docstring: DocString, _styles: S::CharsProperties) {
        unimplemented!();
    }

    fn UnwrapSelf(&mut self) {
        unimplemented!();
    }

    fn WrapPrevious(&mut self, _count: usize, _attrs: S::GroupProperties) {
        unimplemented!();
    }

    fn skip(&mut self, _count: usize) {
        unimplemented!();
    }
}

pub struct NullDocMutator {}

impl<S: Schema> DocMutator<S> for NullDocMutator {
    fn Enter(&mut self) {
        // no-op
    }

    fn Exit(&mut self) {
        // no-op
    }

    /// TODO rename to advance
    fn AdvanceElements(&mut self, _count: usize) {
        // no-op
    }

    fn delete(&mut self, _count: usize) {
        // no-op
    }

    fn DeleteElements(&mut self, _count: usize) {
        // no-op
    }

    fn InsertDocString(&mut self, _docstring: DocString, _style: S::CharsProperties) {
        // no-op
    }

    fn UnwrapSelf(&mut self) {
        // no-op
    }

    fn WrapPrevious(&mut self, _count: usize, _attrs: S::GroupProperties) {
        // no-op
    }

    fn skip(&mut self, _count: usize) {
        // no-op
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, TypescriptDefinition)]
#[serde(tag = "tag", content = "fields")]
pub enum Bytecode {
    Enter,
    Exit,
    AdvanceElements(usize),
    DeleteElements(usize),
    InsertDocString(DocString, serde_json::Value),
    WrapPrevious(usize, serde_json::Value),
    UnwrapSelf,
    JoinTextLeft,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Program(pub Vec<Bytecode>);

#[derive(Clone, Debug)]
pub struct RecordingDocMutator<'a, S: Schema> {
    bc: Program,
    stepper: DocStepper<'a, S>,
    writer: DocWriter<S>,
}

impl<'a, S: Schema> RecordingDocMutator<'a, S> {
    pub fn stepper(&'a self) -> &'a DocStepper<'_, S> {
        &self.stepper
    }

    pub fn result(mut self) -> Result<(DocSpan<S>, Program), Error> {
        self.flush();

        let RecordingDocMutator { writer, bc, .. } = self;
        (writer.result().map(|doc| (doc, bc)))
    }

    pub fn new(stepper: DocStepper<'_, S>) -> RecordingDocMutator<'_, S> {
        RecordingDocMutator {
            bc: Program(vec![]),
            stepper,
            writer: DocWriter::new(),
        }
    }

    fn flush_chars(&mut self) -> bool {
        // Insert right part of a partial string and advance.
        if let Some(index) = self.stepper.char_index() {
            if index > 0 {
                // Partial string.
                let partial = self.stepper.char_cursor_expect().right().expect("hey now");
                // console_log!("🏟 {:?}", partial);
                if let Some(&DocText(ref styles, ref _text)) = self.stepper.head_raw() {
                    self.bc
                        .place(Bytecode::InsertDocString(partial.clone(), json!(styles)));
                    self.writer.place(&DocText(styles.clone(), partial.clone()));
                } else {
                    unreachable!();
                }
                self.stepper.next();
                return true;
            } else if let (Some(ref previous), Some(ref head)) =
                (self.writer.past.last(), self.stepper.head())
            {
                // Full string, but last written state was a doc string.
                if can_element_join(previous, head) {
                    self.bc.place(Bytecode::JoinTextLeft);
                    self.writer.place(head);
                    self.stepper.next();
                    return true;
                }
            }
        }
        false
    }

    fn flush(&mut self) {
        self.flush_chars();

        // console_log!("🎚 self.stepper.head => {:?}", self.stepper.head());
        while let Some(item) = self.stepper.head() {
            // console_log!("🎚 self.stepper item => {:?}", self.stepper.head());
            self.writer.place(&item);
            self.stepper.next();
        }
    }
}

#[allow(non_snake_case)]
impl<'a, S: Schema> DocMutator<S> for RecordingDocMutator<'a, S> {
    fn Enter(&mut self) {
        self.bc.place(Bytecode::Enter);

        self.stepper.enter();

        self.writer.begin();
    }

    fn Exit(&mut self) {
        self.flush();

        self.bc.place(Bytecode::Exit);

        let attrs = self.stepper.exit_with_attrs();
        self.writer.close(attrs);
    }

    /// TODO rename to advance
    fn AdvanceElements(&mut self, mut count: usize) {
        // Insert left part of partial string and advance.
        if self.flush_chars() {
            count -= 1;
        }

        for _ in 0..count {
            self.bc.place(Bytecode::AdvanceElements(1));

            self.writer.place(match self.stepper.head_raw() {
                Some(head) => head,
                _ => panic!("no head element"),
            });

            self.stepper.next();
        }
    }

    fn delete(&mut self, count: usize) {
        let mut bytecode_count = count;
        if let Some(index) = self.stepper.char_index() {
            if index > 0 {
                // We can just move the skipper forward.
                bytecode_count -= 1;
            }
        }
        if bytecode_count > 0 {
            self.bc.place(Bytecode::DeleteElements(bytecode_count));
        }

        for _ in 0..count {
            // No-op writer

            self.stepper.next();
        }
    }

    fn DeleteElements(&mut self, count: usize) {
        self.bc.place(Bytecode::DeleteElements(count));

        for _ in 0..count {
            // No-op writer

            self.stepper.next();
        }
    }

    fn InsertDocString(&mut self, docstring: DocString, styles: S::CharsProperties) {
        self.bc
            .place(Bytecode::InsertDocString(docstring.clone(), json!(styles)));

        // No-op stepper

        self.writer.place(&DocText(styles, docstring));
    }

    fn UnwrapSelf(&mut self) {
        self.flush();

        self.bc.place(Bytecode::UnwrapSelf);

        self.stepper.exit();

        self.writer.unwrap_self();

        // TODO join the previous, written out text node with any now adjacent ones?
        // TODO apply above comment to the .delete() method ALSO
    }

    fn WrapPrevious(&mut self, count: usize, attrs: S::GroupProperties) {
        // console_log!("(A) {:?}", self.stepper.char_index());
        self.bc.place(Bytecode::WrapPrevious(count, json!(attrs)));

        // No-op stepper

        self.writer.wrap_previous(count, attrs);
    }

    fn skip(&mut self, count: usize) {
        let last_index = self.stepper.head_index();

        self.stepper.skip(count);

        let new_index = self.stepper.head_index();
        self.writer
            .place_all(&self.stepper.current().1[last_index..new_index].to_owned());
        if new_index - last_index > 0 {
            self.AdvanceElements(new_index - last_index);
        } else {
            // We're on the same element, possibly a text node.

            // console_log!(" -----> post skip {:?} is {:?}", count, self.stepper.cursor.suffix().char_cursor.clone());
            // TODO don't address char_cursor directly?
            if let Some(ref cursor) = &self.stepper.char_cursor.clone() {
                // Some(..) means left is already of len > 0
                if let Some(left) = cursor.left() {
                    if left.char_len() == count {
                        self.bc.place(Bytecode::DeleteElements(1)); // It's over, delete time
                        if let Some(&DocText(ref styles, _)) = self.stepper.head_raw() {
                            self.InsertDocString(left.clone(), styles.to_owned()); // Insert the left part of string
                        } else {
                            unreachable!();
                        }
                        // The right part of the string is added WHEN
                        return;
                    } else {
                        // Partial advancement
                        let mut text = left.clone();
                        // console_log!("\n\n\nPARTIAL ADVANEMENET {:?}\n\n\n", count);
                        unsafe {
                            text.seek_start_forward(count);
                        }
                        // console_log!("\n\n\nPARTIAL ADVANEMENET {:?}\n\n\n", text);
                        if let Some(&DocText(ref styles, _)) = self.stepper.head_raw() {
                            self.InsertDocString(text, styles.clone());
                        } else {
                            unreachable!();
                        }
                    }
                }
            }
        }
    }
}

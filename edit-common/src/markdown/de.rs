use failure::Error;
use oatie::doc::*;
use oatie::writer::DocWriter;
use pulldown_cmark::{
    Event::{
        self,
        End,
        FootnoteReference,
        HardBreak,
        Html,
        InlineHtml,
        SoftBreak,
        Start,
        Text,
    },
    Parser,
    Tag,
};

struct Ctx<'b, I> {
    iter: I,
    body: &'b mut DocWriter,
    styles: StyleMap,
    bare_text: bool,
}

impl<'a, 'b, I: Iterator<Item = Event<'a>>> Ctx<'b, I> {
    pub fn run(&mut self) {
        while let Some(event) = self.iter.next() {
            match event {
                Start(tag) => {
                    self.start_tag(tag);
                }
                End(tag) => {
                    self.end_tag(tag);
                }
                Text(text) => {
                    // TODO wrapping bare txt in a paragraph makes the result
                    // validate, but 1) the wrapping element should be a div,
                    // since it lacks any margin and 2) it should be contiguous
                    // with all other elements that follow it, so text<b>with</b>bold
                    // doesn't have three block elements generated, 1 for each span.
                    if self.bare_text {
                        self.body.begin();
                    }
                    self.body.place(&DocChars(DocString::from_str_styled(
                        text.as_ref(),
                        self.styles.clone(),
                    )));
                    if self.bare_text {
                        self.body.close(hashmap! { "tag".into() => "p".into() });
                    }
                }
                SoftBreak => {
                    // TODO this should actually use some heuristics to know
                    // if we should soft-space like HTML does. whitespace is
                    // significant in the document model so we can't always
                    // just add a space
                    if self.bare_text {
                        self.body.begin();
                    }
                    self.body.place(&DocChars(DocString::from_str_styled(
                        " ",
                        self.styles.clone(),
                    )));
                    if self.bare_text {
                        self.body.close(hashmap! { "tag".into() => "p".into() });
                    }
                }
                HardBreak => {
                    self.body.place(&DocChars(DocString::from_str_styled(
                        "\n",
                        self.styles.clone(),
                    )));
                }
                Html(html) => {
                    self.body.begin();
                    self.body.place(&DocChars(DocString::from_str_styled(
                        &html,
                        hashmap!{ Style::Normie => None },
                    )));
                    self.body.close(hashmap! { "tag".into() => "html".into() });
                }

                InlineHtml(..) | FootnoteReference(..) => {}
            }
        }
    }

    fn start_tag(&mut self, tag: Tag<'a>) {
        match tag {
            // Blocks
            Tag::Paragraph => {
                self.body.begin();
                self.bare_text = false;
            }
            Tag::Header(_level) => {
                self.body.begin();
                self.bare_text = false;
            }
            Tag::CodeBlock(_info) => {
                self.body.begin();
                self.bare_text = false;
            }

            // List items
            Tag::Item => {
                self.body.begin();
                self.bare_text = true;
            }

            // Block objects
            Tag::Rule => {
                self.body.begin();
            }

            // Spans
            Tag::Link(dest, _title) => {
                self.styles.insert(Style::Link, Some(dest.to_string()));
            }
            Tag::Strong => {
                self.styles.insert(Style::Bold, None);
            }
            Tag::Emphasis => {
                self.styles.insert(Style::Italic, None);
            }

            Tag::Table(..)
            | Tag::TableHead
            | Tag::TableRow
            | Tag::TableCell
            | Tag::BlockQuote
            | Tag::Code
            | Tag::List(_)
            | Tag::Image(..)
            | Tag::FootnoteDefinition(_) => {}
        }
    }

    fn end_tag(&mut self, tag: Tag) {
        match tag {
            // Blocks
            Tag::Paragraph => {
                self.body.close(hashmap! { "tag".into() => "p".into() });
                self.bare_text = true;
            }
            Tag::Header(level) => {
                let tag = format!("h{}", level);
                self.body.close(hashmap! { "tag".into() => tag });
                self.bare_text = true;
            }
            Tag::CodeBlock(_) => {
                self.body.close(hashmap! { "tag".into() => "pre".into() });
                self.bare_text = true;
            }

            // List items
            Tag::Item => {
                self.body
                    .close(hashmap! { "tag".into() => "bullet".into() });
                self.bare_text = true;
            }

            // Block objects
            Tag::Rule => {
                self.body.close(hashmap! { "tag".into() => "hr".into() });
            }
            Tag::Image(_, _) => (), // shouldn't happen, handled in start

            // Spans
            Tag::Link(..) => {
                self.styles.remove(&Style::Link);
            }
            Tag::Strong => {
                self.styles.remove(&Style::Bold);
            }
            Tag::Emphasis => {
                self.styles.remove(&Style::Italic);
            }

            Tag::FootnoteDefinition(_)
            | Tag::Code
            | Tag::TableCell
            | Tag::Table(_)
            | Tag::TableHead
            | Tag::TableRow
            | Tag::List(_)
            | Tag::BlockQuote => {}
        }
    }
}

pub fn markdown_to_doc(input: &str) -> Result<DocSpan, Error> {
    let parser = Parser::new(input);
    let mut doc_writer = DocWriter::new();
    {
        let mut ctx = Ctx {
            iter: parser,
            body: &mut doc_writer,
            styles: hashmap!{ Style::Normie => None },
            bare_text: true,
        };
        ctx.run();
    }
    doc_writer.result()
}

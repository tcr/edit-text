use failure::Error;
use oatie::doc::*;
use oatie::rtf::*;
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
    body: &'b mut DocWriter<RtfSchema>,
    styles: StyleSet,
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
                    self.body.place(&DocText(
                        self.styles.clone(),
                        DocString::from_str(text.as_ref()),
                    ));
                    if self.bare_text {
                        self.body.close(Attrs::Text);
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
                    self.body.place(&DocText(
                        self.styles.clone(),
                        DocString::from_str(" "),
                    ));
                    if self.bare_text {
                        self.body.close(Attrs::Text);
                    }
                }
                HardBreak => {
                    self.body.place(&DocText(
                        self.styles.clone(),
                        DocString::from_str("\n"),
                    ));
                }
                Html(html) => {
                    self.body.begin();
                    self.body.place(&DocText(
                        StyleSet::from(hashset!{ RtfStyle::Normie }),
                        DocString::from_str(&html),
                    ));
                    self.body.close(Attrs::Html);
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
                // FIXME
                // self.styles.insert(Style::Link, Some(dest.to_string()));
                self.styles.insert(RtfStyle::Link);
            }
            Tag::Strong => {
                self.styles.insert(RtfStyle::Bold);
            }
            Tag::Emphasis => {
                self.styles.insert(RtfStyle::Italic);
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

    fn end_tag(&mut self, tag: Tag<'_>) {
        match tag {
            // Blocks
            Tag::Paragraph => {
                self.body.close(Attrs::Text);
                self.bare_text = true;
            }
            Tag::Header(level) => {
                self.body.close(Attrs::Header(level as u8));
                self.bare_text = true;
            }
            Tag::CodeBlock(_) => {
                self.body.close(Attrs::Code);
                self.bare_text = true;
            }

            // List items
            Tag::Item => {
                self.body.close(Attrs::ListItem);
                self.bare_text = true;
            }

            // Block objects
            Tag::Rule => {
                self.body.close(Attrs::Rule);
            }
            Tag::Image(_, _) => (), // shouldn't happen, handled in start

            // Spans
            Tag::Link(..) => {
                self.styles.remove(&RtfStyle::Link);
            }
            Tag::Strong => {
                self.styles.remove(&RtfStyle::Bold);
            }
            Tag::Emphasis => {
                self.styles.remove(&RtfStyle::Italic);
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

pub fn markdown_to_doc(input: &str) -> Result<DocSpan<RtfSchema>, Error> {
    let parser = Parser::new(input);
    let mut doc_writer = DocWriter::new();
    {
        let mut ctx = Ctx {
            iter: parser,
            body: &mut doc_writer,
            styles: StyleSet::from(hashset!{ RtfStyle::Normie }),
            bare_text: true,
        };
        ctx.run();
    }
    doc_writer.result()
}

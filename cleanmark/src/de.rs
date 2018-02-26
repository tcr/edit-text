use failure::Error;
use oatie::doc::*;
use oatie::writer::DocWriter;
use pulldown_cmark::{Event::{self, End, FootnoteReference, HardBreak, Html, InlineHtml,
                             SoftBreak, Start, Text},
                     Parser, Tag};

struct Ctx<'b, I> {
    iter: I,
    body: &'b mut DocWriter,
}

impl<'a, 'b, I: Iterator<Item = Event<'a>>> Ctx<'b, I> {
    pub fn run(&mut self) {
        while let Some(event) = self.iter.next() {
            match event {
                Start(tag) => self.start_tag(tag),
                End(tag) => self.end_tag(tag),
                Text(text) => {
                    self.body.place(&DocChars(text.to_string()));
                }
                HardBreak => {
                    self.body.place(&DocChars("\n".to_string()));
                }
                SoftBreak | Html(..) | InlineHtml(..) | FootnoteReference(..) => {}
            }
        }
    }

    fn start_tag(&mut self, tag: Tag<'a>) {
        match tag {
            Tag::Paragraph => {
                self.body.begin();
            }
            Tag::Header(_level) => {
                self.body.begin();
            }
            Tag::CodeBlock(_info) => {
                self.body.begin();
            }
            Tag::Item => {
                self.body.begin();
            }

            Tag::Rule
            | Tag::Table(..)
            | Tag::TableHead
            | Tag::TableRow
            | Tag::TableCell
            | Tag::BlockQuote
            | Tag::Emphasis
            | Tag::Strong
            | Tag::Code
            | Tag::List(_)
            | Tag::Link(..)
            | Tag::Image(..)
            | Tag::FootnoteDefinition(_) => {}
        }
    }

    fn end_tag(&mut self, tag: Tag) {
        match tag {
            Tag::Paragraph => {
                self.body.close(hashmap! { "tag".into() => "p".into() });
            }
            Tag::Header(level) => {
                let tag = format!("h{}", level);
                self.body.close(hashmap! { "tag".into() => tag });
            }
            Tag::CodeBlock(_) => {
                self.body.close(hashmap! { "tag".into() => "pre".into() });
                // self.buf.push_str("</pre>\n"),
            }
            Tag::Item => {
                self.body
                    .close(hashmap! { "tag".into() => "bullet".into() });
            }

            Tag::Rule => (),
            Tag::Image(_, _) => (), // shouldn't happen, handled in start

            Tag::FootnoteDefinition(_)
            | Tag::Code
            | Tag::TableCell
            | Tag::Link(_, _)
            | Tag::Table(_)
            | Tag::TableHead
            | Tag::TableRow
            | Tag::Emphasis
            | Tag::Strong
            | Tag::List(_)
            | Tag::BlockQuote => {}
        }
    }
}

pub fn markdown_to_doc(input: &str) -> Result<DocSpan, Error> {
    let parser = Parser::new(input);
    let mut doc_writer = DocWriter::new();
    let mut ctx = Ctx {
        iter: parser,
        body: &mut doc_writer,
    };
    ctx.run();
    doc_writer.result()
}

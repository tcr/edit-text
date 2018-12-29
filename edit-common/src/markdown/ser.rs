use failure::Error;
use oatie::doc::*;
use oatie::rtf::*;
use oatie::stepper::DocStepper;
use pulldown_cmark::{
    Event,
    Tag,
};
use pulldown_cmark_to_cmark::fmt::cmark;

struct DocToMarkdown<'a, 'b> {
    doc_stepper: DocStepper<'a, RtfSchema>,
    queue: Vec<Event<'b>>,
}

impl<'a, 'b> DocToMarkdown<'a, 'b> {
    fn new(doc: &'a DocSpan<RtfSchema>) -> Self {
        DocToMarkdown {
            doc_stepper: DocStepper::new(doc),
            queue: vec![],
        }
    }
}

impl<'a, 'b> Iterator for DocToMarkdown<'a, 'b> {
    type Item = Event<'b>;

    fn next(&mut self) -> Option<Event<'b>> {
        if self.queue.len() > 0 {
            return Some(self.queue.remove(0));
        }

        match self.doc_stepper.head() {
            Some(DocGroup(ref attrs, ref body)) => {
                let res = Some(match attrs {
                    Attrs::Text => Event::Start(Tag::Paragraph),
                    Attrs::Header(level) => {
                        Event::Start(Tag::Header(*level as i32))
                    }
                    Attrs::Code => Event::Start(Tag::CodeBlock("".into())),
                    Attrs::Html => {
                        let mut out = String::new();
                        for child in body {
                            match *child {
                                DocChars(_, ref text) => {
                                    out.push_str(text.as_str());
                                }
                                _ => {}
                            }
                        }
                        self.doc_stepper.next();
                        return Some(Event::Html(out.into()));
                    }
                    Attrs::ListItem => {
                        if let Some(DocGroup(ref pre_attrs, _)) = self.doc_stepper.unhead() {
                            if *pre_attrs != Attrs::ListItem {
                                self.doc_stepper.enter();
                                return Some(Event::Start(Tag::Item));
                            }
                        }
                        self.queue.push(Event::Start(Tag::Item));
                        Event::Start(Tag::List(None))
                    }
                    Attrs::Caret { .. } => {
                        self.doc_stepper.next();
                        return self.next();
                    }
                    Attrs::Rule => Event::Start(Tag::Rule),
                    // _ => {
                    //     eprintln!("Unexpected tag {:?}!", attrs);
                    //     self.doc_stepper.next();
                    //     return self.next();
                    // }
                });
                self.doc_stepper.enter();
                res
            }
            Some(DocChars(ref styles, ref text)) => {
                // Styling.
                let text_event = Event::Text(text.to_string().replace("\n", "  \n").into());
                let res = if styles.contains(&RtfStyle::Bold) {
                    self.queue.push(text_event);
                    self.queue.push(Event::End(Tag::Strong));
                    Some(Event::Start(Tag::Strong))
                } else {
                    Some(text_event)
                };

                self.doc_stepper.next();
                res
            }
            None => {
                if self.doc_stepper.is_done() {
                    None
                } else {
                    let mut stepper_clone = self.doc_stepper.clone();
                    stepper_clone.unenter();
                    let attrs = match stepper_clone.head() {
                        Some(DocGroup(ref attrs, _)) => attrs.clone(),
                        _ => unreachable!(),
                    };
                    self.doc_stepper.exit();
                    Some(match attrs {
                        Attrs::Text => Event::End(Tag::Paragraph),
                        Attrs::Header(level) => {
                            Event::End(Tag::Header(level as i32))
                        }
                        Attrs::Code => {
                            self.queue.push(Event::End(Tag::CodeBlock("".into())));
                            Event::Text("\n".to_string().into())
                        }
                        Attrs::ListItem => {
                            if let Some(DocGroup(ref post_attrs, _)) = self.doc_stepper.head() {
                                if *post_attrs != Attrs::ListItem {
                                    self.queue.push(Event::End(Tag::List(None)));
                                }
                            }
                            Event::End(Tag::Item)
                        }
                        Attrs::Rule => Event::End(Tag::Rule),
                        _ => unimplemented!(),
                    })
                }
            }
        }
    }
}

pub fn doc_to_markdown(doc: &DocSpan<RtfSchema>) -> Result<String, Error> {
    let to_mark = DocToMarkdown::new(&doc);
    let mut buf = String::new();
    cmark(to_mark, &mut buf, None)?;
    Ok(buf)
}

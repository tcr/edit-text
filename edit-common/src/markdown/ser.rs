use failure::Error;
use oatie::doc::*;
use oatie::stepper::DocStepper;
use pulldown_cmark::{
    Event,
    Tag,
};
use pulldown_cmark_to_cmark::fmt::cmark;

struct DocToMarkdown<'a, 'b> {
    doc_stepper: DocStepper<'a>,
    queue: Vec<Event<'b>>,
}

impl<'a, 'b> DocToMarkdown<'a, 'b> {
    fn new(doc: &'a DocSpan) -> Self {
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
                let res = Some(match attrs["tag"].as_ref() {
                    "p" => Event::Start(Tag::Paragraph),
                    "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                        let level = attrs["tag"][1..].parse::<i32>().unwrap_or(1);
                        Event::Start(Tag::Header(level))
                    }
                    "pre" => Event::Start(Tag::CodeBlock("".into())),
                    "html" => {
                        let mut out = String::new();
                        for child in body {
                            match *child {
                                DocChars(ref text) => {
                                    out.push_str(text.as_str());
                                }
                                _ => {}
                            }
                        }
                        self.doc_stepper.next();
                        return Some(Event::Html(out.into()));
                    }
                    "bullet" => {
                        if let Some(DocGroup(ref pre_attrs, _)) = self.doc_stepper.unhead() {
                            if pre_attrs["tag"] == "bullet" {
                                self.doc_stepper.enter();
                                return Some(Event::Start(Tag::Item));
                            }
                        }
                        self.queue.push(Event::Start(Tag::Item));
                        Event::Start(Tag::List(None))
                    }
                    "caret" => {
                        self.doc_stepper.next();
                        return self.next();
                    }
                    "hr" => Event::Start(Tag::Rule),
                    _ => {
                        eprintln!("Unexpected tag {:?}!", attrs["tag"]);
                        self.doc_stepper.next();
                        return self.next();
                    }
                });
                self.doc_stepper.enter();
                res
            }
            Some(DocChars(ref text)) => {

                // Styling.
                let text_event = Event::Text(text.to_string().replace("\n", "  \n").into());
                let res = if let Some(styles) = text.styles() {
                    if styles.contains_key(&Style::Bold) {
                        self.queue.push(text_event);
                        self.queue.push(Event::End(Tag::Strong));
                        Some(Event::Start(Tag::Strong))
                    } else {
                        Some(text_event)
                    }
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
                    Some(match attrs["tag"].as_ref() {
                        "p" => Event::End(Tag::Paragraph),
                        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                            let level = attrs["tag"][1..].parse::<i32>().unwrap_or(1);
                            Event::End(Tag::Header(level))
                        }
                        "pre" => {
                            self.queue.push(Event::End(Tag::CodeBlock("".into())));
                            Event::Text("\n".to_string().into())
                        }
                        "bullet" => {
                            if let Some(DocGroup(ref post_attrs, _)) = self.doc_stepper.head() {
                                if post_attrs["tag"] != "bullet" {
                                    self.queue.push(Event::End(Tag::List(None)));
                                }
                            }
                            Event::End(Tag::Item)
                        }
                        "hr" => Event::End(Tag::Rule),
                        _ => unimplemented!(),
                    })
                }
            }
        }
    }
}

pub fn doc_to_markdown(doc: &DocSpan) -> Result<String, Error> {
    let to_mark = DocToMarkdown::new(&doc);
    let mut buf = String::new();
    cmark(to_mark, &mut buf, None)?;
    Ok(buf)
}

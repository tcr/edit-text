use failure::Error;
use oatie::doc::*;
use oatie::stepper::DocStepper;
use pulldown_cmark::{Event, Tag};
use pulldown_cmark_to_cmark::fmt::cmark;

struct DocToMarkdown<'a> {
    doc_stepper: DocStepper,
    queue: Vec<Event<'a>>,
}

impl<'a> DocToMarkdown<'a> {
    fn new(doc: &DocSpan) -> Self {
        DocToMarkdown {
            doc_stepper: DocStepper::new(doc),
            queue: vec![],
        }
    }
}

impl<'a> Iterator for DocToMarkdown<'a> {
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Event<'a>> {
        if self.queue.len() > 0 {
            return Some(self.queue.remove(0));
        }

        match self.doc_stepper.head() {
            Some(DocGroup(ref attrs, _)) => {
                let res = Some(match attrs["tag"].as_ref() {
                    "p" => Event::Start(Tag::Paragraph),
                    "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                        let level = attrs["tag"][1..].parse::<i32>().unwrap_or(1);
                        Event::Start(Tag::Header(level))
                    }
                    "pre" => Event::Start(Tag::CodeBlock("".into())),
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
                self.doc_stepper.next();
                Some(Event::Text(text.to_string().replace("\n", "  \n").into()))
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

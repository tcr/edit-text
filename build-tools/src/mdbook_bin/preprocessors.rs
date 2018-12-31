//! Svgbob preprocessing

use mdbook::book::{
    Book,
    BookItem,
};
use mdbook::errors::Error;
use mdbook::preprocess::*;
use regex::{
    Captures,
    Regex,
};

pub struct SvgbobPreprocessor;

impl Preprocessor for SvgbobPreprocessor {
    fn name(&self) -> &str {
        "svgbob"
    }

    fn run(&self, ctx: &PreprocessorContext, book: &mut Book) -> Result<(), Error> {
        process(&mut book.sections)
    }
}

fn process<'a, I>(items: I) -> Result<(), Error>
where
    I: IntoIterator<Item = &'a mut BookItem> + 'a,
{
    let re = Regex::new(r"```(?:svg)?bob\n([\S\s]+?)\n```").unwrap();
    for item in items {
        if let BookItem::Chapter(ref mut chapter) = item {
            // eprintln!("svgbob: processing chapter '{}'", chapter.name);
            let res = re.replace_all(&chapter.content, |captures: &Captures| {
                let bob_source = captures.get(1).unwrap().as_str();
                // eprintln!("!!!! REPLACING.... {:?}", bob_source);
                format!("{}", svgbob::to_svg(bob_source)).replace("\n", " ")
            });
            // if re.is_match(&chapter.content) {
            //     eprintln!("\n\n\nresult {}\n\n\n", res);
            // }
            chapter.content = res.to_string();
            process(&mut chapter.sub_items);
        }
    }
    Ok(())
}

pub struct TOCPreprocessor;

impl Preprocessor for TOCPreprocessor {
    fn name(&self) -> &str {
        "toc"
    }

    fn run(&self, ctx: &PreprocessorContext, book: &mut Book) -> Result<(), Error> {
        for section in &mut book.sections {
            if let BookItem::Chapter(ref mut chapter) = section {
                if !chapter.sub_items.is_empty() {
                    let toc: Vec<String> = chapter
                        .sub_items
                        .iter()
                        .filter_map(|sub_item| {
                            if let BookItem::Chapter(ref chapter) = sub_item {
                                Some(format!(
                                    "1. [{}]({})",
                                    chapter.name,
                                    chapter.path.to_string_lossy()
                                ))
                            } else {
                                None
                            }
                        })
                        .collect();

                    chapter.content = chapter
                        .content
                        .replace("{{#toc}}", &format!("\n\n{}\n\n", toc.join("\n")));
                }
            }
        }
        Ok(())
    }
}

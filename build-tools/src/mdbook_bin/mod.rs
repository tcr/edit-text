#![allow(warnings)]

pub mod serve;
pub mod watch;
pub mod preprocessors;

use mdbook::MDBook;

pub fn inject_preprocessors(book: &mut MDBook) {
    book.with_preprecessor(self::preprocessors::SvgbobPreprocessor);
    book.with_preprecessor(self::preprocessors::TOCPreprocessor);
}

#![allow(warnings)]

pub mod preprocessors;
pub mod serve;
pub mod watch;

use mdbook::MDBook;

pub fn inject_preprocessors(book: &mut MDBook) {
    book.with_preprecessor(self::preprocessors::SvgbobPreprocessor);
    book.with_preprecessor(self::preprocessors::TOCPreprocessor);
}

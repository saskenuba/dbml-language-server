//! DBML is a very simple language with a single document structure, with no imports at all.
//!
//! With this in mind, each request parses the whole document, since there is no need for the
//! additional complexity of storing old document trees.

use std::sync::Mutex;

use once_cell::sync::Lazy;
use tree_sitter::{Language, Parser};

pub mod file;
pub mod providers;
pub mod wrappers;

extern "C" {
    fn tree_sitter_dbml() -> Language;
}

#[allow(dead_code)]
pub static LANGUAGE: Lazy<Language> = Lazy::new(|| unsafe { tree_sitter_dbml() });

#[allow(dead_code)]
pub static PARSER: Lazy<Mutex<Parser>> = Lazy::new(|| {
    let mut parser = Parser::new();
    parser.set_language(*LANGUAGE).unwrap();
    Mutex::new(parser)
});

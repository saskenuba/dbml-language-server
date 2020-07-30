use std::{
    ffi::OsString,
    fs::{File, OpenOptions},
    io::{BufReader, Read},
};

use anyhow::Result;
use tree_sitter::Tree;
use url::Url;

use crate::PARSER;

/// Convenience function that parses a file from a URI
pub fn open_and_parse(file_path: &Url, old_tree: Option<&Tree>) -> Result<Option<Tree>> {
    read_file(file_path).map(|file_bytes| parse_file(file_bytes, old_tree))
}

/// Convenience function that parses a file from a URI
pub fn open_and_parse_with_source(
    file_path: &Url,
    old_tree: Option<&Tree>,
) -> Result<(Option<Tree>, Vec<u8>)> {
    read_file(file_path).map(|file_bytes| (parse_file(&file_bytes, old_tree), file_bytes))
}

fn read_file(file_uri: &Url) -> Result<Vec<u8>> {
    let mut file_path = None;

    if !file_uri.origin().is_tuple() {
        // We can unwrap because we are certain that it is an os path
        file_path = Some(file_uri.to_file_path().unwrap().into_os_string());
    }

    let file = read_from_disk(file_path.unwrap())?;

    let mut buf_reader = BufReader::new(file);
    let mut storage = Vec::new();
    buf_reader.read_to_end(&mut storage)?;
    Ok(storage)
}

pub fn parse_file<T: AsRef<[u8]>>(file: T, old_tree: Option<&Tree>) -> Option<Tree> {
    PARSER.lock().unwrap().parse(file, old_tree)
}

fn read_from_disk(file_path: OsString) -> Result<File> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .open(file_path)
        .map_err(|e| e.into())
}

//! Wrappers useful for converting from/into tree sitter to/from lsp server.

use std::ops::{Deref, DerefMut};
use tower_lsp::lsp_types::{Position as LspPosition, Range as LspRange};
use tree_sitter::{Point as TreePoint, Range as TreeRange};

#[derive(Debug, Clone, Copy)]
pub struct Point(pub TreePoint);
#[derive(Debug, Clone, Copy)]
pub struct Position(pub LspPosition);
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct Range(pub LspRange);

impl From<TreeRange> for Range {
    fn from(range: TreeRange) -> Self {
        Self {
            0: LspRange {
                start: LspPosition {
                    line: range.start_point.row as u64,
                    character: range.start_point.column as u64,
                },
                end: LspPosition {
                    line: range.end_point.row as u64,
                    character: range.end_point.column as u64,
                },
            },
        }
    }
}

impl From<LspPosition> for Point {
    fn from(pos: LspPosition) -> Self {
        Self {
            0: TreePoint {
                row: pos.line as usize,
                column: pos.character as usize,
            },
        }
    }
}
impl Point {
    pub fn column_start(&self) -> Point {
        Self {
            0: TreePoint {
                row: self.row,
                column: 0,
            },
        }
    }

    pub fn line_above(&self) -> Point {
        Self {
            0: TreePoint {
                row: self.row - 1,
                column: self.column,
            },
        }
    }
}

impl Deref for Range {
    type Target = LspRange;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for Point {
    type Target = TreePoint;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Point {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

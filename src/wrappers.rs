use std::ops::Deref;
use tower_lsp::lsp_types::{Position as LspPosition, Range as LspRange};
use tree_sitter::{Point as TreePoint, Range as TreeRange};

pub struct Point(pub TreePoint);
pub struct Position(pub LspPosition);
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

use util::indexed_vec::Delta;
use source::parse_result::ByteIndex;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter, Result};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LineColumn {
    pub line: u32,
    pub column: Delta<ByteIndex>,
}

// Inclusive line/column range
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LineColumnRange {
    pub start: LineColumn,
    pub end: Option<LineColumn>,
}

impl LineColumn {
    pub fn new(line: u32, column: Delta<ByteIndex>) -> LineColumn {
        LineColumn { line, column }
    }
}

impl LineColumnRange {
    pub fn new(start: LineColumn, end: LineColumn) -> LineColumnRange {
        LineColumnRange {
            start,
            end: Some(end),
        }
    }
    pub fn zero_width(start: LineColumn) -> LineColumnRange {
        LineColumnRange { start, end: None }
    }
}

impl PartialOrd for LineColumn {
    fn partial_cmp(&self, other: &LineColumn) -> Option<Ordering> {
        let result = self.line.partial_cmp(&other.line);
        match result {
            Some(Ordering::Equal) => self.column.partial_cmp(&other.column),
            _ => result,
        }
    }
}

impl Display for LineColumn {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

impl Display for LineColumnRange {
    fn fmt(&self, f: &mut Formatter) -> Result {
        if let Some(ref end) = self.end {
            if self.start.line == end.line {
                if self.start.column == end.column {
                    write!(f, "{}:{}", self.start.line, self.start.column)
                } else {
                    write!(
                        f,
                        "{}:{}-{}",
                        self.start.line, self.start.column, end.column
                    )
                }
            } else {
                write!(
                    f,
                    "{}:{}-{}:{}",
                    self.start.line, self.start.column, end.line, end.column
                )
            }
        } else {
            write!(f, "{}:{}<0>", self.start.line, self.start.column)
        }
    }
}

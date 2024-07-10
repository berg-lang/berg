use derive_more::{Display, From, Into};
use nonzero_ext::nonzero;
use std::{cmp::Ordering, fmt, num::NonZeroU32, ops::RangeInclusive};

use berg_util::Delta;

use crate::syntax::bytes::{ByteIndex, ByteRange};

///
/// Data about lines in the document.
/// 
/// The first line is at the beginning of the document.
/// 
/// Line ranges include the line endings. All lines except the last one include a line ending and
/// are thus not empty.and may be an empty
/// line.
/// Lines start at the beginning of the document and after a line terminator.
/// Lines end after a line 
/// 
#[derive(Debug)]
pub struct DocumentLines {
    pub line_starts: Box<[ByteIndex]>,
    pub eof: ByteIndex,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LineColumn {
    pub line: LineNumber,
    pub column: ColumnNumber,
}

// Inclusive line/column range
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LineColumnRange {
    pub start: LineColumn,
    pub end: Option<LineColumn>,
}

#[derive(Clone, Copy, Debug, Display, PartialEq, Eq, PartialOrd, Ord, From, Into)]
pub struct LineNumber(pub NonZeroU32);

#[derive(Clone, Copy, Debug, Display, PartialEq, Eq, PartialOrd, Ord, From, Into)]
pub struct ColumnNumber(pub NonZeroU32);

impl Default for DocumentLines {
    fn default() -> Self {
        Self {
            line_starts: Box::new([0.into()]),
            eof: 0.into(),
        }
    }
}

impl DocumentLines {
    pub fn line_count(&self) -> u32 {
        self.line_starts.len() as u32
    }

    pub fn first_line_number(&self) -> LineNumber {
        LineNumber(nonzero!(1u32))
    }

    pub fn last_line_number(&self) -> LineNumber {
        unsafe { LineNumber::new_unchecked(self.line_count()) }
    }

    pub fn line_numbers(&self) -> RangeInclusive<LineNumber> {
        self.first_line_number()..=self.last_line_number()
    }

    pub fn line_start(&self, line: LineNumber) -> ByteIndex {
        self.line_starts[(line.0.get() - 1) as usize]
    }

    pub fn line_end(&self, line: LineNumber) -> ByteIndex {
        if line == self.last_line_number() {
            self.eof
        } else {
            self.line_start(line.next_line())
        }
    }

    pub fn location(&self, index: ByteIndex) -> LineColumn {
        let line = match self.line_starts.binary_search(&index) {
            // If the index happens to be at the start of a line, we'll get
            // the 0-based index of that line, and we want 1-based.
            Ok(line) => line + 1,
            // If the index is not the start of a line, we'll get the index of the
            // *next* line, which is the same as a 1-based index to our line.
            Err(line) => line,
        };

        let line = unsafe { LineNumber::new_unchecked(line as u32) };
        let bytes_to_column = index - self.line_start(line);
        let column = unsafe { ColumnNumber::new_unchecked(bytes_to_column.0.0 + 1) };
        LineColumn { line, column }
    }

    pub fn range(&self, range: &ByteRange) -> LineColumnRange {
        let start = self.location(range.start);
        if range.start == range.end {
            LineColumnRange { start, end: None }
        } else {
            let end = Some(self.location(range.end - 1));
            LineColumnRange { start, end }
        }
    }

    pub fn byte_index(&self, location: LineColumn) -> ByteIndex {
        self.line_start(location.line) + location.column.bytes_from_line_start()
    }

    #[allow(clippy::range_plus_one)]
    pub fn byte_range(&self, range: LineColumnRange) -> ByteRange {
        let start = self.byte_index(range.start);
        match range.end {
            Some(end) => start..(self.byte_index(end) + 1),
            None => start..start,
        }
    }
}

impl LineColumn {
    pub fn new(line: LineNumber, column: ColumnNumber) -> LineColumn {
        LineColumn { line, column }
    }
}

impl LineColumnRange {
    pub fn new(start: LineColumn, end: LineColumn) -> LineColumnRange {
        let end = Some(end);
        LineColumnRange { start, end }
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

impl fmt::Display for LineColumn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

impl fmt::Display for LineColumnRange {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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

impl LineNumber {
    pub const ONE: Self = Self(nonzero!(1u32));

    pub unsafe fn new_unchecked(value: impl Into<u32>) -> Self {
        Self(NonZeroU32::new_unchecked(value.into()))
    }

    pub fn next_line(self) -> Self {
        unsafe { Self::new_unchecked(self.0.get() + 1) }
    }
}

impl ColumnNumber {
    pub const ONE: Self = Self(nonzero!(1u32));

    pub unsafe fn new_unchecked(value: impl Into<u32>) -> Self {
        Self(NonZeroU32::new_unchecked(value.into()))
    }

    pub fn bytes_from_line_start(self) -> Delta<ByteIndex> {
        Delta(ByteIndex(self.0.get() - 1))
    }
}

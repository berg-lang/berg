use crate::syntax::{ByteIndex, ByteRange, WhitespaceIndex};
use crate::util::indexed_vec::Delta;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter, Result};
use std::str;
use std::u32;
use string_interner::StringInterner;
use string_interner::backend::StringBackend;

///
/// Debug data about the original source.
/// 
/// Includes enough character data to reconstruct the original source.
/// 
#[derive(Debug)]
pub struct CharData {
    // size in bytes
    // byte_size: usize,
    // Size in bytes
    pub size: ByteIndex,
    // checksum
    // time retrieved
    // time modified
    // system retrieved on
    
    ///
    /// Beginning index of each line.
    ///
    pub line_starts: Vec<ByteIndex>,

    ///
    /// Whitespace characters found in the document.
    /// 
    pub whitespace_characters: StringInterner<StringBackend<WhitespaceIndex>>,

    ///
    /// Ordered list of whitespace ranges, except ' ' and '\n'
    /// (which are the default for space and line ranges, respectively).
    /// 
    /// Only the starting byte index is stored, since the length is stored in
    /// [`whitespace_characters`].
    /// 
    pub whitespace_ranges: Vec<(WhitespaceIndex, ByteIndex)>,

    ///
    /// Ordered list of comments in the document.
    /// 
    /// These include the # character at the beginning of the comment and do *not*
    /// include the line ending character.
    /// 
    /// Comments may include non-UTF-8 characters. (This is why it's Vec<u8> and
    /// not String.)
    /// 
    pub comments: Vec<(Vec<u8>, ByteIndex)>,
}

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

///
/// An index into [`Whitespace::whitespace_characters`]
/// 
impl Default for CharData {
    fn default() -> Self {
        CharData {
            size: Default::default(),
            line_starts: vec!(0.into()),
            whitespace_characters: StringInterner::new(),
            whitespace_ranges: Default::default(),
            comments: Default::default(),
        }
    }
}

impl CharData {
    ///
    /// Add the run of whitespace to the whitespace list.
    /// 
    /// # Panics
    /// 
    /// * Panics if spaces.len() == 0
    /// * Panics if `append()` is not called with `start` increasing each time.
    /// * Panics if ByteIndex == u32::MAX
    /// 
    pub fn append_whitespace(&mut self, whitespace: &str, start: ByteIndex) -> WhitespaceIndex{
        let whitespace_index = self.whitespace_characters.get_or_intern(whitespace);
        self.whitespace_ranges.push((whitespace_index, start));
        whitespace_index
    }

    pub fn append_comment(&mut self, bytes: &[u8], start: ByteIndex) {
        self.comments.push((bytes.into(), start))
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

        let column = index + 1 - self.line_starts[line - 1];
        let line = line as u32;
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
        self.line_starts[(location.line-1) as usize] + location.column - 1
    }

    #[allow(clippy::range_plus_one)]
    pub fn byte_range(&self, range: LineColumnRange) -> ByteRange {
        let start = self.byte_index(range.start);
        match range.end {
            Some(end) => start..(self.byte_index(end)+1),
            None => start..start,
        }
    }
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

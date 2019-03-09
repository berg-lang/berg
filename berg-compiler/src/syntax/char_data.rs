use crate::syntax::{ByteIndex, ByteRange, ByteSlice};
use crate::util::indexed_vec::Delta;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter, Result};
use std::str;

#[derive(Debug)]
pub struct CharData {
    // size in bytes
    // byte_size: usize,
    // Size in Unicode codepoints
    pub(crate) size: ByteIndex,
    // checksum
    // time retrieved
    // time modified
    // system retrieved on
    /// Beginning index of each line.
    pub(crate) line_starts: Vec<ByteIndex>,
    /// Whitespace of each character type except ' ' and '\n' (those are the default whitespace)
    pub(crate) whitespace: Whitespace,
}

#[derive(Debug, Default)]
pub struct Whitespace {
    pub char_ranges: Vec<(String, Vec<ByteRange>)>,
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

impl Default for CharData {
    fn default() -> Self {
        CharData {
            size: Default::default(),
            line_starts: vec![ByteIndex::from(0)],
            whitespace: Default::default(),
        }
    }
}

impl CharData {
    pub(crate) fn append_line(&mut self, buffer: &ByteSlice, range: ByteRange) {
        self.line_starts.push(range.end);
        let newline_char = unsafe { str::from_utf8_unchecked(&buffer[range.clone()]) };
        self.whitespace.ranges_for_char(newline_char).push(range);
    }

    pub(crate) fn location(&self, index: ByteIndex) -> LineColumn {
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

    pub(crate) fn range(&self, range: &ByteRange) -> LineColumnRange {
        let start = self.location(range.start);
        if range.start == range.end {
            LineColumnRange { start, end: None }
        } else {
            let end = Some(self.location(range.end - 1));
            LineColumnRange { start, end }
        }
    }

    pub(crate) fn byte_index(&self, location: LineColumn) -> ByteIndex {
        self.line_starts[(location.line-1) as usize] + location.column - 1
    }
    #[allow(clippy::range_plus_one)]
    pub(crate) fn byte_range(&self, range: LineColumnRange) -> ByteRange {
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

impl Whitespace {
    pub fn append(&mut self, spaces: &str, start: ByteIndex) {
        let mut char_indices = spaces.char_indices();
        let (mut current_char_start, mut current_char) = char_indices.next().unwrap();
        for (next_char_start, next_char) in char_indices {
            if next_char == current_char {
                continue;
            }

            // Store the character (and the number of repeats)
            // We don't store ' ' since it's so common
            if current_char != ' ' {
                let space_start = start + current_char_start;
                let space_end = start + next_char_start;
                let space_char = unsafe {
                    spaces.get_unchecked(
                        current_char_start..current_char_start + current_char.len_utf8(),
                    )
                };
                self.ranges_for_char(space_char)
                    .push(space_start..space_end);
            }

            current_char = next_char;
            current_char_start = next_char_start;
        }
    }

    pub fn ranges_for_char(&mut self, space_char: &str) -> &mut Vec<ByteRange> {
        // Find the vec we're storing this character in. (e.g. \t)
        let index = match self
            .char_ranges
            .iter()
            .position(|&(ref range_space_char, _)| range_space_char == space_char)
        {
            Some(index) => index,
            None => {
                self.char_ranges
                    .push((space_char.to_string(), Default::default()));
                self.char_ranges.len() - 1
            }
        };
        &mut self.char_ranges[index].1
    }
}

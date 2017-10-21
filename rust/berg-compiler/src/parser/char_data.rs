use std::ops::Range;
use std::fmt::*;
use std::u32;

index_type! {
    pub struct ByteIndex(pub u32) <= u32::MAX;
}

#[derive(Debug)]
pub struct CharData {
    // size in bytes
    // byte_size: usize,
    // Size in Unicode codepoints
    pub byte_length: ByteIndex,
    // checksum
    // time retrieved
    // time modified
    // system retrieved on
    // Start indices of each line
    pub line_starts: Vec<ByteIndex>,
}

impl Default for CharData {
    fn default() -> Self { CharData { byte_length: ByteIndex::from(0), line_starts: vec![ByteIndex::from(0)] } }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LineColumn {
    pub line: u32,
    pub column: ByteIndex,
}

// Inclusive line/column range
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LineColumnRange {
    pub start: LineColumn,
    pub end: Option<LineColumn>,
}

impl CharData {
    pub fn append_line(&mut self, line_start_index: ByteIndex) {
        self.line_starts.push(line_start_index);
    }
    pub fn location(&self, index: ByteIndex) -> LineColumn {
        // TODO binary search to make it faster. But, meh.
        let mut line = self.line_starts.len();
        while self.line_starts[line - 1] > index {
            line -= 1
        }

        let column = index - self.line_starts[line - 1] + 1;
        let line = line as u32;
        LineColumn { line, column }
    }

    pub fn range(&self, range: Range<ByteIndex>) -> LineColumnRange {
        let start = self.location(range.start);
        if range.start == range.end {
            LineColumnRange { start, end: None }
        } else {
            let end = Some(self.location(range.end - 1));
            LineColumnRange { start, end }
        }
    }

    pub fn byte_length(&self) -> ByteIndex {
        self.byte_length
    }
}

impl LineColumn {
    pub fn new(line: u32, column: ByteIndex) -> LineColumn {
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
                        self.start.line,
                        self.start.column,
                        end.column
                    )
                }
            } else {
                write!(
                    f,
                    "{}:{}-{}:{}",
                    self.start.line,
                    self.start.column,
                    end.line,
                    end.column
                )
            }
        } else {
            write!(f, "{}:{}", self.start.line, self.start.column)
        }
    }
}

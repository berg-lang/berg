use std::cmp::Ordering;
use std::ops::Range;
use std::fmt::*;

// TODO make this struct X(usize) to make accidental cross-casting impossible
pub type ByteIndex = u32;

#[derive(Debug)]
pub struct CharData {
    // size in bytes
    // byte_size: usize,
    // Size in Unicode codepoints
    char_size: ByteIndex,
    // checksum
    // time retrieved
    // time modified
    // system retrieved on
    // Start indices of each line
    line_starts: Vec<ByteIndex>,
}

#[derive(Debug)]
pub struct LineColumn {
    pub line: u32,
    pub column: u32,
}

// Inclusive line/column range
#[derive(Debug)]
pub struct LineColumnRange {
    pub start: LineColumn,
    pub end: Option<LineColumn>,
}

impl CharData {
    pub fn new() -> CharData {
        CharData { char_size: 0, line_starts: vec![0] }
    }
    pub fn append_line(&mut self, line_start_index: ByteIndex) {
        self.line_starts.push(line_start_index);
    }
    pub fn location(&self, index: ByteIndex) -> LineColumn {
        // TODO binary search to make it faster. But, meh.
        let mut line = self.line_starts.len();
        while self.line_starts[line-1] > index {
            line -= 1
        }

        let column = index - self.line_starts[line-1] + 1;
        let line = line as u32;
        LineColumn { line, column }
    }

    pub fn range(&self, range: Range<ByteIndex>) -> LineColumnRange {
        let start = self.location(range.start);
        if range.start == range.end {
            LineColumnRange { start, end: None }
        } else {
            let end = Some(self.location(range.end-1));
            LineColumnRange { start, end }
        }
    }

    pub fn char_size(&self) -> ByteIndex {
        self.char_size
    }
}

impl LineColumn {
    pub fn none() -> LineColumn {
        LineColumn { line: 0, column: 0 }
    }
}

impl PartialEq for LineColumn {
    fn eq(&self, other: &LineColumn) -> bool {
        self.line == other.line && self.column == other.column
    }
}

impl PartialOrd for LineColumn {
    fn partial_cmp(&self, other: &LineColumn) -> Option<Ordering> {
        let result = self.line.partial_cmp(&other.column);
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
                    write!(f, "{}:{}-{}", self.start.line, self.start.column, end.column)
                }
            } else {
                write!(f, "{}:{}-{}:{}", self.start.line, self.start.column, end.line, end.column)
            }
        } else {
            write!(f, "{}:{}", self.start.line, self.start.column)
        }
    }
}

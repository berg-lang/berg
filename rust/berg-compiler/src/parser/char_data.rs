use std::cmp::Ordering;
use std::ops::Range;

// TODO make this struct X(usize) to make accidental cross-casting impossible
pub type ByteIndex = usize;

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
    pub line: usize,
    pub column: usize,
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
            line += 1
        }

        let column = index - self.line_starts[line-1] + 1;
        LineColumn { line, column }
    }

    pub fn range(&self, range: &Range<ByteIndex>) -> Range<LineColumn> {
        let start = self.location(range.start);
        let end = self.location(range.end);
        Range { start, end }
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

use internals::*;

pub struct SourceMetadata {
    // size in bytes
    // byte_size: usize,
    // Size in Unicode codepoints
    char_size: usize,
    // checksum
    // time retrieved
    // time modified
    // system retrieved on
    /// The index of each line
    line_starts: Vec<usize>,
}

impl SourceMetadata {
    pub fn new(char_size: usize) -> SourceMetadata {
        SourceMetadata { char_size, line_starts: vec![0] }
    }
    fn append_line(&mut self, line_start_index: usize) {
        self.line_starts.push(line_start_index);
    }
    pub fn location(&self, index: usize) -> LineColumn {
        // TODO binary search to make it faster. But, meh.
        let mut line = self.line_starts.len();
        while self.line_starts[line-1] > index {
            line += 1
        }

        let column = index - self.line_starts[line-1] + 1;
        LineColumn { line, column }
    }
    pub fn range(&self, range: &Range<usize>) -> Range<LineColumn> {
        let start = self.location(range.start);
        let end = self.location(range.end);
        Range { start, end }
    }
}

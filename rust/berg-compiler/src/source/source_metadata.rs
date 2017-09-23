use internals::*;

#[derive(Debug)]
pub struct SourceMetadata<'a> {
    source: &'a Source,
    // size in bytes
    // byte_size: usize,
    // Size in Unicode codepoints
    char_size: SourceIndex,
    // checksum
    // time retrieved
    // time modified
    // system retrieved on
    // Start indices of each line
    line_starts: Vec<SourceIndex>,
}

impl<'a> SourceMetadata<'a> {
    pub fn new(source: &'a Source) -> SourceMetadata<'a> {
        SourceMetadata { source, char_size: 0, line_starts: vec![0] }
    }
    pub fn append_line(&mut self, line_start_index: SourceIndex) {
        self.line_starts.push(line_start_index);
    }
    pub fn location(&self, index: SourceIndex) -> LineColumn {
        // TODO binary search to make it faster. But, meh.
        let mut line = self.line_starts.len();
        while self.line_starts[line-1] > index {
            line += 1
        }

        let column = index - self.line_starts[line-1] + 1;
        LineColumn { line, column }
    }

    pub fn range(&self, range: &Range<SourceIndex>) -> Range<LineColumn> {
        let start = self.location(range.start);
        let end = self.location(range.end);
        Range { start, end }
    }
    pub fn source(&self) -> &'a Source {
        &self.source
    }
    pub fn char_size(&self) -> SourceIndex {
        self.char_size
    }
}

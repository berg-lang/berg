use compile_errors::*;

use std::cmp::Ordering;
use std::env;
use std::io;
use std::ops::Range;
use std::path::PathBuf;

pub use source::*;

pub struct Berg {
    root: io::Result<PathBuf>,
    sources: Vec<Box<Source>>,
}

// TODO make these struct X(usize) to make accidental cross-casting impossible
pub type GraphemeIndex = usize;
pub type ByteIndex = usize;

#[derive(Debug)]
pub struct ParseResult {
    pub metadata: SourceMetadata,
    pub expressions: Vec<SyntaxExpression>,
    pub errors: CompileErrors,
}

// ExpressionType, String, LeftChild, RightChild
#[derive(Debug)]
pub struct SyntaxExpression {
    pub expression_type: SyntaxExpressionType,
    pub string: String,
    pub start: usize,
}

#[derive(Debug)]
pub enum SyntaxExpressionType {
    IntegerLiteral,
}

#[derive(Debug)]
pub struct SourceMetadata {
    // size in bytes
    // byte_size: usize,
    // Size in Unicode codepoints
    char_size: GraphemeIndex,
    // checksum
    // time retrieved
    // time modified
    // system retrieved on
    // Start indices of each line
    line_starts: Vec<GraphemeIndex>,
}

#[derive(Debug)]
pub struct LineColumn {
    pub line: usize,
    pub column: usize,
}

impl Berg {
    pub fn from_env() -> Berg {
        Berg { root: env::current_dir(), sources: vec![] }
    }
    pub fn new(root: PathBuf) -> Berg {
        Berg { root: Ok(root), sources: vec![] }
    }

    pub fn root(&self) -> &io::Result<PathBuf> {
        &self.root
    }

    pub fn add_file_source(&mut self, path: PathBuf) {
        self.sources.push(Box::new(FileSource::new(path)))
    }
    pub fn add_string_source(&mut self, name: String, contents: String) {
        self.sources.push(Box::new(StringSource::new(name, contents)))
    }
    pub fn parse(&self) -> Vec<ParseResult> {
        self.sources.iter().map(|source| source.parse(&self)).collect()
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

impl SourceMetadata {
    pub fn new() -> SourceMetadata {
        SourceMetadata { char_size: 0, line_starts: vec![0] }
    }
    pub fn append_line(&mut self, line_start_index: GraphemeIndex) {
        self.line_starts.push(line_start_index);
    }
    pub fn location(&self, index: GraphemeIndex) -> LineColumn {
        // TODO binary search to make it faster. But, meh.
        let mut line = self.line_starts.len();
        while self.line_starts[line-1] > index {
            line += 1
        }

        let column = index - self.line_starts[line-1] + 1;
        LineColumn { line, column }
    }

    pub fn range(&self, range: &Range<GraphemeIndex>) -> Range<LineColumn> {
        let start = self.location(range.start);
        let end = self.location(range.end);
        Range { start, end }
    }
    pub fn char_size(&self) -> GraphemeIndex {
        self.char_size
    }
}

pub use compiler::compile_error::*;
pub use parser::results::SyntaxExpressionType::*;

use std::cmp::Ordering;
use std::ops::Range;

#[derive(Debug)]
pub struct ParseResult {
    char_data: CharData,
    expressions: Vec<SyntaxExpression>,
}

impl ParseResult {
    pub fn new(char_data: CharData, expressions: Vec<SyntaxExpression>) -> ParseResult {
        ParseResult { char_data, expressions }
    }
}

// ExpressionType, String, LeftChild, RightChild
#[derive(Debug)]
pub struct SyntaxExpression {
    pub expression_type: SyntaxExpressionType,
    pub start: ByteIndex,
    pub string: String,
}

impl SyntaxExpression {
    pub fn new(expression_type: SyntaxExpressionType, start: ByteIndex, string: String) -> SyntaxExpression {
        SyntaxExpression { expression_type, start, string }
    }
}

// TODO make this struct X(usize) to make accidental cross-casting impossible
pub type ByteIndex = usize;

#[derive(Debug)]
pub enum SyntaxExpressionType {
    IntegerLiteral,
}

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


use SyntaxExpressionType::*;
use compile_errors::*;
use parser;

use std::cmp::Ordering;
use std::env;
use std::ffi::OsStr;
use std::fmt;
use std::io;
use std::ops::Range;
use std::path::PathBuf;

pub struct Berg {
    root: io::Result<PathBuf>,
}

#[derive(Debug)]
pub enum Source {
    File(PathBuf),
    String(String, String),
}

// TODO make these struct X(usize) to make accidental cross-casting impossible
pub type GraphemeIndex = usize;
pub type ByteIndex = usize;

#[derive(Debug)]
pub struct ParseResult<'a> {
    pub metadata: SourceMetadata<'a>,
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

impl Source {
    pub fn name<'a>(&'a self) -> &'a OsStr {
        match *self {
            Source::File(ref path) => path.file_name().unwrap(),
            Source::String(ref name, ..) => String::as_ref(name),
        }
    }
}

impl Berg {
    pub fn from_env() -> Berg {
        Berg { root: env::current_dir() }
    }
    pub fn new(root: PathBuf) -> Berg {
        Berg { root: Ok(root) }
    }

    pub fn root(&self) -> &io::Result<PathBuf> {
        &self.root
    }

    pub fn file(&self, path: PathBuf) -> Source {
        Source::File(path)
    }
    pub fn string(&self, name: String, contents: String) -> Source {
        Source::String(name, contents)
    }
    pub fn parse<'a>(&self, source: &'a Source) -> ParseResult<'a> {
        let berg = Berg::from_env();
        parser::parse(source, &berg)
    }
}

impl<'a> ParseResult<'a> {
    pub fn source(&self) -> &'a Source {
        self.metadata.source()
    }
}

impl<'a> fmt::Display for ParseResult<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Source: {:?}", self.source().name())?;
        if self.expressions.len() > 0 {
            write!(f, "  Expressions:")?;
            for ref expression in &self.expressions {
                write!(f, "  - {}", expression)?;
            }
        }
        if self.errors.len() > 0 {
            write!(f, "  Errors:")?;
            for ref error in self.errors.all() {
                error.format(f, &self.metadata)?;
            }
        }
        Ok(())
    }
}

impl<'a> fmt::Display for SyntaxExpression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.expression_type {
            IntegerLiteral => write!(f, "{:?}", self.string)
        }
    }
}

#[derive(Debug)]
pub struct SourceMetadata<'a> {
    source: &'a Source,
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

impl<'a> SourceMetadata<'a> {
    pub fn new(source: &'a Source) -> SourceMetadata<'a> {
        SourceMetadata { source, char_size: 0, line_starts: vec![0] }
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
    pub fn source(&self) -> &'a Source {
        &self.source
    }
    pub fn char_size(&self) -> GraphemeIndex {
        self.char_size
    }
}

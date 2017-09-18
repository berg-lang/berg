use compiler::internals::*;

pub enum CompileError {
    SourceNotFound(io::Error),
    InvalidUtf8(Range<usize>),
    IoOpenError(io::Error),
    IoReadError(usize,io::Error),
}

impl CompileError {
    pub fn code(&self) -> u16 {
        match self {
            // Codes 1-100, and x000 (1000, 2000, ...) are unused. Hard to google.
            // Syntax errors: 101-999
            &SourceNotFound(..) => 101,
            &InvalidUtf8(..) => 102,

            // Type errors: 1001-1999

            // System errors: 9001-9999
            &IoOpenError(..) => 9001,
            &IoReadError(..) => 9002,
        }
    }
    pub fn range(&self, metadata: &SourceMetadata) -> Range<LineColumn> {
        match self {
            &SourceNotFound(_)|&IoOpenError(_) => Range { start: LineColumn::none(), end: LineColumn::none() },
            &InvalidUtf8(ref range) => metadata.range(range),
            &IoReadError(loc, _) => metadata.range(&(loc..loc)),
        }
    }
    pub fn message(&self, source: &Source) -> String {
        match self {
            &SourceNotFound(ref error) => format!("File not found: {:?} (error: {})", source.name(), error),
            &InvalidUtf8(_) => format!("Invalid UTF-8"),
            &IoOpenError(ref error) => format!("I/O error opening {:?}: {}", source.name(), error),
            &IoReadError(_, ref error) => format!("I/O error while reading {:?}: {}", source.name(), error),
        }
    }
}

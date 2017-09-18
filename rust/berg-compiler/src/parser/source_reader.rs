use std::io::*;
use std::fs::File;
use std::ops::Range;

use compile_error::CompileError::*;
use compile_error_reporter::*;
use source::*;
use source_metadata::*;

/// Stream wrapper built for parsing:
/// - reports errors when UTF-8 is bad or I/O fails
/// - sets line starts information for row/column printing information
/// - sets file size
/// - ensures stream is peekable
pub struct SourceReader {
    chars: SourceReaderChars,
    metadata: SourceMetadata,
    index: usize,
    peek_char: Option<char>,
    errors: CompileErrorReporter,
}

enum SourceReaderChars {
    Unreadable,
    File(Chars<BufReader<File>>),
}

impl SourceReader {
    pub fn start(source: &Source, errors: CompileErrorReporter) -> SourceReader {
        let metadata = SourceMetadata::new(0);
        let index = 0;
        let mut chars = SourceReaderChars::open(source, &errors);

        // Read the first character so peek works
        let peek_char = chars.next(index, &errors);

        SourceReader { chars, metadata, index, peek_char, errors }
    }
    pub fn peek(&mut self) -> Option<char> {
        self.peek_char
    }
    pub fn read(&mut self) -> Option<char> {
        if let Some(prev_char) = self.peek_char {
            self.index += 1;
            self.peek_char = self.chars.next(self.index, &self.errors);
            Some(prev_char)
        } else {
            None
        }
    }
}

impl SourceReaderChars {
    fn open(source: &Source, errors: &CompileErrorReporter) -> SourceReaderChars {
        match source {
            &Source::File(ref path) => {
                match File::open(&path) {
                    Ok(file) => {
                        let chars = BufReader::new(file).chars();
                        SourceReaderChars::File(chars)
                    },
                    Err(error) => {
                        let compile_error = match error.kind() {
                            ErrorKind::NotFound => SourceNotFound(error),
                            _ => IoOpenError(error)
                        };
                        errors.report(compile_error);
                        SourceReaderChars::Unreadable
                    },
                }
            },
        }
    }
    // Really really want this inline so we aren't passing these parameters all the time.
    #[inline(always)]
    fn next(&mut self, index: usize, errors: &CompileErrorReporter) -> Option<char> {
        match self {
            &mut SourceReaderChars::File(ref mut chars) => {
                match chars.next() {
                    Some(Ok(ch)) => Some(ch),
                    Some(Err(CharsError::NotUtf8)) => { errors.report(InvalidUtf8(Range { start: index, end: index })); None }
                    Some(Err(CharsError::Other(error))) => { errors.report(IoReadError(index, error)); None }
                    None => None,
                }
            },
            &mut SourceReaderChars::Unreadable => None
        }
    }
}
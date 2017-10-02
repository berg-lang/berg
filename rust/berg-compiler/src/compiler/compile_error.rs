use parser::results::*;
use compiler::source::*;
use std::io;
use std::ops::Range;
use std::path::Path;

pub use compiler::compile_error::ErrorType::*;

#[derive(Debug)]
pub struct CompileError<'c> {
    error_type: ErrorType,
    messages: Vec<CompileErrorMessage<'c>>,
}

impl<'c> CompileError<'c> {
    pub fn new(error_type: ErrorType, messages: Vec<CompileErrorMessage<'c>>) -> Self {
        CompileError { error_type, messages }
    }
    pub fn error_type(&self) -> ErrorType { self.error_type }
    pub fn messages(&'c self) -> &'c Vec<CompileErrorMessage<'c>> { &self.messages }
}

#[derive(Debug)]
pub struct CompileErrorMessage<'c> {
    source: Option<&'c Source>,
    range: Option<Range<ByteIndex>>,
    replacement: Option<String>,
    message: String,
}

impl<'c> CompileErrorMessage<'c> {
    pub fn replacement(source: &'c Source, range: Range<ByteIndex>, replacement: String, message: String) -> Self {
        CompileErrorMessage { message, source: Some(source), range: Some(range), replacement: Some(replacement) }
    }
    pub fn source_range(source: &'c Source, range: Range<ByteIndex>, message: String) -> Self {
        CompileErrorMessage { message, source: Some(source), range: Some(range), replacement: None }
    }
    pub fn source_only(source: &'c Source, message: String) -> Self {
        CompileErrorMessage { message, source: Some(source), range: None, replacement: None }
    }
    pub fn generic(message: String) -> Self {
        CompileErrorMessage { message, source: None, range: None, replacement: None }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ErrorType {
    SourceNotFound = 101,
    InvalidUtf8 = 102,
    UnsupportedCharacters = 103,
    IoOpenError = 9001,
    IoReadError = 9002,
    IoCurrentDirectoryError = 9003,
}

impl ErrorType {
    pub fn code(self) -> u32 { self as u32 }
    pub fn io_generic<'c>(self, source: &'c Source, error: io::Error) -> CompileError<'c> {
        let error_message = match self {
            IoCurrentDirectoryError => format!("I/O error getting current directory: {}", error),
            _ => unreachable!()
        };
        let message = CompileErrorMessage::source_only(source, error_message);
        CompileError::new(self, vec![message])
    }
    pub fn io_read<'c>(self, source: &'c Source, index: ByteIndex, error: io::Error) -> CompileError<'c> {
        let range = Range { start: index, end: index };
        let error_message = match self {
            IoReadError => format!("I/O read error: '{}'", error),
            _ => unreachable!(),
        };
        let message = CompileErrorMessage::source_range(source, range, error_message);
        CompileError::new(self, vec![message])
    }
    pub fn io_open<'c>(self, source: &'c Source, error: io::Error, path: &Path) -> CompileError<'c> {
        let error_message = match self {
            SourceNotFound => format!("Not found: '{:?}' (error: '{}')", path, error),
            IoOpenError => format!("I/O error opening '{:?}': '{}'", path, error),
            _ => unreachable!(),
        };
        let message = CompileErrorMessage::source_only(source, error_message);
        CompileError::new(self, vec![message])
    }
    pub fn invalid<'c, T: AsRef<[u8]>>(self, source: &'c Source, start: ByteIndex, string: T) -> CompileError<'c> {
        let string = string.as_ref();
        let range = Range { start: start, end: start + string.len() };
        let error_message = match self {
            InvalidUtf8 => format!("Invalid UTF-8 bytes: '{}'", string.iter().map(|b| format!("{:02X}", b)).collect::<Vec<String>>().join("")),
            _ => unreachable!(),
        };
        let message = CompileErrorMessage::source_range(source, range, error_message);
        CompileError::new(self, vec![message])
    }
    pub fn at<'c>(self, source: &'c Source, start: ByteIndex, string: &str) -> CompileError<'c> {
        let range = Range { start: start, end: start + string.len() };
        let error_message = match self {
            UnsupportedCharacters => format!("Unsupported characters {:?}", string),
            _ => unreachable!(),
        };
        let message = CompileErrorMessage::source_range(source, range, error_message);
        CompileError::new(self, vec![message])
    }
    // pub fn format(&self, f: &mut fmt::Formatter, source: &Source, char_data: &CharData) -> fmt::Result {
    //     write!(f, "{:?} {:?} - BRGR-{} {}", source.name(), self.range(&char_data), self.code(), self.message(source))
    // }
    // pub fn range(&self, char_data: &CharData) -> Range<LineColumn> {
    //     match *self {
    //         SourceNotFound(..)|IoOpenError(..)|IoCurrentDirectoryError(..) => Range { start: LineColumn::none(), end: LineColumn::none() },
    //         InvalidUtf8(ref range)|UnsupportedCharacters(ref range) => char_data.range(range),
    //         IoReadError(loc, _) => char_data.range(&(loc..loc)),
    //     }
    // }
    // pub fn message(&self, source: &Source) -> String {
    //     match *self {
    //         SourceNotFound(ref error) => format!("File not found: {:?} (error: {})", source.name(), error),
    //         UnsupportedCharacters(_) => format!("Unsupported characters"),
    //         InvalidUtf8(_) => format!("Invalid UTF-8"),
    //         IoOpenError(ref error) => format!("I/O error opening {:?}: {}", source.name(), error),
    //         IoReadError(_, ref error) => format!("I/O error while reading {:?}: {}", source.name(), error),
    //         IoCurrentDirectoryError(ref path, _, ref message) => format!("I/O error getting current directory to read relative path {:?}: {}", path, message),
    //     }
    // }
}

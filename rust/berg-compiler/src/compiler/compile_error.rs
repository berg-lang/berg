use public::*;
use std::io;
use std::ops::Range;
use std::path::Path;

#[derive(Debug)]
pub struct CompileError {
    error_type: CompileErrorType,
    messages: Vec<CompileErrorMessage>,
}

impl CompileError {
    pub fn new(error_type: CompileErrorType, messages: Vec<CompileErrorMessage>) -> Self {
        CompileError { error_type, messages }
    }
    pub fn error_type(&self) -> CompileErrorType { self.error_type }
    pub fn messages(&self) -> &Vec<CompileErrorMessage> { &self.messages }
}

#[derive(Debug)]
pub struct CompileErrorMessage {
    source: Option<SourceIndex>,
    range: Option<Range<ByteIndex>>,
    replacement: Option<String>,
    message: String,
}

impl CompileErrorMessage {
    pub fn replacement(source: SourceIndex, range: Range<ByteIndex>, replacement: String, message: String) -> Self {
        CompileErrorMessage { message, source: Some(source), range: Some(range), replacement: Some(replacement) }
    }
    pub fn source_range(source: SourceIndex, range: Range<ByteIndex>, message: String) -> Self {
        CompileErrorMessage { message, source: Some(source), range: Some(range), replacement: None }
    }
    pub fn source_only(source: SourceIndex, message: String) -> Self {
        CompileErrorMessage { message, source: Some(source), range: None, replacement: None }
    }
    pub fn generic(message: String) -> Self {
        CompileErrorMessage { message, source: None, range: None, replacement: None }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum CompileErrorType {
    SourceNotFound = 101,
    InvalidUtf8 = 102,
    UnsupportedCharacters = 103,
    IoOpenError = 9001,
    IoReadError = 9002,
    IoCurrentDirectoryError = 9003,
}

impl CompileErrorType {
    pub fn code(self) -> u32 { self as u32 }
    pub fn io_generic(self, source: SourceIndex, error: &io::Error) -> CompileError {
        let error_message = match self {
            IoCurrentDirectoryError => format!("I/O error getting current directory: {}", error),
            _ => unreachable!()
        };
        let message = CompileErrorMessage::source_only(source, error_message);
        CompileError::new(self, vec![message])
    }
    pub fn io_read(self, source: SourceIndex, index: ByteIndex, error: io::Error) -> CompileError {
        let range = Range { start: index, end: index };
        let error_message = match self {
            IoReadError => format!("I/O read error: '{}'", error),
            _ => unreachable!(),
        };
        let message = CompileErrorMessage::source_range(source, range, error_message);
        CompileError::new(self, vec![message])
    }
    pub fn io_open(self, source: SourceIndex, error: io::Error, path: &Path) -> CompileError {
        let error_message = match self {
            SourceNotFound => format!("Not found: '{:?}' (error: '{}')", path, error),
            IoOpenError => format!("I/O error opening '{:?}': '{}'", path, error),
            _ => unreachable!(),
        };
        let message = CompileErrorMessage::source_only(source, error_message);
        CompileError::new(self, vec![message])
    }
    pub fn invalid<T: AsRef<[u8]>>(self, source: SourceIndex, start: ByteIndex, string: T) -> CompileError {
        let string = string.as_ref();
        let range = Range { start: start, end: start + string.len() };
        let error_message = match self {
            InvalidUtf8 => format!("Invalid UTF-8 bytes: '{}'", string.iter().map(|b| format!("{:02X}", b)).collect::<Vec<String>>().join("")),
            _ => unreachable!(),
        };
        let message = CompileErrorMessage::source_range(source, range, error_message);
        CompileError::new(self, vec![message])
    }
    pub fn at(self, source: SourceIndex, start: ByteIndex, string: &str) -> CompileError {
        let range = Range { start: start, end: start + string.len() };
        let error_message = match self {
            UnsupportedCharacters => format!("Unsupported characters {:?}", string),
            _ => unreachable!(),
        };
        let message = CompileErrorMessage::source_range(source, range, error_message);
        CompileError::new(self, vec![message])
    }
}

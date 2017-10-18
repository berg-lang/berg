use public::*;
use std::io;
use std::ops::Range;
use std::path::Path;

#[derive(Clone, Debug)]
pub struct CompileError {
    error_type: CompileErrorType,
    messages: Vec<CompileErrorMessage>,
}

impl CompileError {
    pub fn new(error_type: CompileErrorType, messages: Vec<CompileErrorMessage>) -> Self {
        CompileError {
            error_type,
            messages,
        }
    }
    pub fn error_type(&self) -> CompileErrorType {
        self.error_type
    }
    pub fn messages(&self) -> &Vec<CompileErrorMessage> {
        &self.messages
    }
}

#[derive(Clone, Debug)]
pub struct CompileErrorMessage {
    pub source: Option<SourceIndex>,
    pub range: Option<Range<ByteIndex>>,
    pub replacement: Option<String>,
    pub message: String,
}

impl CompileErrorMessage {
    pub fn new_replacement(
        source: SourceIndex,
        range: Range<ByteIndex>,
        replacement: String,
        message: String,
    ) -> Self {
        CompileErrorMessage {
            message,
            source: Some(source),
            range: Some(range),
            replacement: Some(replacement),
        }
    }
    pub fn source_range(source: SourceIndex, range: Range<ByteIndex>, message: String) -> Self {
        CompileErrorMessage {
            message,
            source: Some(source),
            range: Some(range),
            replacement: None,
        }
    }
    pub fn source_only(source: SourceIndex, message: String) -> Self {
        CompileErrorMessage {
            message,
            source: Some(source),
            range: None,
            replacement: None,
        }
    }
    pub fn generic(message: String) -> Self {
        CompileErrorMessage {
            message,
            source: None,
            range: None,
            replacement: None,
        }
    }

    pub fn source(&self) -> Option<SourceIndex> {
        self.source
    }
    pub fn range(&self) -> &Option<Range<ByteIndex>> {
        &self.range
    }
    pub fn replacement(&self) -> &Option<String> {
        &self.replacement
    }
    pub fn message(&self) -> &String {
        &self.message
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CompileErrorType {
    // Compile errors related to I/O and format
    SourceNotFound = 101,
    IoOpenError = 102,
    IoReadError = 103,
    IoCurrentDirectoryError = 104,
    SourceTooLarge = 105,
    TooManySources = 106,
    InvalidUtf8 = 107,
    UnsupportedCharacters = 108,

    // Compile errors related to structure
    MissingBothOperands = 201,
    MissingLeftOperand = 202,
    MissingRightOperand = 203,
    UnrecognizedOperator = 204,
    OperatorsOutOfPrecedenceOrder = 205,

    // Compile errors related to type
    DivideByZero = 1001, // Errors that are most likely transient
}

impl CompileErrorType {
    pub fn code(self) -> u32 {
        self as u32
    }
    pub fn io_generic(self, source: SourceIndex, error: &io::Error) -> CompileError {
        let error_message = match self {
            IoCurrentDirectoryError => format!("I/O error getting current directory: {}", error),
            _ => unreachable!(),
        };
        let message = CompileErrorMessage::source_only(source, error_message);
        CompileError::new(self, vec![message])
    }
    pub fn io_read(self, source: SourceIndex, index: ByteIndex, error: &io::Error) -> CompileError {
        let range = Range {
            start: index,
            end: index,
        };
        let error_message = match self {
            IoReadError => format!("I/O read error: '{}'", error),
            _ => unreachable!(),
        };
        let message = CompileErrorMessage::source_range(source, range, error_message);
        CompileError::new(self, vec![message])
    }
    pub fn io_open(self, source: SourceIndex, error: &io::Error, path: &Path) -> CompileError {
        let error_message = match self {
            SourceNotFound => format!("Not found: '{:?}' (error: '{}')", path, error),
            IoOpenError => format!("I/O error opening '{:?}': '{}'", path, error),
            _ => unreachable!(),
        };
        let message = CompileErrorMessage::source_only(source, error_message);
        CompileError::new(self, vec![message])
    }
    pub fn invalid_bytes<T: AsRef<[u8]>>(
        self,
        source: SourceIndex,
        start: ByteIndex,
        string: T,
    ) -> CompileError {
        let string = string.as_ref();
        let len = string.len() as ByteIndex;
        let range = Range {
            start: start,
            end: start + len,
        };
        let error_message = match self {
            InvalidUtf8 => format!(
                "Invalid UTF-8 bytes: '{}'",
                string
                    .iter()
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<String>>()
                    .join("")
            ),
            _ => unreachable!(),
        };
        let message = CompileErrorMessage::source_range(source, range, error_message);
        CompileError::new(self, vec![message])
    }
    pub fn at(self, source: SourceIndex, start: ByteIndex, string: &str) -> CompileError {
        let range = Range {
            start: start,
            end: start + (string.len() as u32),
        };
        let error_message = match self {
            UnsupportedCharacters => format!("Unsupported characters {:?}", string),
            UnrecognizedOperator => format!("Unrecognized operator {:?}", string),
            MissingBothOperands => format!(
                "Operator {:?} has no value on either side to operate on!",
                string
            ),
            MissingLeftOperand => format!(
                "Operator {:?} has no value on the left hand side to operate on!",
                string
            ),
            MissingRightOperand => format!(
                "Operator {:?} has no value on the right hand side to operate on!",
                string
            ),
            OperatorsOutOfPrecedenceOrder => format!(
                "Operator {:?} has higher precedence than the previous operator! Automatic precedence resolution is not supported. Perhaps you should place this operator in parentheses?",
                string
            ),
            DivideByZero => format!(
                "Division by zero is illegal. Perhaps you meant a different number on the right hand side of the '{:?}'?",
                string
            ),
            _ => unreachable!(),
        };
        let message = CompileErrorMessage::source_range(source, range, error_message);
        CompileError::new(self, vec![message])
    }
    pub fn source_only(self, source: SourceIndex) -> CompileError {
        let error_message = match self {
            SourceTooLarge => {
                "SourceSpec code too large: source files greater than 4GB are unsupported."
                    .to_string()
            }
            _ => unreachable!(),
        };
        let message = CompileErrorMessage::source_only(source, error_message);
        CompileError::new(self, vec![message])
    }
    pub fn generic(self) -> CompileError {
        let error_message = match self {
            TooManySources => format!("Too many source files opened! Max is {}.", u32::max_value()),
            _ => unreachable!(),
        };
        let message = CompileErrorMessage::generic(error_message);
        CompileError::new(self, vec![message])
    }
}

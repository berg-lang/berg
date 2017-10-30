use std::ops::Deref;
use public::*;
use std::io;
use std::ops::Range;
use std::path::Path;
use std::str;

#[derive(Clone, Debug)]
pub struct CompileError {
    error_type: CompileErrorType,
    messages: Vec<CompileErrorMessage>,
}

#[derive(Clone, Debug)]
pub struct CompileErrorMessage {
    pub source: Option<SourceIndex>,
    pub range: Option<Range<ByteIndex>>,
    pub replacement: Option<String>,
    pub message: String,
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
    MissingOperandsBetween = 201,
    MissingRightOperand = 202,
    MissingLeftOperand = 203,    
    UnrecognizedOperator = 204,
    OperatorsOutOfPrecedenceOrder = 205,
    OpenWithoutClose = 206,
    CloseWithoutOpen = 207,

    // Compile errors related to type
    DivideByZero = 1001,
    BadTypeLeftOperand = 1002,
    BadTypeRightOperand = 1003,
    BadTypeBothOperands = 1004,
}

#[derive(Debug)]
pub struct CompileErrors {
    errors: Vec<CompileError>,
}

impl Default for CompileErrors {
    fn default() -> Self { CompileErrors { errors: Default::default() } }
}

impl CompileErrors {
    pub(crate) fn report(&mut self, error: CompileError) {
        self.errors.push(error)
    }
    pub(crate) fn report_generic(&mut self, error_type: CompileErrorType) {
        self.report(error_type.generic())
    }
    pub(crate) fn extend(&mut self, errors: CompileErrors) {
        self.errors.extend(errors.errors)
    }
}

impl Deref for CompileErrors {
    type Target = Vec<CompileError>;
    fn deref(&self) -> &Vec<CompileError> { &self.errors }
}

impl Deref for SourceCompileErrors {
    type Target = Vec<CompileError>;
    fn deref(&self) -> &Vec<CompileError> { &self.errors }
}

pub(crate) struct SourceCompileErrors {
    errors: CompileErrors,
    source: SourceIndex,
}

impl SourceCompileErrors {
    pub(crate) fn new(source: SourceIndex) -> Self {
        let errors = Default::default();
        SourceCompileErrors { errors, source }
    }
    pub(crate) fn report(&mut self, error: CompileError) {
        self.errors.report(error)
    }
    pub(crate) fn report_at(&mut self, error_type: CompileErrorType, range: Range<ByteIndex>, string: &str) {
        let error = error_type.at(self.source, range, string);
        self.report(error)
    }
    pub(crate) unsafe fn report_at_utf8_unchecked(&mut self, error_type: CompileErrorType, range: Range<ByteIndex>, buffer: &[u8]) {
        let bytes = &buffer[usize::from(range.start)..usize::from(range.end)];
        let string = str::from_utf8_unchecked(bytes);
        self.report_at(error_type, range, string)
    }
    pub(crate) fn report_invalid_utf8(&mut self, range: Range<ByteIndex>, buffer: &[u8]) {
        let bytes = &buffer[usize::from(range.start)..usize::from(range.end)];
        let error = CompileErrorType::InvalidUtf8.invalid_bytes(self.source, range, bytes);
        self.report(error)
    }
    pub(crate) fn report_io_read(&mut self, index: ByteIndex, error: &io::Error) {
        let error = CompileErrorType::IoReadError.io_read(self.source, index, error);
        self.report(error)
    }
    pub(crate) fn report_io_open(&mut self, error_type: CompileErrorType, error: &io::Error, path: &Path) {
        let error = error_type.io_open(self.source, error, path);
        self.report(error)
    }
    pub(crate) fn report_io_source(&mut self, error_type: CompileErrorType, error: &io::Error) {
        let error = error_type.io_source(self.source, error);
        self.report(error)
    }
    pub(crate) fn close(self) -> CompileErrors {
        self.errors
    }
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

impl CompileErrorType {
    pub fn code(self) -> u32 {
        self as u32
    }
    pub fn io_source(self, source: SourceIndex, error: &io::Error) -> CompileError {
        use compiler::compile_errors::CompileErrorType::*;
        let error_message = match self {
            IoCurrentDirectoryError => format!("I/O error getting current directory: {}", error),
            _ => unreachable!(),
        };
        let message = CompileErrorMessage::source_only(source, error_message);
        CompileError::new(self, vec![message])
    }
    pub fn io_read(self, source: SourceIndex, index: ByteIndex, error: &io::Error) -> CompileError {
        use compiler::compile_errors::CompileErrorType::*;
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
        use compiler::compile_errors::CompileErrorType::*;
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
        range: Range<ByteIndex>,
        string: T,
    ) -> CompileError {
        use compiler::compile_errors::CompileErrorType::*;
        let string = string.as_ref();
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
    pub fn at(self, source: SourceIndex, range: Range<ByteIndex>, string: &str) -> CompileError {
        use compiler::compile_errors::CompileErrorType::*;
        let error_message = match self {
            UnsupportedCharacters => format!("Unsupported characters {:?}", string),
            UnrecognizedOperator => format!("Unrecognized operator {:?}", string),
            MissingOperandsBetween => format!(
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
            BadTypeLeftOperand => format!(
                "The value on the left side of the operator {:?} is not a number!",
                string
            ),
            BadTypeRightOperand => format!(
                "The value on the right side of the operator {:?} is not a number!",
                string
            ),
            BadTypeBothOperands => format!(
                "The values on either side of the operator {:?} are not numbers!",
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
            OpenWithoutClose => format!(
                "Open '{:?}' found without a matching close.",
                string
            ),
            CloseWithoutOpen => format!(
                "Closing '{:?}' found without a matching open.",
                string
            ),
            _ => unreachable!(),
        };
        let message = CompileErrorMessage::source_range(source, range, error_message);
        CompileError::new(self, vec![message])
    }
    pub fn source_only(self, source: SourceIndex) -> CompileError {
        use compiler::compile_errors::CompileErrorType::*;
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
        use compiler::compile_errors::CompileErrorType::*;
        let error_message = match self {
            TooManySources => format!("Too many source files opened! Max is {}.", u32::max_value()),
            _ => unreachable!(),
        };
        let message = CompileErrorMessage::generic(error_message);
        CompileError::new(self, vec![message])
    }
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

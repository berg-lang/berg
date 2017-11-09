use std::path::PathBuf;
use compiler::Compiler;
use compiler::source_data::{ByteRange,SourceIndex};
use checker::checker_type::Type;
use std::fmt;
use std::str;

pub trait CompileError: fmt::Debug {
    fn code(&self) -> u32;
    fn message(&self, compiler: &Compiler) -> CompileErrorMessage;
}

#[derive(Debug,Clone)]
pub struct CompileErrorMessage {
    pub location: CompileErrorLocation,
    pub message: String,
}
#[derive(Debug,Clone)]
pub enum CompileErrorLocation {
    Generic,
    SourceOnly { source: SourceIndex },
    SourceRange { source: SourceIndex, range: ByteRange },
}

macro_rules! compile_errors {
    ($(pub struct $name:ident $fields:tt ($code:expr) = $message_type:ident $message:tt;)*) => {
        $(compile_errors! { @single pub struct $name $fields ($code) = $message_type $message; })*
    };
    (@single pub struct $name:ident $fields:tt ($code:expr) = $message_type:ident $message:tt;) => {
        compile_errors! { @define_struct $name, $fields, $message_type }
        compile_errors! { @impl_struct $name, $code, $fields, $message_type, $message }
    };
    (@define_struct $name:ident, { $(pub $field:tt: $field_type:ty),* }, string_generic) => {
        #[derive(Debug,Clone)]
        pub(crate) struct $name { $(pub $field: $field_type,)* }
    };
    (@define_struct $name:ident, { $(pub $field:tt: $field_type:ty),* }, format_generic) => {
        #[derive(Debug,Clone)]
        pub(crate) struct $name { $(pub $field: $field_type,)* }
    };
    (@define_struct $name:ident, { $(pub $field:tt: $field_type:ty),* }, $message_type:ident) => {
        #[derive(Debug,Clone)]
        pub(crate) struct $name { pub(crate) source: SourceIndex, $(pub $field: $field_type,)* }
    };
    (@impl_struct $name:ident, $code:expr, $fields:tt, $message_type:ident, $message:tt) => {
        impl $name {
            pub(crate) const CODE: u32 = $code;
        }
        impl CompileError for $name {
            fn code(&self) -> u32 { $name::CODE }
            compile_errors! { @message $message_type, $message, $fields }
        }
    };
    (@message string, ($range:tt, $message:tt), $fields:tt) => (
        fn message(&self, _: &Compiler) -> CompileErrorMessage {
            CompileErrorMessage {
                message: $message.to_string(),
                location: CompileErrorLocation::SourceRange { source: self.source, range: self.$range.clone() }
            }
        }
    );
    (@message string_source, ($message:tt), $fields:tt) => (
        fn message(&self, _: &Compiler) -> CompileErrorMessage {
            CompileErrorMessage {
                message: $message.to_string(),
                location: CompileErrorLocation::SourceOnly { source: self.source }
            }
        }
    );
    (@message string_generic, ($message:tt), $fields:tt) => (
        fn message(&self, _: &Compiler) -> CompileErrorMessage {
            CompileErrorMessage {
                message: $message.to_string(),
                location: CompileErrorLocation::Generic { }
            }
        }
    );
    (@message format, ($range:tt, $message:tt), { $(pub $field:tt: $field_type:tt),* }) => (
        fn message(&self, _compiler: &Compiler) -> CompileErrorMessage {
            CompileErrorMessage {
                message: format!($message, $($field = compile_errors!(@field_value _compiler, self, (self.$field), $field_type)),*),
                location: CompileErrorLocation::SourceRange { source: self.source, range: self.$range.clone() },
            }
        }
    );
    (@message format_source, ($message:tt), { $(pub $field:tt: $field_type:tt),* }) => (
        fn message(&self, _compiler: &Compiler) -> CompileErrorMessage {
            CompileErrorMessage {
                message: format!($message, $($field = compile_errors!(@field_value _compiler, self, (self.$field), $field_type)),*),
                location: CompileErrorLocation::SourceOnly { source: self.source },
            }
        }
    );
    (@message format_generic, ($message:tt), { $(pub $field:tt: $field_type:tt),* }) => (
        fn message(&self, compiler: &Compiler) -> CompileErrorMessage {
            CompileErrorMessage {
                message: format!($message, $($field = compile_errors!(@field_value compiler, self, (self.$field), $field_type)),*),
                location: CompileErrorLocation::Generic,
            }
        }
    );
    (@field_value $compiler:ident, $self:ident, $value:tt, ByteRange) => (source_string($compiler, $self.source, &$value));
    (@field_value $compiler:ident, $self:ident, $value:tt, $type:tt) => ($value);
}

fn source_string(compiler: &Compiler, source: SourceIndex, range: &ByteRange) -> String {
    let buffer = compiler.with_source(source, |source_data| source_data.source_spec().open(compiler, source));
    if range.end <= buffer.len() {
        if let Ok(string) = str::from_utf8(&buffer[range]) {
            return string.to_string();
        }
    }
    String::from("ERROR: source may have changed since compiling, source range is no longer valid UTF-8")
}

// compile_errors! {
//     pub(crate) struct SourceNotFound { pub path: PathBuf, pub io_error: io::Error } (101) = format("I/O error getting current directory to expand {path:?}: {io_error}")
// }
compile_errors! {
    // Compile errors independent of parsing
    pub struct SourceNotFound          { pub path: PathBuf, pub io_error_string: String } (101) = format_source("I/O error getting current directory to expand {path:?}: {io_error_string}");
    pub struct IoOpenError             { pub path: PathBuf, pub io_error_string: String } (102) = format_source("I/O error opening {path:?}: {io_error_string}");
    pub struct IoReadError             { pub range: ByteRange, pub path: PathBuf, pub io_error_string: String } (103) = format_source("I/O error at {range} reading {path:?}: {io_error_string}");
    pub struct IoCurrentDirectoryError { pub path: PathBuf, pub io_error_string: String } (104) = format_source("I/O error getting current directory to determine path of {path:?}: {io_error_string}");
    pub struct SourceTooLarge          { pub size: usize } (105) = string_source("SourceSpec code too large: source files greater than 4GB are unsupported.");
    pub struct TooManySources          { pub num_sources: usize } (106) = string_generic("Too many source files opened!");

    // Compile errors related to format (tokenizer)
    pub struct InvalidUtf8             { pub bytes: ByteRange } (201) = string(bytes, "Invalid UTF-8! Perhaps this isn't a Berg source file?");
    pub struct UnsupportedCharacters   { pub characters: ByteRange } (202) = string(characters, "Invalid Unicode characters");

    // Compile errors related to structure (parser)
    pub struct MissingRightOperand     { pub operator: ByteRange } (301) = format(operator, "Operator {operator} has no value on the right hand side to operate on!");
    pub struct MissingLeftOperand      { pub operator: ByteRange } (302) = format(operator, "Operator {operator} has no value on the left hand side to operate on!");
    pub struct OpenWithoutClose        { pub open_range: ByteRange, pub close: String } (303) = format(open_range, "Open '{open_range}' found without a matching close '{close}'.");
    pub struct CloseWithoutOpen        { pub close_range: ByteRange, pub open: String } (304) = format(close_range, "Closing '{close_range}' found without a matching '{open}'.");

    // Compile errors related to type (checker)
    pub struct UnrecognizedOperator    { pub operator: ByteRange } (1001) = format(operator, "Unrecognized operator {operator}");
    pub struct DivideByZero            { pub divide: ByteRange } (1002) = format(divide, "Division by zero is illegal. Perhaps you meant a different number on the right hand side of the '{divide}'?");
    pub struct BadTypeLeftOperand      { pub operator: ByteRange, pub left: Type } (1003) = format(operator, "The value on the left side of '{operator}' is not a number! It is {left:?} instead.");
    pub struct BadTypeRightOperand     { pub operator: ByteRange, pub right: Type } (1004) = format(operator, "The value on the right side of '{operator}' is not a number! It is {right:?} instead.");
}

use ast::token::Fixity;
use compiler::Compiler;
use compiler::source_data::{ByteRange,SourceIndex};
use checker::checker_type::Type;
use std::fmt;
use std::path::PathBuf;

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
    pub struct LeftSideOfAssignmentMustBeIdentifier { pub left: ByteRange, pub operator: ByteRange } (305) = format(left, "The assignment operator '{operator}' must have a property declaration or name on the left side (like \":foo {operator} ...\" or \"foo {operator} ...\"): the left side is currently {left}.");
    pub struct LeftSideOfIncrementOrDecrementOperandMustBeIdentifier { pub left: ByteRange, pub operator: ByteRange } (305) = format(left, "The assignment operator '{operator}' must have a property name on the left side (like \"foo{operator}\"): the left side is currently '{left}'.");
    pub struct RightSideOfIncrementOrDecrementOperandMustBeIdentifier { pub right: ByteRange, pub operator: ByteRange } (305) = format(right, "The assignment operator '{operator}' must have a property name on the right side (like \"{operator}foo ...\" or \"{operator}foo ...\"): the right side is currently {right}.");

    // Compile errors related to type (checker)
    pub struct UnrecognizedOperator    { pub operator: ByteRange, pub fixity: Fixity } (1001) = format(operator, "Unrecognized {fixity} operator {operator}");
    pub struct DivideByZero            { pub divide: ByteRange } (1002) = format(divide, "Division by zero is illegal. Perhaps you meant a different number on the right hand side of the '{divide}'?");
    pub struct BadTypeLeftOperand      { pub operator: ByteRange, pub left: Type } (1003) = format(operator, "The value on the left side of '{operator}' is not a number! It is {left:?} instead.");
    pub struct BadTypeRightOperand     { pub operator: ByteRange, pub right: Type } (1004) = format(operator, "The value on the right side of '{operator}' is not a number! It is {right:?} instead.");
    pub struct NoSuchProperty          { pub reference: ByteRange } (1005) = format(reference, "No such property: '{reference}'");
    pub struct PropertyNotSet          { pub reference: ByteRange } (1006) = format(reference, "Property '{reference}' was declared, but never set to a value!");
    pub struct IdentifierStartsWithNumber { pub identifier: ByteRange } (1005) = format(identifier, "Properties cannot start with a number: '{identifier}'");
}

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

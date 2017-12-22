use source::line_column::LineColumnRange;
use ast::expression::SourceToken;
use ast::AstIndex;
use ast::expression::SourceExpression;
use ast::expression::Expression;
use util::display_arg::DisplayContext;
use std::fmt::Formatter;
use interpreter::value::Value;
use ast::token::Fixity;
use ast::expression::OperandPosition;
use ast::expression::OperandType;
use compiler::Compiler;
use source::SourceIndex;
use source::parse_result::ByteRange;
use std::fmt;
use std::path::PathBuf;

compile_errors! {
    // Compile errors independent of parsing
    pub struct SourceNotFound          { pub source: SourceIndex, pub path: PathBuf, pub io_error_string: String } (101) = source(source):format("I/O error getting current directory path {path} ({source}): {io_error_string}");
    pub struct IoOpenError             { pub source: SourceIndex, pub path: PathBuf, pub io_error_string: String } (102) = source(source):format("I/O error opening {path} ({source}): {io_error_string}");
    pub struct IoReadError             { pub source: SourceIndex, pub path: PathBuf, pub io_error_string: String } (103) = source(source):format("I/O error reading {path} ({source}): {io_error_string}");
    pub struct IoCurrentDirectoryError { pub source: SourceIndex, pub path: PathBuf, pub io_error_string: String } (104) = source(source):format("I/O error getting current directory to determine path of {path} ({source}): {io_error_string}");
    pub struct SourceTooLarge          { pub source: SourceIndex, pub size: usize } (105) = source(source):string("Source file {source} too large: source files greater than 4GB are unsupported.");
    pub struct TooManySources          { pub num_sources: usize } (106) = generic():string("Too many source files opened!");

    // Compile errors related to format (tokenizer)
    pub struct InvalidUtf8             { pub bytes: SourceRange } (201) = range(bytes):string("Invalid UTF-8! Perhaps this isn't a Berg source file?");
    pub struct UnsupportedCharacters   { pub characters: SourceRange } (202) = range(characters):string("Invalid Unicode characters");

    // Compile errors related to structure (parser)
    pub struct MissingOperand          { pub operator: SourceRange, pub position: OperandPosition } (301) = range(operator):format("Operator {operator} has no value on {position} to operate on!");
    pub struct OpenWithoutClose        { pub open: SourceRange, pub close_string: &'static str } (303) = range(open):format("Open '{open}' found without a matching close '{close_string}'.");
    pub struct CloseWithoutOpen        { pub close: SourceRange, pub open_string: &'static str } (304) = range(close):format("Closing '{close}' found without a matching open '{open_string}'.");
    pub struct LeftSideOfAssignmentMustBeIdentifier { pub operator: SourceRange, pub left: SourceRange } (305) = range(left):format("The assignment operator '{operator}' must have a field declaration or name on the left side (like \":foo {operator} ...\" or \"foo {operator} ...\": the left side is currently {left}.");
    pub struct LeftSideOfIncrementOrDecrementMustBeIdentifier { pub operator: SourceRange, pub left: SourceRange } (305) = range(left):format("The assignment operator '{operator}' must have a field name on the left side (like \"foo{operator}\": the left side is currently '{left}'.");
    pub struct RightSideOfIncrementOrDecrementMustBeIdentifier { pub operator: SourceRange, pub right: SourceRange } (305) = range(right):format("The assignment operator '{operator}' must have a field name on the right side (like \"{operator}foo ...\" or \"{operator}foo ...\": the right side is currently {right}.");

    // Compile errors related to type (checker)
    pub struct UnrecognizedOperator    { pub operator: SourceRange, pub fixity: Fixity } (1001) = range(operator):format("Unrecognized {fixity} operator {operator}");
    pub struct DivideByZero            { pub divide: SourceRange } (1002) = range(divide):format("Division by zero is illegal. Perhaps you meant a different number on the right hand side of the '{divide}'?");
    pub struct BadType                 { pub operator: SourceRange, pub operand: SourceRange, pub actual_value: Value, pub expected_type: OperandType, pub position: OperandPosition } (1003) = range(operator):format("The value of '{operand}' is {actual_value}, but {position} '{operator}' must be an {expected_type}!");
    pub struct NoSuchField             { pub reference: SourceRange } (1005) = range(reference):format("No such field: '{reference}'");
    pub struct NoSuchFieldYet          { pub reference: SourceRange } (1005) = range(reference):format("No such field *yet* in block: '{reference}'. May appear later. NOTE: when we have a full typechecker, this will become impossible / a compile time error with more information.");
    pub struct FieldNotSet             { pub reference: SourceRange } (1006) = range(reference):format("Field '{reference}' was declared, but never set to a value!");
    pub struct IdentifierStartsWithNumber { pub identifier: SourceRange } (1007) = range(identifier):format("Properties cannot start with a number: '{identifier}'");
    pub struct NoMoreOutput            { pub reference: SourceRange } (1008) = range(reference):format("Block referenced at '{reference}' has already been exhausted! No more output to give. NOTE: when we have a full typechecker, this will become impossible / a compile time error with more information.");
    pub struct CircularDependency      { pub expression: SourceRange } (1008) = range(expression):format("Circular dependency at '{expression}'!");
}

pub trait CompileError: fmt::Debug {
    fn location(&self) -> CompileErrorLocation;
    fn code(&self) -> CompileErrorCode;
    fn message<'c>(&self, compiler: &'c Compiler) -> CompileErrorMessage;
    fn box_clone(&self) -> Box<CompileError>;
}

impl Clone for Box<CompileError> {
    fn clone(&self) -> Box<CompileError> {
        self.box_clone()
    }
}

impl PartialEq for CompileError {
    fn eq(&self, other: &CompileError) -> bool {
        self.code() == other.code()
    }
}
impl<'c> DisplayContext<&'c Compiler> for CompileError {
    fn fmt(&self, f: &mut Formatter, compiler: &&'c Compiler) -> fmt::Result {
        let message = self.message(compiler);
        match message.location {
            CompileErrorLocation::Generic|CompileErrorLocation::SourceOnly(_) => write!(f, "{}", message.message),
            CompileErrorLocation::SourceRange(range) => {
                write!(f, "{}: {}", range.line_column_range(compiler), message.message)
            },
        }
    }
}

#[derive(Debug,Clone,PartialEq)]
pub struct CompileErrorMessage {
    pub location: CompileErrorLocation,
    pub message: String,
}

#[derive(Debug,Clone,PartialEq)]
pub enum CompileErrorLocation {
    Generic,
    SourceOnly(SourceIndex),
    SourceRange(SourceRange)
}

#[derive(Debug,Clone,PartialEq)]
pub enum SourceRange {
    Token { source: SourceIndex, index: AstIndex },
    Expression { source: SourceIndex, expression: Expression },
}

impl SourceRange {
    fn with<T,F: Fn(SourceExpression)->T>(&self, compiler: &Compiler, f: F) -> T {
        match *self {
            SourceRange::Expression{source,expression} => {
                let source = compiler.source(source);
                let borrowed = source.borrow();
                let parse_result = borrowed.parse_result();
                f(SourceExpression { parse_result: &parse_result, expression })
            },
            _ => unreachable!(),
        }
    }
    fn with_token<T,F: Fn(SourceToken)->T>(&self, compiler: &Compiler, f: F) -> T {
        match *self {
            SourceRange::Token{source,index} => {
                let source = compiler.source(source);
                let borrowed = source.borrow();
                let parse_result = borrowed.parse_result();
                f(SourceToken { parse_result: &parse_result, index })
            },
            _ => unreachable!(),
        }
    }
    fn source(&self) -> SourceIndex {
        match *self {
            SourceRange::Token{source,..}|SourceRange::Expression{source,..} => source,
        }
    }
    pub fn range(&self, compiler: &Compiler) -> ByteRange {
        match *self {
            SourceRange::Token{..} => self.with_token(compiler, |token| token.range()),
            SourceRange::Expression{..} => self.with(compiler, |expression| expression.range()),
        }
    }
    fn as_string(&self, compiler: &Compiler) -> String {
        let range = self.range(compiler);
        source_string(compiler, self.source(), &range)
    }
    fn line_column_range(&self, compiler: &Compiler) -> LineColumnRange {
        match *self {
            SourceRange::Token{..} => self.with_token(compiler, |token| { token.line_column_range() }),
            SourceRange::Expression{..} => self.with(compiler, |expression| { expression.line_column_range() }),
        }
    }
}

impl<'e> From<SourceExpression<'e>> for SourceRange {
    fn from(expression: SourceExpression<'e>) -> Self {
        SourceRange::Expression { source: expression.parse_result.index, expression: expression.expression }
    }
}
impl<'e> From<SourceToken<'e>> for SourceRange {
    fn from(token: SourceToken<'e>) -> Self {
        SourceRange::Token { source: token.parse_result.index, index: token.index }
    }
}

impl<'c> DisplayContext<&'c Compiler> for SourceRange {
    fn fmt(&self, f: &mut Formatter, compiler: &&'c Compiler) -> fmt::Result {
        write!(f, "{}", self.as_string(compiler))
    }
}

macro_rules! impl_display_context {
    ($context:ty: $($type:ty),*) => {
        $(
            impl<'c> DisplayContext<$context> for $type {
                fn fmt(&self, f: &mut Formatter, _compiler: &&'c Compiler) -> fmt::Result {
                    let x: &Display = self;
                    x.fmt(f)
                }
            }
        )*
    }
}

macro_rules! impl_display_context_with_debug {
    ($context:ty: $($type:ty),*) => {
        $(
            impl<'c> DisplayContext<$context> for $type {
                fn fmt(&self, f: &mut Formatter, _compiler: &&'c Compiler) -> fmt::Result {
                    let x: &Debug = self;
                    x.fmt(f)
                }
            }
        )*
    }
}

impl_display_context! { &'c Compiler: &'c str, String, SourceIndex, Fixity, OperandPosition, OperandType }

impl_display_context_with_debug! { &'c Compiler: PathBuf }

pub fn source_string(compiler: &Compiler, index: SourceIndex, range: &ByteRange) -> String {
    use std::str;
    let source = compiler.source(index);
    let source = source.borrow();
    let buffer = source.reopen(compiler);
    if range.end <= buffer.len() {
        if let Ok(string) = str::from_utf8(&buffer[range]) {
            return string.to_string();
        }
    }
    String::from("ERROR: source may have changed since compiling, source range is no longer valid UTF-8")
}

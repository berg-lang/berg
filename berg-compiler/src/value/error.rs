use crate::eval::BlockRef;
use crate::syntax::{
    AstRef, ByteRange, Expression, FieldIndex, Fixity, IdentifierIndex, LineColumnRange, LiteralIndex,
    OperandPosition, RawLiteralIndex,
};
use crate::value::*;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Error<'a> {
    pub error: BergError<'a>,
    pub stack: Vec<(AstRef<'a>, Expression)>,
}

///
/// This is the error type that is sent throughout the code.
///
/// A *Stacked* value means the error has already been attached to at least one
/// stack trace.
///
#[derive(Debug, Clone)]
pub enum EvalError<'a> {
    /// A *Raw* value means we've thrown an error from native code and don't yet
    /// know what error triggered it so that a real error can be displayed.
    Raw(BergError<'a>),
    /// A *Error* value means the error has at least one stack frame attached
    /// and is able to fully display itself.
    Error(Error<'a>),
}

#[derive(Debug, Clone)]
pub enum BergError<'a> {
    // File open errors
    SourceNotFound,
    IoOpenError,
    IoReadError,
    CurrentDirectoryError,
    SourceTooLarge(usize),

    // Code errors
    InvalidUtf8(RawLiteralIndex),
    UnsupportedCharacters(LiteralIndex),
    IdentifierStartsWithNumber(LiteralIndex),
    MissingExpression,
    AssignmentTargetMustBeIdentifier,
    OpenWithoutClose,
    CloseWithoutOpen,
    UnsupportedOperator(Box<BergVal<'a>>, Fixity, IdentifierIndex),
    DivideByZero,
    NoSuchField(FieldIndex),
    FieldNotSet(FieldIndex),
    CircularDependency,
    // TODO stop boxing BergVals
    BadType(Box<BergVal<'a>>, &'static str),
    BadOperandType(OperandPosition, Box<BergVal<'a>>, &'static str),
    PrivateField(BlockRef<'a>, IdentifierIndex),
    NoSuchPublicField(BlockRef<'a>, IdentifierIndex),
    NoSuchPublicFieldOnValue(Box<BergVal<'a>>, IdentifierIndex),
    NoSuchPublicFieldOnRoot(IdentifierIndex),
    ImmutableFieldOnRoot(FieldIndex),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ErrorCode {
    // Compile errors related to system (source)
    SourceNotFound = 101,
    IoOpenError,
    IoReadError,
    CurrentDirectoryError,
    SourceTooLarge,

    // Compile errors related to format (tokenizer)
    InvalidUtf8 = 201,
    UnsupportedCharacters,
    IdentifierStartsWithNumber,

    // Compile errors related to structure (parser)
    MissingOperand = 301,
    AssignmentTargetMustBeIdentifier,
    OpenWithoutClose,
    CloseWithoutOpen,

    // Compile errors related to type (checker)
    UnsupportedOperator = 1001,
    DivideByZero,
    BadType,
    NoSuchField,
    NoSuchPublicField,
    FieldNotSet,
    CircularDependency,
    PrivateField,
    ImmutableField,
}

pub enum ErrorLocation<'a> {
    Generic,
    SourceOnly(AstRef<'a>),
    SourceRange(AstRef<'a>, ByteRange),
}

impl<'a> From<BergError<'a>> for EvalError<'a> {
    fn from(error: BergError<'a>) -> Self {
        EvalError::Raw(error)
    }
}

impl<'a> From<Error<'a>> for EvalError<'a> {
    fn from(error: Error<'a>) -> Self {
        EvalError::Error(error)
    }
}

impl<'a> ErrorLocation<'a> {
    pub fn range(&self) -> LineColumnRange {
        match *self {
            ErrorLocation::SourceRange(ref ast, ref range) => ast.char_data().range(range),
            _ => unreachable!(),
        }
    }
    pub fn byte_range(&self) -> &ByteRange {
        match *self {
            ErrorLocation::SourceRange(_, ref range) => range,
            _ => unreachable!(),
        }
    }
}

impl<'a> Error<'a> {
    pub fn new(error: BergError<'a>, ast: &AstRef<'a>, expression: Expression) -> Self {
        Error {
            error,
            stack: Default::default(),
        }
        .push_frame(ast, expression)
    }

    pub fn push_frame(mut self, ast: &AstRef<'a>, expression: Expression) -> Self {
        self.stack.push((ast.clone(), expression));
        self
    }

    pub fn code(&self) -> ErrorCode {
        self.error.code()
    }

    pub fn location(&self) -> ErrorLocation<'a> {
        use self::BergError::*;
        use self::ErrorLocation::*;
        let ast = self.ast();
        match self.error {
            // File open errors
            CurrentDirectoryError => ErrorLocation::Generic,
            SourceNotFound | IoOpenError | IoReadError | SourceTooLarge(..) => {
                SourceOnly(ast.clone())
            }

            MissingExpression | UnsupportedOperator(..) => SourceRange(
                ast.clone(),
                ast.token_ranges()[self.expression().operator()].clone(),
            ),
            BadOperandType(position, ..) => {
                SourceRange(ast.clone(), position.get(self.expression(), ast).range(ast))
            }

            DivideByZero => SourceRange(
                ast.clone(),
                self.expression().right_expression(ast).range(ast),
            ),

            OpenWithoutClose => SourceRange(
                ast.clone(),
                ast.token_ranges()[self.expression().open_operator(ast)].clone(),
            ),

            CloseWithoutOpen => SourceRange(
                ast.clone(),
                ast.token_ranges()[self.expression().close_operator(ast)].clone(),
            ),

            // Expression errors
            InvalidUtf8(..)
            | UnsupportedCharacters(..)
            | IdentifierStartsWithNumber(..)
            | AssignmentTargetMustBeIdentifier
            | NoSuchField(..)
            | NoSuchPublicField(..)
            | NoSuchPublicFieldOnValue(..)
            | NoSuchPublicFieldOnRoot(..)
            | FieldNotSet(..)
            | CircularDependency
            | ImmutableFieldOnRoot(..)
            | PrivateField(..)
            | BadType(..) => ErrorLocation::SourceRange(ast.clone(), self.expression().range(ast)),
        }
    }

    fn ast_location(&self) -> &(AstRef<'a>, Expression) {
        self.stack.first().unwrap()
    }

    fn ast(&self) -> &AstRef<'a> {
        &self.ast_location().0
    }

    fn expression(&self) -> Expression {
        self.ast_location().1
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::ErrorCode::*;
        let string = match *self {
            SourceNotFound => "SourceNotFound",
            IoOpenError => "IoOpenError",
            IoReadError => "IoReadError",
            CurrentDirectoryError => "CurrentDirectoryError",
            SourceTooLarge => "SourceTooLarge",
            InvalidUtf8 => "InvalidUtf8",
            UnsupportedCharacters => "UnsupportedCharacters",
            IdentifierStartsWithNumber => "IdentifierStartsWithNumber",
            MissingOperand => "MissingOperand",
            AssignmentTargetMustBeIdentifier => "AssignmentTargetMustBeIdentifier",
            OpenWithoutClose => "OpenWithoutClose",
            CloseWithoutOpen => "CloseWithoutOpen",
            UnsupportedOperator => "UnsupportedOperator",
            DivideByZero => "DivideByZero",
            BadType => "BadType",
            NoSuchField => "NoSuchField",
            NoSuchPublicField => "NoSuchPublicField",
            FieldNotSet => "FieldNotSet",
            CircularDependency => "CircularDependency",
            PrivateField => "PrivateField",
            ImmutableField => "ImmutableField",
        };
        write!(f, "{}", string)
    }
}

impl<'a> fmt::Display for EvalError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use EvalError::*;
        match self {
            Raw(error) => write!(f, "{:?}", error),
            Error(error) => write!(f, "{}", error),
        }
    }
}

impl<'a> fmt::Display for Error<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use BergError::*;
        let expression = self.expression();
        let ast = self.ast();
        match self.error {
            SourceNotFound => write!(
                f,
                "I/O error getting current directory path {:?} ({}): {}",
                ast.source().absolute_path().unwrap(),
                ast.source().name(),
                ast.open_io_error()
            ),
            IoOpenError => write!(
                f,
                "I/O error opening {:?} ({}): {}",
                ast.source().absolute_path().unwrap(),
                ast.source().name(),
                ast.open_io_error()
            ),
            IoReadError => write!(
                f,
                "I/O error reading {:?} ({}): {}",
                ast.source().absolute_path().unwrap(),
                ast.source().name(),
                ast.open_io_error()
            ),
            CurrentDirectoryError => write!(
                f,
                "I/O error getting current directory to determine path of {:?}: {}",
                ast.source().name(),
                ast.root().root_path().as_ref().unwrap_err()
            ),
            SourceTooLarge(size) => write!(
                f,
                "SourceRef file {} too large ({} bytes): source files greater than 4GB are unsupported.",
                ast.source().name(),
                size
            ),
            InvalidUtf8(raw_literal) => {
                write!(f, "Invalid UTF-8 bytes! Perhaps this isn't a Berg UTF-8 source file? Invalid bytes: '")?;
                let bytes = ast.raw_literal_string(raw_literal);
                // Only print up to the first 12 bytes to prevent the error message from being ridiculous
                let print_max = 12.min(bytes.len());
                for byte in &bytes[0..print_max] {
                    write!(f, "{:2X}", byte)?;
                }
                if print_max > 12 {
                    write!(f, "...")?;
                }
                write!(f, "'")
            }
            UnsupportedCharacters(literal) => write!(f, "Unsupported Unicode characters! Perhaps this isn't a Berg source file? Unsupported characters: '{}'", ast.literal_string(literal)),
            OpenWithoutClose => write!(
                f,
                "Open '{}' found without a matching close '{}'.",
                expression.open_token(ast).to_string(ast),
                expression.boundary(ast).close_string()
            ),
            CloseWithoutOpen => write!(
                f,
                "Close '{}' found without a matching open '{}'.",
                expression.close_token(ast).to_string(ast),
                expression.boundary(ast).open_string()
            ),
            UnsupportedOperator(ref value, fixity, identifier) => write!(
                f,
                "Unsupported {} operator {} on value {}",
                fixity,
                ast.identifier_string(identifier),
                value
            ),
            DivideByZero => write!(
                f,
                "Division by zero is illegal. Perhaps you meant a different number on the right hand side of the '{}'?",
                expression.token(ast).to_string(ast)
            ),
            NoSuchField(field_index) => write!(
                f,
                "No such field: '{}'",
                ast.field_name(field_index)
            ),
            FieldNotSet(field_index) => write!(
                f,
                "Field '{}' was declared, but never set to a value!",
                ast.field_name(field_index)
            ),
            NoSuchPublicField(ref block, name) => write!(
                f,
                "No field '{}' exists on '{}'! Perhaps it's a misspelling?",
                ast.identifier_string(name),
                block
            ),
            NoSuchPublicFieldOnValue(ref value, name) => write!(
                f,
                "No field '{}' exists on '{}'! Perhaps it's a misspelling?",
                ast.identifier_string(name),
                value
            ),
            NoSuchPublicFieldOnRoot(name) => write!(
                f,
                "No field '{}' exists on the root! Also, how did you manage to do '.' on the root?",
                ast.identifier_string(name)
            ),
            PrivateField(ref value, name) => write!(
                f,
                "Field '{}' on '{}' is private and cannot be accessed with '.'! Perhaps you meant to declare the field with ':{}' instead of '{}'?",
                ast.identifier_string(name),
                value,
                ast.identifier_string(name),
                ast.identifier_string(name)
            ),
            ImmutableFieldOnRoot(field_index) => write!(
                f,
                "'{}' cannot be modified!",
                ast.field_name(field_index)
            ),
            IdentifierStartsWithNumber(literal) => write!(
                f,
                "Field names must start with letters or '_', but '{}' starts with a number! You may have mistyped the field name, or missed an operator?",
                ast.literal_string(literal)
            ),
            CircularDependency => write!(
                f,
                "Circular dependency at '{}'!",
                expression.to_string(ast)
            ),
            MissingExpression => write!(
                f,
                "Operator {} has no value on {} to operate on!",
                expression.token(ast).to_string(ast),
                expression.operand_position(ast)
            ),
            AssignmentTargetMustBeIdentifier => write!(
                f,
                "The assignment operator '{operator}' must have a field declaration or name on {position} (like \":foo {operator} ...\" or \"foo {operator} ...\": {position} is currently {operand}.",
                operator = expression.parent(ast).token(ast).to_string(ast),
                position = expression.operand_position(ast),
                operand = expression.to_string(ast),
            ),
            BadOperandType(position,ref actual_value,expected_type) => write!(
                f,
                "The value of '{operand}' is {actual_value}, but {position} '{operator}' must be an {expected_type}!",
                operand = position.get(expression, ast).to_string(ast),
                actual_value = actual_value,
                position = position,
                operator = expression.token(ast).to_string(ast),
                expected_type = expected_type
            ),
            BadType(ref actual_value,expected_type) => write!(
                f,
                "The value of '{}' is {}, but we expected {}!",
                expression.to_string(ast),
                actual_value,
                expected_type
            ),
        }
    }
}

impl<'a> BergError<'a> {
    pub fn push_frame(self, ast: &AstRef<'a>, expression: Expression) -> Error<'a> {
        Error::new(self, ast, expression)
    }

    pub fn err<T>(self) -> Result<T, EvalError<'a>> {
        Err(EvalError::Raw(self))
    }

    pub fn code(&self) -> ErrorCode {
        use self::BergError::*;
        match *self {
            // File open errors
            SourceNotFound => ErrorCode::SourceNotFound,
            IoOpenError => ErrorCode::IoOpenError,
            IoReadError => ErrorCode::IoReadError,
            CurrentDirectoryError => ErrorCode::CurrentDirectoryError,
            SourceTooLarge(..) => ErrorCode::SourceTooLarge,

            // Expression errors
            InvalidUtf8(..) => ErrorCode::InvalidUtf8,
            UnsupportedCharacters(..) => ErrorCode::UnsupportedCharacters,
            IdentifierStartsWithNumber(..) => ErrorCode::IdentifierStartsWithNumber,
            MissingExpression => ErrorCode::MissingOperand,
            AssignmentTargetMustBeIdentifier => ErrorCode::AssignmentTargetMustBeIdentifier,
            OpenWithoutClose => ErrorCode::OpenWithoutClose,
            CloseWithoutOpen => ErrorCode::CloseWithoutOpen,

            // Compile errors related to type (checker)
            UnsupportedOperator(..) => ErrorCode::UnsupportedOperator,
            DivideByZero => ErrorCode::DivideByZero,
            NoSuchField(..) => ErrorCode::NoSuchField,
            NoSuchPublicField(..) | NoSuchPublicFieldOnValue(..) | NoSuchPublicFieldOnRoot(..) => {
                ErrorCode::NoSuchPublicField
            }
            PrivateField(..) => ErrorCode::PrivateField,
            FieldNotSet(..) => ErrorCode::FieldNotSet,
            CircularDependency => ErrorCode::CircularDependency,
            ImmutableFieldOnRoot(..) => ErrorCode::ImmutableField,
            BadOperandType(..) | BadType(..) => ErrorCode::BadType,
        }
    }
}

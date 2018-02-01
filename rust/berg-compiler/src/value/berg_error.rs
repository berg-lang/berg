use syntax::{AstRef, FieldIndex, Fixity, IdentifierIndex, LineColumnRange, OperandPosition};
use eval::Expression;
use parser::ByteRange;
use std::fmt;
use util::try_from::TryFrom;
use value::*;

#[derive(Debug, Clone)]
pub enum BergError<'a> {
    // File open errors
    SourceNotFound,
    IoOpenError,
    IoReadError,
    CurrentDirectoryError,
    SourceTooLarge(usize),

    // Code errors
    InvalidUtf8,
    UnsupportedCharacters,
    IdentifierStartsWithNumber,
    MissingOperand,
    AssignmentTargetMustBeIdentifier,
    OpenWithoutClose,
    CloseWithoutOpen,
    UnsupportedOperator(Box<BergVal<'a>>, Fixity, IdentifierIndex),
    DivideByZero,
    NoSuchField(FieldIndex),
    FieldNotSet(FieldIndex),
    CircularDependency,
    BadType(Box<BergVal<'a>>, &'static str),
    BadOperandType(OperandPosition, Box<BergVal<'a>>, &'static str),
    ImmutableField(FieldIndex),
}

#[derive(Debug, Clone)]
pub struct BergErrorStack<'a> {
    pub error: BergError<'a>,
    pub stack: Vec<(AstRef<'a>, Expression)>,
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
    FieldNotSet,
    CircularDependency,
    ImmutableField,
}

pub enum ErrorLocation<'a> {
    Generic,
    SourceOnly(AstRef<'a>),
    SourceRange(AstRef<'a>, ByteRange),
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

impl<'a> TypeName for BergErrorStack<'a> {
    const TYPE_NAME: &'static str = "error";
}

impl<'a> BergValue<'a> for BergErrorStack<'a> {
    fn unwind_error(mut self, ast: AstRef<'a>, expression: Expression) -> BergVal<'a> {
        self.stack.push((ast, expression));
        self.into()
    }
}

impl<'a> From<BergError<'a>> for BergVal<'a> {
    fn from(from: BergError<'a>) -> Self {
        BergVal::BergErrorStack(BergErrorStack::empty(from))
    }
}

impl<'a> From<BergErrorStack<'a>> for BergVal<'a> {
    fn from(from: BergErrorStack<'a>) -> Self {
        BergVal::BergErrorStack(from)
    }
}

impl<'a> TryFrom<BergVal<'a>> for BergErrorStack<'a> {
    type Error = BergVal<'a>;
    fn try_from(from: BergVal<'a>) -> Result<Self, Self::Error> {
        match from {
            BergVal::BergErrorStack(value) => Ok(value),
            _ => Err(from),
        }
    }
}

impl<'a> BergErrorStack<'a> {
    pub fn code(&self) -> ErrorCode {
        self.error.code()
    }

    pub fn location(&self) -> ErrorLocation<'a> {
        use value::BergError::*;
        use value::ErrorLocation::*;
        let ast = self.ast();
        match self.error {
            // File open errors
            CurrentDirectoryError => ErrorLocation::Generic,
            SourceNotFound | IoOpenError | IoReadError | SourceTooLarge(..) => {
                SourceOnly(ast.clone())
            }

            MissingOperand | UnsupportedOperator(..) => SourceRange(
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
            InvalidUtf8
            | UnsupportedCharacters
            | IdentifierStartsWithNumber
            | AssignmentTargetMustBeIdentifier
            | NoSuchField(..)
            | FieldNotSet(..)
            | CircularDependency
            | ImmutableField(..)
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
        use value::ErrorCode::*;
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
            FieldNotSet => "FieldNotSet",
            CircularDependency => "CircularDependency",
            ImmutableField => "ImmutableField",
        };
        write!(f, "{}", string)
    }
}

impl<'a> fmt::Display for BergErrorStack<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use value::BergError::*;
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
            InvalidUtf8 => {
                write!(f, "Invalid UTF-8! Perhaps this isn't a Berg source file?")
            }
            UnsupportedCharacters => write!(f, "Invalid Unicode characters"),
            CloseWithoutOpen => write!(
                f,
                "Open '{}' found without a matching close '{}'.",
                expression.open_token(ast).to_string(ast),
                expression.boundary(ast).close_string()
            ),
            OpenWithoutClose => write!(
                f,
                "Close '{}' found without a matching open '{}'.",
                expression.close_token(ast).to_string(ast),
                expression.boundary(ast).open_string()
            ),
            UnsupportedOperator(ref value, fixity, identifier) => write!(
                f,
                "Unsupported {} operator {} on value {:?}",
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
            ImmutableField(field_index) => write!(
                f,
                "'{}' cannot be modified!",
                ast.field_name(field_index)
            ),
            IdentifierStartsWithNumber => write!(
                f,
                "Fields must start with letters or '_', but field '{}' starts with a number! Perhaps you meant to reference a field, or were typing a number?",
                expression.to_string(ast)
            ),
            CircularDependency => write!(
                f,
                "Circular dependency at '{}'!",
                expression.to_string(ast)
            ),
            MissingOperand => write!(
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
                "The value of '{operand}' is {actual_value:?}, but {position} '{operator}' must be an {expected_type}!",
                operand = position.get(expression, ast).to_string(ast),
                actual_value = actual_value,
                position = position,
                operator = expression.token(ast).to_string(ast),
                expected_type = expected_type
            ),
            BadType(ref actual_value,expected_type) => write!(
                f,
                "The value of '{operand}' is {actual_value:?}, but we expected {expected_type}!",
                operand = expression.to_string(ast),
                actual_value = actual_value,
                expected_type = expected_type
            ),
        }
    }
}

impl<'a> BergError<'a> {
    pub fn err<T>(self) -> BergResult<'a, T> {
        Err(self.into())
    }

    pub fn code(&self) -> ErrorCode {
        use value::BergError::*;
        match *self {
            // File open errors
            SourceNotFound => ErrorCode::SourceNotFound,
            IoOpenError => ErrorCode::IoOpenError,
            IoReadError => ErrorCode::IoReadError,
            CurrentDirectoryError => ErrorCode::CurrentDirectoryError,
            SourceTooLarge(..) => ErrorCode::SourceTooLarge,

            // Expression errors
            InvalidUtf8 => ErrorCode::InvalidUtf8,
            UnsupportedCharacters => ErrorCode::UnsupportedCharacters,
            IdentifierStartsWithNumber => ErrorCode::IdentifierStartsWithNumber,
            MissingOperand => ErrorCode::MissingOperand,
            AssignmentTargetMustBeIdentifier => ErrorCode::AssignmentTargetMustBeIdentifier,
            OpenWithoutClose => ErrorCode::OpenWithoutClose,
            CloseWithoutOpen => ErrorCode::CloseWithoutOpen,

            // Compile errors related to type (checker)
            UnsupportedOperator(..) => ErrorCode::UnsupportedOperator,
            DivideByZero => ErrorCode::DivideByZero,
            NoSuchField(..) => ErrorCode::NoSuchField,
            FieldNotSet(..) => ErrorCode::FieldNotSet,
            CircularDependency => ErrorCode::CircularDependency,
            ImmutableField(..) => ErrorCode::ImmutableField,
            BadOperandType(..) | BadType(..) => ErrorCode::BadType,
        }
    }
}

impl<'a> BergErrorStack<'a> {
    fn empty(error: BergError<'a>) -> Self {
        BergErrorStack {
            error,
            stack: Default::default(),
        }
    }
}

use crate::eval::BlockRef;
use crate::syntax::{
    AstIndex, AstRef, ByteRange, ExpressionTreeWalker, ExpressionRef, FieldIndex, Fixity, IdentifierIndex,
    LineColumnRange, LiteralIndex, RawLiteralIndex,
};
use crate::value::implement::*;
use std::fmt;

///
/// Standard berg error, either with or without a full error location.
/// 
#[derive(Debug, Clone)]
pub enum ErrorVal<'a> {
    ExpressionError(BergError<'a>, ExpressionErrorPosition),
    Error(Error<'a>),
}

///
/// Standard berg error.
/// 
/// Contains a BergError and a stack of error locations.
///
#[derive(Debug, Clone)]
pub struct Error<'a> {
    pub error: BergError<'a>,
    pub expression: ExpressionRef<'a>,
}

///
/// Standard berg error.
/// 
/// This class is generally used to determine the type of an error, or for
/// implementors to create local errors without having to know an expression's
/// location. An Error or EvalError is needed to give it a source location that
/// can actually be reported.
/// 
#[derive(Debug, Clone)]
pub enum BergError<'a> {
    // File open errors

    ///
    /// The source file to be read could not be found.
    /// 
    SourceNotFound,

    ///
    /// There was an I/O error opening the source file. The file may or may not
    /// exist (depending on the type of the IoOpenError).
    /// 
    IoOpenError,

    ///
    /// There was an I/O error reading the source file.
    ///
    IoReadError,

    ///
    /// There was an error determining the current directory.
    ///
    CurrentDirectoryError,

    ///
    /// A source file was more than 32-bits (4GB).
    /// 
    SourceTooLarge(usize),

    // Code errors
    InvalidUtf8(RawLiteralIndex),
    UnsupportedCharacters(LiteralIndex),
    IdentifierStartsWithNumber(LiteralIndex),
    MissingOperand,
    AssignmentTargetMustBeIdentifier,
    RightSideOfDotMustBeIdentifier,
    OpenWithoutClose,
    CloseWithoutOpen,
    UnsupportedOperator(Box<BergResult<'a>>, Fixity, IdentifierIndex),
    DivideByZero,
    NoSuchField(FieldIndex),
    FieldNotSet(FieldIndex),
    CircularDependency,
    IfWithoutBlock,
    IfWithoutCondition,
    IfBlockMustBeBlock,
    ElseBlockMustBeBlock,
    ElseWithoutBlock,
    ElseWithoutIf,
    IfFollowedByNonElse,
    WhileWithoutCondition,
    WhileWithoutBlock,
    WhileConditionMustBeBlock,
    WhileBlockMustBeBlock,
    ForeachWithoutInput,
    ForeachWithoutBlock,
    ForeachBlockMustBeBlock,
    TryWithoutBlock,
    TryBlockMustBeBlock,
    TryWithoutCatchOrFinally,
    CatchWithoutBlock,
    CatchBlockMustBeBlock,
    CatchWithoutResult,
    FinallyWithoutBlock,
    FinallyBlockMustBeBlock,
    FinallyWithoutResult,
    // TODO stop boxing BergVals
    // BadOperandType(Box<EvalResult<'a>>, &'static str),
    BadOperandType(Box<BergResult<'a>>, &'static str),
    PrivateField(BlockRef<'a>, IdentifierIndex),
    NoSuchPublicField(BlockRef<'a>, IdentifierIndex),
    NoSuchPublicFieldOnValue(Box<BergResult<'a>>, IdentifierIndex),
    NoSuchPublicFieldOnRoot(IdentifierIndex),
    ImmutableFieldOnRoot(FieldIndex),

    // These are control values--only errors if nobody catches them.
    BreakOutsideLoop,
    ContinueOutsideLoop,
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
    RightSideOfDotMustBeIdentifier,
    OpenWithoutClose,
    CloseWithoutOpen,
    IfWithoutCondition,
    IfWithoutBlock,
    IfBlockMustBeBlock,
    ElseBlockMustBeBlock,
    ElseWithoutBlock,
    ElseWithoutIf,
    IfFollowedByNonElse,
    WhileWithoutCondition,
    WhileWithoutBlock,
    WhileConditionMustBeBlock,
    WhileBlockMustBeBlock,
    ForeachWithoutInput,
    ForeachWithoutBlock,
    ForeachBlockMustBeBlock,
    TryWithoutBlock,
    TryBlockMustBeBlock,
    TryWithoutCatchOrFinally,
    CatchWithoutBlock,
    CatchBlockMustBeBlock,
    CatchWithoutResult,
    FinallyWithoutBlock,
    FinallyBlockMustBeBlock,
    FinallyWithoutResult,

    // Compile errors related to type (checker)
    UnsupportedOperator = 1001,
    DivideByZero,
    BadOperandType,
    NoSuchField,
    NoSuchPublicField,
    FieldNotSet,
    CircularDependency,
    PrivateField,
    ImmutableField,
    BreakOutsideLoop,
    ContinueOutsideLoop,
}

#[derive(Debug, Clone)]
pub enum ErrorLocation<'a> {
    Generic,
    SourceOnly(AstRef<'a>),
    SourceExpression(AstRef<'a>, AstIndex),
    SourceRange(AstRef<'a>, ByteRange),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ExpressionErrorPosition {
    Expression,
    Left,
    Right,
    LeftLeft,
    LeftRight,
    RightLeft,
    RightRight,
}

impl<'a> ErrorVal<'a> {
    pub fn code(&self) -> ErrorCode {
        use ErrorVal::*;
        match self {
            ExpressionError(error, _) => error.code(),            
            Error(error) => error.code(),
        }
    }
    pub fn reposition(self, new_position: ExpressionErrorPosition) -> ErrorVal<'a> {
        use ErrorVal::*;
        use ExpressionErrorPosition::*;
        match self {
            ExpressionError(error, position) => match (new_position, position) {
                (new_position, Expression) => ExpressionError(error, new_position),
                (Expression, position) => ExpressionError(error, position),
                (Left, Left) => ExpressionError(error, LeftLeft),
                (Left, Right) => ExpressionError(error, LeftRight),
                (Right, Left) => ExpressionError(error, RightLeft),
                (Right, Right) => ExpressionError(error, RightRight),
                _ => unreachable!("{:?} {:?} at {:?}", error, position, new_position),
            }
            _ => self,
        }

    }
}
impl<'a> ErrorLocation<'a> {
    pub fn range(&self) -> LineColumnRange {
        match self {
            ErrorLocation::SourceExpression(ast, _) | ErrorLocation::SourceRange(ast, _) => {
                ast.char_data.range(&self.byte_range())
            }
            _ => unreachable!(),
        }
    }
    pub fn byte_range(&self) -> ByteRange {
        match self {
            ErrorLocation::SourceExpression(ast, index) => {
                ExpressionTreeWalker::new((), ast, *index).byte_range()
            }
            ErrorLocation::SourceRange(_, range) => range.clone(),
            _ => unreachable!(),
        }
    }
}

impl<'a> Error<'a> {
    pub fn new(error: BergError<'a>, expression: ExpressionRef<'a>) -> Self {
        Error {
            error,
            expression,
        }
    }

    pub fn code(&self) -> ErrorCode {
        self.error.code()
    }

    pub fn expression(&self) -> ExpressionRef<'a> {
        self.expression.clone()
    }

    pub fn location(&self) -> ErrorLocation<'a> {
        use self::BergError::*;
        use self::ErrorLocation::*;
        let expression = self.expression();
        match self.error {
            // File open errors
            CurrentDirectoryError => ErrorLocation::Generic,
            SourceNotFound | IoOpenError | IoReadError | SourceTooLarge(..) => {
                SourceOnly(expression.ast)
            }

            MissingOperand => {
                let range = expression.ast.token_ranges[expression.expression().parent_expression().root_index()].clone();
                SourceRange(expression.ast, range)
            }

            UnsupportedOperator(..) => {
                let range = expression.ast.token_ranges[expression.expression().root_index()].clone();
                SourceRange(expression.ast, range)
            }

            DivideByZero | RightSideOfDotMustBeIdentifier | IfFollowedByNonElse => {
                let operand = expression.expression().right_expression().root_index();
                SourceExpression(expression.ast, operand)
            }

            IfWithoutBlock => {
                let if_expression = expression.expression().left_expression().root_index();
                SourceExpression(expression.ast, if_expression)
            }
            OpenWithoutClose => {
                let range =
                    expression.ast.token_ranges[expression.expression().open_operator()].clone();
                SourceRange(expression.ast, range)
            }

            CloseWithoutOpen => {
                let range =
                    expression.ast.token_ranges[expression.expression().close_operator()].clone();
                SourceRange(expression.ast, range)
            }


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
            | BadOperandType(..)
            | IfWithoutCondition
            | ElseWithoutBlock
            | ElseWithoutIf
            | IfBlockMustBeBlock
            | ElseBlockMustBeBlock
            | WhileWithoutCondition
            | WhileWithoutBlock
            | WhileConditionMustBeBlock
            | WhileBlockMustBeBlock
            | ForeachWithoutInput
            | ForeachWithoutBlock
            | ForeachBlockMustBeBlock
            | BreakOutsideLoop
            | ContinueOutsideLoop
            | TryWithoutBlock
            | TryBlockMustBeBlock
            | TryWithoutCatchOrFinally
            | CatchWithoutBlock
            | CatchBlockMustBeBlock
            | CatchWithoutResult
            | FinallyWithoutBlock
            | FinallyBlockMustBeBlock
            | FinallyWithoutResult
            => ErrorLocation::SourceExpression(expression.ast, expression.root),
        }
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
            RightSideOfDotMustBeIdentifier => "RightSideOfDotMustBeIdentifier",
            OpenWithoutClose => "OpenWithoutClose",
            CloseWithoutOpen => "CloseWithoutOpen",
            UnsupportedOperator => "UnsupportedOperator",
            DivideByZero => "DivideByZero",
            BadOperandType => "BadOperandType",
            NoSuchField => "NoSuchField",
            NoSuchPublicField => "NoSuchPublicField",
            FieldNotSet => "FieldNotSet",
            CircularDependency => "CircularDependency",
            PrivateField => "PrivateField",
            ImmutableField => "ImmutableField",
            IfWithoutBlock => "IfWithoutBlock",
            IfWithoutCondition => "IfWithoutCondition",
            IfBlockMustBeBlock => "IfBlockMustBeBlock",
            ElseBlockMustBeBlock => "ElseBlockMustBeBlock",
            ElseWithoutBlock => "ElseWithoutBlock",
            ElseWithoutIf => "ElseWithoutIf",
            IfFollowedByNonElse => "IfFollowedByNonElse",
            WhileWithoutCondition => "WhileWithoutCondition",
            WhileWithoutBlock => "WhileWithoutBlock",
            WhileConditionMustBeBlock => "WhileConditionMustBeBlock",
            WhileBlockMustBeBlock => "WhileBlockMustBeBlock",
            ForeachWithoutInput => "ForeachWithoutInput",
            ForeachWithoutBlock => "ForeachWithoutBlock",
            ForeachBlockMustBeBlock => "ForeachBlockMustBeBlock",
            BreakOutsideLoop => "BreakOutsideLoop",
            ContinueOutsideLoop => "ContinueOutsideLoop",
            TryWithoutBlock => "TryWithoutBlock",
            TryBlockMustBeBlock => "TryBlockMustBeBlock",
            TryWithoutCatchOrFinally => "TryWithoutCatchOrFinally",
            CatchWithoutBlock => "CatchWithoutBlock",
            CatchBlockMustBeBlock => "CatchBlockMustBeBlock",
            CatchWithoutResult => "CatchWithoutResult",
            FinallyWithoutBlock => "FinallyWithoutBlock",
            FinallyBlockMustBeBlock => "FinallyBlockMustBeBlock",
            FinallyWithoutResult => "FinallyWithoutResult",
        };
        write!(f, "{}", string)
    }
}

impl<'a> fmt::Display for ErrorVal<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ErrorVal::*;
        match self {
            Error(error) => write!(f, "{}", error),
            ExpressionError(error, position) => match position {
                ExpressionErrorPosition::Expression => write!(f, "{:?}", error),
                _ => write!(f, "{:?} at {:?}", error, position)
            }
        }
    }
}

impl<'a> fmt::Display for Error<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use BergError::*;
        let expression = self.expression();
        match self.error {
            SourceNotFound => write!(
                f,
                "I/O error getting current directory path {:?} ({}): {}",
                expression.ast.source.absolute_path().unwrap(),
                expression.ast.source.name(),
                expression.ast.open_io_error()
            ),
            IoOpenError => write!(
                f,
                "I/O error opening {:?} ({}): {}",
                expression.ast.source.absolute_path().unwrap(),
                expression.ast.source.name(),
                expression.ast.open_io_error()
            ),
            IoReadError => write!(
                f,
                "I/O error reading {:?} ({}): {}",
                expression.ast.source.absolute_path().unwrap(),
                expression.ast.source.name(),
                expression.ast.open_io_error()
            ),
            CurrentDirectoryError => write!(
                f,
                "I/O error getting current directory to determine path of {:?}: {}",
                expression.ast.source.name(),
                expression.ast.root().root_path().as_ref().unwrap_err()
            ),
            SourceTooLarge(size) => write!(
                f,
                "SourceRef file {} too large ({} bytes): source files greater than 4GB are unsupported.",
                expression.ast.source.name(),
                size
            ),
            InvalidUtf8(raw_literal) => {
                write!(f, "Invalid UTF-8 bytes! Perhaps this isn't a Berg UTF-8 source file? Invalid bytes: '")?;
                let bytes = expression.ast.raw_literal_string(raw_literal);
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
            UnsupportedCharacters(literal) => write!(f, "Unsupported Unicode characters! Perhaps this isn't a Berg source file? Unsupported characters: '{}'", expression.ast.literal_string(literal)),
            OpenWithoutClose => write!(
                f,
                "Open '{}' found without a matching close '{}'.",
                expression.expression().open_token().to_string(&expression.ast),
                expression.expression().boundary().close_string()
            ),
            CloseWithoutOpen => write!(
                f,
                "Close '{}' found without a matching open '{}'.",
                expression.expression().close_token().to_string(&expression.ast),
                expression.expression().boundary().open_string()
            ),
            UnsupportedOperator(ref value, fixity, identifier) => write!(
                f,
                "Unsupported {} operator {} on value {}",
                fixity,
                expression.ast.identifier_string(identifier),
                value.display()
            ),
            DivideByZero => write!(
                f,
                "Division by zero is illegal. Perhaps you meant a different number on the right hand side of the '{}'?",
                expression.expression().token().to_string(&expression.ast)
            ),
            NoSuchField(field_index) => write!(
                f,
                "No such field: '{}'",
                expression.ast.field_name(field_index)
            ),
            FieldNotSet(field_index) => write!(
                f,
                "Field '{}' was declared, but never set to a value!",
                expression.ast.field_name(field_index)
            ),
            NoSuchPublicField(ref block, name) => write!(
                f,
                "No field '{}' exists on '{}'! Perhaps it's a misspelling?",
                expression.ast.identifier_string(name),
                block
            ),
            NoSuchPublicFieldOnValue(ref value, name) => write!(
                f,
                "No field '{}' exists on '{}'! Perhaps it's a misspelling?",
                expression.ast.identifier_string(name),
                value.display()
            ),
            NoSuchPublicFieldOnRoot(name) => write!(
                f,
                "No field '{}' exists on the root! By the way, how in the world did you manage to do '.' on the root? That's supposed to be impossible ...",
                expression.ast.identifier_string(name)
            ),
            IfWithoutCondition => write!(
                f,
                "if statement missing a condition! Did you mean to add a condition, such as 'if x == 1'?"
            ),
            IfWithoutBlock => write!(
                f,
                "if statement missing a block! if needs two arguments, a condition and then a block, such as '{} {{ do something here; }}'?",
                expression.expression()
            ),
            IfBlockMustBeBlock => write!(
                f,
                "if block must be a block! Did you mean to add brackets here, like '{{ {} }}'?",
                expression.expression()
            ),
            ElseWithoutBlock => write!(
                f,
                "else statement missing a block! else requires a block to run, such as '... else {{ do something here; }}'?"
            ),
            ElseBlockMustBeBlock => write!(
                f,
                "else block must be a block! Did you mean to add brackets here, like '{{ {} }}'?",
                expression.expression()
            ),
            ElseWithoutIf => write!(
                f,
                "else statement without if! else can only happen after an if statement, like if 1 == 1 {{ }} else {{ }}"
            ),
            IfFollowedByNonElse => write!(
                f,
                "Extra statement after if! if statements can only be followed by 'else' or 'else if'. Perhaps you meant to put the code in a block, or to insert a semicolon to terminate the if?"
            ),
            WhileWithoutCondition => write!(
                f,
                "while statement missing a condition! Did you mean to add a condition, such as 'while x < 10'?"
            ),
            WhileConditionMustBeBlock => write!(
                f,
                "while condition must be a block to ensure it can be called multiple times! Did you mean to add brackets here, like '{{ {} }}'?",
                expression.expression()
            ),
            WhileWithoutBlock => write!(
                f,
                "while statement missing a block! while requires a block to run, such as 'while x < 10 {{ x++ }}'?"
            ),
            WhileBlockMustBeBlock => write!(
                f,
                "while block must be a block! Did you mean to add brackets here, like '{{ {} }}'?",
                expression.expression()
            ),
            ForeachWithoutInput => write!(
                f,
                "foreach is missing input and block! It should look like: foreach Collection {{ :value }}"
            ),
            ForeachWithoutBlock => write!(
                f,
                "foreach statement missing a block! while requires a block to run, such as 'while x < 10 {{ x++ }}'?"
            ),
            ForeachBlockMustBeBlock => write!(
                f,
                "foreach block must be a block! Did you mean to add brackets here, like '{{ {} }}'?",
                expression.expression()
            ),
            BreakOutsideLoop => write!(
                f,
                "break found outside loop! break must be called from within a while loop."
            ),
            ContinueOutsideLoop => write!(
                f,
                "continue found outside loop! break must be called from within a while loop."
            ),
            TryWithoutBlock => write!(
                f,
                "try statement missing a block! try requires a block to run, such as 'try {{ x++ }}'."
            ),
            TryBlockMustBeBlock => write!(
                f,
                "try block must be a block! Did you mean to add brackets here, like '{{ {} }}'?",
                expression.expression()
            ),
            TryWithoutCatchOrFinally => write!(
                f,
                "try must be followed by catch or finally!"
            ),
            CatchWithoutBlock => write!(
                f,
                "catch statement missing a block! catch requires a block to run, such as 'try {{ 1/0 }} catch {{ :error.ErrorCode }}'."
            ),
            CatchBlockMustBeBlock => write!(
                f,
                "catch block must be a block! Did you mean to add brackets here, like '{{ {} }}'?",
                expression.expression()
            ),
            CatchWithoutResult => write!(
                f,
                "catch statement must follow an expression! For example, '{{ 1/0 }} catch {{ :error.ErrorCode }}'?"
            ),
            FinallyWithoutBlock => write!(
                f,
                "finally statement missing a block! while requires a block to run, such as 'while x < 10 {{ x++ }}'?"
            ),
            FinallyBlockMustBeBlock => write!(
                f,
                "finally block must be a block! Did you mean to add brackets here, like '{{ {} }}'?",
                expression.expression()
            ),
            FinallyWithoutResult => write!(
                f,
                "finally statement must follow an expression! For example, '{{ 1/0 }} finally {{ ... }}'?"
            ),
            PrivateField(ref value, name) => write!(
                f,
                "Field '{}' on '{}' is private and cannot be accessed with '.'! Perhaps you meant to declare the field with ':{}' instead of '{}'?",
                expression.ast.identifier_string(name),
                value,
                expression.ast.identifier_string(name),
                expression.ast.identifier_string(name)
            ),
            ImmutableFieldOnRoot(field_index) => write!(
                f,
                "'{}' cannot be modified!",
                expression.ast.field_name(field_index)
            ),
            IdentifierStartsWithNumber(literal) => write!(
                f,
                "Field names must start with letters or '_', but '{}' starts with a number! You may have mistyped the field name, or missed an operator?",
                expression.ast.literal_string(literal)
            ),
            CircularDependency => write!(
                f,
                "Circular dependency at '{}'!",
                expression
            ),
            MissingOperand => write!(
                f,
                "Operator {} has no value on {} to operate on!",
                expression.expression().parent_expression().token().to_string(&expression.ast),
                expression.expression().operand_position()
            ),
            AssignmentTargetMustBeIdentifier => write!(
                f,
                "The assignment operator '{operator}' must have a field declaration or name on {position} (like \":foo {operator} ...\" or \"foo {operator} ...\": {position} is currently {operand}.",
                operator = expression.expression().parent_expression().token().to_string(&expression.ast),
                position = expression.expression().operand_position(),
                operand = expression,
            ),
            RightSideOfDotMustBeIdentifier => write!(
                f,
                "The field access operator '{operator}' must have an identifier on the right side (like \"{left}.FieldName\"): currently it is '{right}'.",
                operator = expression.expression().token().to_string(&expression.ast),
                left = expression.expression().left_expression(),
                right = expression.expression().right_expression(),
            ),
            BadOperandType(ref actual_value,expected_type) => write!(
                f,
                "The value of '{operand}' is {actual_value}, but {position} '{operator}' must be an {expected_type}!",
                operand = expression.expression(),
                actual_value = actual_value.display(),
                position = expression.expression().operand_position(),
                operator = expression.expression().token_string(),
                expected_type = expected_type
            ),
            // BadOperandType(ref actual_value,expected_type) => write!(
            //     f,
            //     "The value of '{}' is {}, but we expected {}!",
            //     expression,
            //     actual_value.display(),
            //     expected_type
            // ),
        }
    }
}

impl<'a> BergError<'a> {
    pub fn at_location(self, expression: impl Into<ExpressionRef<'a>>) -> Error<'a> {
        Error::new(self, expression.into())
    }

    pub fn operand_err<T>(self, position: ExpressionErrorPosition) -> Result<T, ErrorVal<'a>> {
        Err(ErrorVal::ExpressionError(self, position))
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
            MissingOperand => ErrorCode::MissingOperand,
            AssignmentTargetMustBeIdentifier => ErrorCode::AssignmentTargetMustBeIdentifier,
            RightSideOfDotMustBeIdentifier => ErrorCode::RightSideOfDotMustBeIdentifier,
            OpenWithoutClose => ErrorCode::OpenWithoutClose,
            CloseWithoutOpen => ErrorCode::CloseWithoutOpen,
            IfWithoutCondition => ErrorCode::IfWithoutCondition,
            IfWithoutBlock => ErrorCode::IfWithoutBlock,
            ElseWithoutBlock => ErrorCode::ElseWithoutBlock,
            ElseWithoutIf => ErrorCode::ElseWithoutIf,
            IfFollowedByNonElse => ErrorCode::IfFollowedByNonElse,
            IfBlockMustBeBlock => ErrorCode::IfBlockMustBeBlock,
            ElseBlockMustBeBlock => ErrorCode::ElseBlockMustBeBlock,
            WhileWithoutCondition => ErrorCode::WhileWithoutCondition,
            WhileWithoutBlock => ErrorCode::WhileWithoutBlock,
            WhileConditionMustBeBlock => ErrorCode::WhileConditionMustBeBlock,
            WhileBlockMustBeBlock => ErrorCode::WhileBlockMustBeBlock,
            BreakOutsideLoop => ErrorCode::BreakOutsideLoop,
            ContinueOutsideLoop => ErrorCode::ContinueOutsideLoop,
            ForeachWithoutInput => ErrorCode::ForeachWithoutInput,
            ForeachWithoutBlock => ErrorCode::ForeachWithoutBlock,
            ForeachBlockMustBeBlock => ErrorCode::ForeachBlockMustBeBlock,
            TryWithoutBlock => ErrorCode::TryWithoutBlock,
            TryBlockMustBeBlock => ErrorCode::TryBlockMustBeBlock,
            TryWithoutCatchOrFinally => ErrorCode::TryWithoutCatchOrFinally,
            CatchWithoutBlock => ErrorCode::CatchWithoutBlock,
            CatchBlockMustBeBlock => ErrorCode::CatchBlockMustBeBlock,
            CatchWithoutResult => ErrorCode::CatchWithoutResult,
            FinallyWithoutBlock => ErrorCode::FinallyWithoutBlock,
            FinallyBlockMustBeBlock => ErrorCode::FinallyBlockMustBeBlock,
            FinallyWithoutResult => ErrorCode::FinallyWithoutResult,

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
            BadOperandType(..) => ErrorCode::BadOperandType,
        }
    }
}

impl<'a> From<BergError<'a>> for ErrorVal<'a> {
    fn from(from: BergError<'a>) -> Self {
        ErrorVal::ExpressionError(from, ExpressionErrorPosition::Expression)
    }
}

impl<'a> From<Error<'a>> for ErrorVal<'a> {
    fn from(from: Error<'a>) -> Self {
        ErrorVal::Error(from)
    }
}

impl<'a> BergValue<'a> for ErrorVal<'a> {
    fn next_val(self) -> Result<Option<NextVal<'a>>, ErrorVal<'a>> {
        self.err()
    }

    fn into_native<T: TryFromBergVal<'a>>(self) -> Result<T, ErrorVal<'a>> {
        self.err()
    }

    fn try_into_native<T: TryFromBergVal<'a>>(self) -> Result<Option<T>, ErrorVal<'a>> {
        self.err()
    }

    fn into_val(self) -> BergResult<'a> {
        self.err()
    }

    fn eval_val(self) -> EvalResult<'a> {
        self.err()
    }
    fn evaluate(self) -> BergResult<'a> {
        self.err()
    }
    fn at_position(self, new_position: ExpressionErrorPosition) -> BergResult<'a> {
        self.reposition(new_position).err()
    }

    fn infix(self, operator: IdentifierIndex, right: RightOperand<'a, impl BergValue<'a>>) -> EvalResult<'a> {
        // Literally the only thing we support on errors are { } catch and { } finally
        if operator == crate::syntax::identifiers::APPLY {
            match right.get() {
                // { } catch
                Ok(RightOperand(EvalVal::Catch, _)) => return EvalVal::CatchResult(self.err()).ok(),
                // { } finally
                Ok(RightOperand(EvalVal::Finally, _)) => return EvalVal::FinallyResult(self.err()).ok(),
                _ => {}
            }
        }
        self.reposition(Left).err()
    }

    fn infix_assign(self, _operator: IdentifierIndex, _right: RightOperand<'a, impl BergValue<'a>>) -> EvalResult<'a> {
        self.reposition(Left).err()
    }

    fn postfix(self, _operator: IdentifierIndex) -> EvalResult<'a> {
        self.reposition(Left).err()
    }

    fn prefix(self, _operator: IdentifierIndex) -> EvalResult<'a> {
        self.reposition(Right).err()
    }

    fn subexpression_result(self, _boundary: ExpressionBoundary) -> EvalResult<'a> {
        self.reposition(Right).err()
    }

    fn field(self, _name: IdentifierIndex) -> EvalResult<'a> {
        self.err()
    }

    fn set_field(&mut self, _name: IdentifierIndex, _value: BergVal<'a>) -> Result<(), ErrorVal<'a>> where Self: Clone {
        self.clone().err()
    }
}

use exception::ErrorLocation;

use super::implement::*;
use crate::eval::BlockRef;
use berg_parser::identifiers::ERROR_CODE;
use berg_parser::{
    ExpressionPosition, FieldIndex, Fixity, IdentifierIndex, LiteralIndex, RawLiteralIndex,
};
use std::rc::Rc;
use std::{fmt, io};

///
/// Standard berg error.
///
/// This class is generally used to determine the type of an error, or for
/// implementors to create local errors without having to know an expression's
/// location. An Error or EvalError is needed to give it a source location that
/// can actually be reported.
///
#[derive(Debug, Clone)]
pub enum CompilerError {
    // File open errors
    SourceLoadError(SourceLoadError),
    // Code errors
    InvalidUtf8(RawLiteralIndex),
    UnsupportedCharacters(LiteralIndex),
    IdentifierStartsWithNumber(LiteralIndex),
    MissingOperand,
    AssignmentTargetMustBeIdentifier,
    RightSideOfDotMustBeIdentifier,
    OpenWithoutClose,
    CloseWithoutOpen,
    UnsupportedOperator(Box<dyn BergValue>, Fixity, IdentifierIndex),
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
    CatchWithoutFinally,
    FinallyWithoutBlock,
    FinallyBlockMustBeBlock,
    FinallyWithoutResult,
    ThrowWithoutException,

    // TODO stop boxing BergVals
    // BadOperandType(Box<EvalResult>, &'static str),
    BadOperandType(Box<dyn BergValue>, &'static str),
    PrivateField(BlockRef, IdentifierIndex),
    NoSuchPublicField(BlockRef, IdentifierIndex),
    NoSuchPublicFieldOnValue(Box<dyn BergValue>, IdentifierIndex),
    NoSuchPublicFieldOnRoot(IdentifierIndex),
    ImmutableFieldOnRoot(FieldIndex),
    ImmutableFieldOnValue(Box<dyn BergValue>, IdentifierIndex),

    // These are control values--only errors if nobody catches them.
    BreakOutsideLoop,
    ContinueOutsideLoop,
}

#[derive(Debug, Clone)]
pub enum SourceLoadError {
    // File open errors
    ///
    /// The source file to be read could not be found.
    ///
    SourceNotFound(Rc<io::Error>),

    ///
    /// There was an I/O error opening the source file. The file may or may not
    /// exist (depending on the type of the IoOpenError).
    ///
    IoOpenError(Rc<io::Error>),

    ///
    /// There was an I/O error reading the source file.
    ///
    IoReadError(Rc<io::Error>),

    ///
    /// There was an error determining the current directory.
    ///
    CurrentDirectoryError(Rc<io::Error>),

    ///
    /// A source file was more than 32-bits (4GB).
    ///
    SourceTooLarge(usize),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CompilerErrorCode {
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
    CatchWithoutFinally,
    FinallyWithoutBlock,
    FinallyBlockMustBeBlock,
    FinallyWithoutResult,
    ThrowWithoutException,

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

impl BergValue for CompilerError {}

impl EvaluatableValue for CompilerError {
    fn evaluate(self) -> BergResult
    where
        Self: Sized,
    {
        self.ok()
    }
}

impl Value for CompilerError {
    fn lazy_val(self) -> Result<BergVal, EvalException>
    where
        Self: Sized,
    {
        self.ok()
    }

    fn eval_val(self) -> EvalResult
    where
        Self: Sized,
    {
        self.ok()
    }

    fn into_native<T: TryFromBergVal>(self) -> Result<T, EvalException> {
        default_into_native(self)
    }

    fn try_into_native<T: TryFromBergVal>(self) -> Result<Option<T>, EvalException> {
        default_try_into_native(self)
    }

    fn display(&self) -> &dyn std::fmt::Display {
        self
    }
}

impl IteratorValue for CompilerError {
    fn next_val(self) -> Result<NextVal, EvalException> {
        single_next_val(self)
    }
}

impl ObjectValue for CompilerError {
    fn field(self, name: IdentifierIndex) -> EvalResult
    where
        Self: Sized,
    {
        match name {
            ERROR_CODE => (self.code() as usize).ok(),
            _ => default_field(self, name),
        }
    }

    fn set_field(
        &mut self,
        name: IdentifierIndex,
        value: BergVal,
    ) -> Result<(), EvalException> {
        match name {
            ERROR_CODE => CompilerError::ImmutableFieldOnValue(Box::new(self.clone()), name).err(),
            _ => default_set_field(self, name, value),
        }
    }
}

impl OperableValue for CompilerError {
    fn infix(
        self,
        operator: IdentifierIndex,
        right: RightOperand<impl EvaluatableValue>,
    ) -> EvalResult
    where
        Self: Sized,
    {
        default_infix(self, operator, right)
    }

    fn infix_assign(
        self,
        operator: IdentifierIndex,
        right: RightOperand<impl EvaluatableValue>,
    ) -> EvalResult
    where
        Self: Sized,
    {
        default_infix_assign(self, operator, right)
    }

    fn prefix(self, operator: IdentifierIndex) -> EvalResult
    where
        Self: Sized,
    {
        default_prefix(self, operator)
    }

    fn postfix(self, operator: IdentifierIndex) -> EvalResult
    where
        Self: Sized,
    {
        default_postfix(self, operator)
    }

    fn subexpression_result(self, boundary: ExpressionBoundary) -> EvalResult
    where
        Self: Sized,
    {
        default_subexpression_result(self, boundary)
    }
}

impl TryFromBergVal for CompilerError {
    const TYPE_NAME: &'static str = "CompilerError";
    fn try_from_berg_val(
        from: EvalVal,
    ) -> Result<Result<Self, BergVal>, EvalException> {
        match from.lazy_val()?.evaluate()? {
            BergVal::CompilerError(value) => Ok(Ok(value)),
            from => Ok(Err(from)),
        }
    }
}

impl From<CompilerError> for BergVal {
    fn from(from: CompilerError) -> Self {
        BergVal::CompilerError(from)
    }
}

impl From<CompilerError> for EvalVal {
    fn from(from: CompilerError) -> Self {
        BergVal::from(from).into()
    }
}

impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for CompilerErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::CompilerErrorCode::*;
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
            CatchWithoutFinally => "CatchWithoutFinally",
            FinallyWithoutBlock => "FinallyWithoutBlock",
            FinallyBlockMustBeBlock => "FinallyBlockMustBeBlock",
            FinallyWithoutResult => "FinallyWithoutResult",
            ThrowWithoutException => "ThrowWithoutException",
        };
        write!(f, "{}", string)
    }
}

impl fmt::Display for EvalException {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use EvalException::*;
        match self {
            Error(error) => write!(f, "{}", error),
            Thrown(error, position) => match position {
                ExpressionPosition::Expression => write!(f, "{:?}", error),
                _ => write!(f, "{:?} at {:?}", error, position),
            },
        }
    }
}

impl CompilerError {
    pub fn at_location(self, expression: impl Into<ExpressionRef>) -> Exception {
        Exception::new(self.into(), expression.into())
    }

    pub fn operand_err<T>(self, position: ExpressionPosition) -> Result<T, EvalException> {
        Err(EvalException::Thrown(self.into(), position))
    }

    pub fn code(&self) -> CompilerErrorCode {
        use self::CompilerError::*;
        match self {
            // File open errors
            SourceLoadError(error) => error.code(),
            // Expression errors
            InvalidUtf8(..) => CompilerErrorCode::InvalidUtf8,
            UnsupportedCharacters(..) => CompilerErrorCode::UnsupportedCharacters,
            IdentifierStartsWithNumber(..) => CompilerErrorCode::IdentifierStartsWithNumber,
            MissingOperand => CompilerErrorCode::MissingOperand,
            AssignmentTargetMustBeIdentifier => CompilerErrorCode::AssignmentTargetMustBeIdentifier,
            RightSideOfDotMustBeIdentifier => CompilerErrorCode::RightSideOfDotMustBeIdentifier,
            OpenWithoutClose => CompilerErrorCode::OpenWithoutClose,
            CloseWithoutOpen => CompilerErrorCode::CloseWithoutOpen,
            IfWithoutCondition => CompilerErrorCode::IfWithoutCondition,
            IfWithoutBlock => CompilerErrorCode::IfWithoutBlock,
            ElseWithoutBlock => CompilerErrorCode::ElseWithoutBlock,
            ElseWithoutIf => CompilerErrorCode::ElseWithoutIf,
            IfFollowedByNonElse => CompilerErrorCode::IfFollowedByNonElse,
            IfBlockMustBeBlock => CompilerErrorCode::IfBlockMustBeBlock,
            ElseBlockMustBeBlock => CompilerErrorCode::ElseBlockMustBeBlock,
            WhileWithoutCondition => CompilerErrorCode::WhileWithoutCondition,
            WhileWithoutBlock => CompilerErrorCode::WhileWithoutBlock,
            WhileConditionMustBeBlock => CompilerErrorCode::WhileConditionMustBeBlock,
            WhileBlockMustBeBlock => CompilerErrorCode::WhileBlockMustBeBlock,
            BreakOutsideLoop => CompilerErrorCode::BreakOutsideLoop,
            ContinueOutsideLoop => CompilerErrorCode::ContinueOutsideLoop,
            ForeachWithoutInput => CompilerErrorCode::ForeachWithoutInput,
            ForeachWithoutBlock => CompilerErrorCode::ForeachWithoutBlock,
            ForeachBlockMustBeBlock => CompilerErrorCode::ForeachBlockMustBeBlock,
            TryWithoutBlock => CompilerErrorCode::TryWithoutBlock,
            TryBlockMustBeBlock => CompilerErrorCode::TryBlockMustBeBlock,
            TryWithoutCatchOrFinally => CompilerErrorCode::TryWithoutCatchOrFinally,
            CatchWithoutBlock => CompilerErrorCode::CatchWithoutBlock,
            CatchBlockMustBeBlock => CompilerErrorCode::CatchBlockMustBeBlock,
            CatchWithoutResult => CompilerErrorCode::CatchWithoutResult,
            CatchWithoutFinally => CompilerErrorCode::CatchWithoutFinally,
            FinallyWithoutBlock => CompilerErrorCode::FinallyWithoutBlock,
            FinallyBlockMustBeBlock => CompilerErrorCode::FinallyBlockMustBeBlock,
            FinallyWithoutResult => CompilerErrorCode::FinallyWithoutResult,
            ThrowWithoutException => CompilerErrorCode::ThrowWithoutException,

            // Compile errors related to type (checker)
            UnsupportedOperator(..) => CompilerErrorCode::UnsupportedOperator,
            DivideByZero => CompilerErrorCode::DivideByZero,
            NoSuchField(..) => CompilerErrorCode::NoSuchField,
            NoSuchPublicField(..) | NoSuchPublicFieldOnValue(..) | NoSuchPublicFieldOnRoot(..) => {
                CompilerErrorCode::NoSuchPublicField
            }
            PrivateField(..) => CompilerErrorCode::PrivateField,
            FieldNotSet(..) => CompilerErrorCode::FieldNotSet,
            CircularDependency => CompilerErrorCode::CircularDependency,
            ImmutableFieldOnValue(..) | ImmutableFieldOnRoot(..) => {
                CompilerErrorCode::ImmutableField
            }
            BadOperandType(..) => CompilerErrorCode::BadOperandType,
        }
    }

    pub fn location(&self, expression: ExpressionRef) -> ErrorLocation {
        use self::CompilerError::*;
        use super::exception::ErrorLocation::*;
        match self {
            SourceLoadError(error) => error.location(expression.ast),

            MissingOperand => {
                let range = expression.ast.token_ranges
                    [expression.expression().parent_expression().root_index()]
                .clone();
                SourceRange(expression.ast, range)
            }

            UnsupportedOperator(..) => {
                let range =
                    expression.ast.token_ranges[expression.expression().root_index()].clone();
                SourceRange(expression.ast, range)
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
            | ImmutableFieldOnValue(..)
            | ImmutableFieldOnRoot(..)
            | PrivateField(..)
            | BadOperandType(..)
            | DivideByZero
            | RightSideOfDotMustBeIdentifier
            | IfWithoutCondition
            | ElseWithoutBlock
            | ElseWithoutIf
            | IfBlockMustBeBlock
            | ElseBlockMustBeBlock
            | IfFollowedByNonElse
            | IfWithoutBlock
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
            | CatchWithoutFinally
            | FinallyWithoutBlock
            | FinallyBlockMustBeBlock
            | FinallyWithoutResult
            | ThrowWithoutException => {
                ErrorLocation::SourceExpression(expression.ast, expression.root)
            }
        }
    }

    pub fn fmt_display(&self, expression: &ExpressionRef, f: &mut fmt::Formatter) -> fmt::Result {
        use CompilerError::*;
        match *self {
            SourceLoadError(ref error) => error.fmt_display(&expression.ast, f),
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
                "Division by zero is illegal. Perhaps you meant a different number on the right hand side of the '/'?"
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
                expression.expression().parent_expression()
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
                "catch statement missing a block! catch requires a block to run, such as 'try {{ 1/0 }} catch {{ :error.CompilerErrorCode }}'."
            ),
            CatchBlockMustBeBlock => write!(
                f,
                "catch block must be a block! Did you mean to add brackets here, like '{{ {} }}'?",
                expression.expression()
            ),
            CatchWithoutResult => write!(
                f,
                "catch statement must follow an expression! For example, '{{ 1/0 }} catch {{ :error.CompilerErrorCode }}'?"
            ),
            CatchWithoutFinally => write!(
                f,
                "catch must be followed by finally (or nothing)!"
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
            ThrowWithoutException => write!(
                f,
                "throw must be passed a value to throw! For example, 'throw 1'"
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
            ImmutableFieldOnValue(ref value, name) => write!(
                f,
                "'{}' on '{}' cannot be modified!",
                expression.ast.identifier_string(name),
                value.display()
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
                "The field access operator '.' must have an identifier on the right side (like \"{left}.FieldName\"): currently it is '{right}'.",
                left = expression.expression().parent_expression().left_expression(),
                right = expression.expression(),
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

impl SourceLoadError {
    pub fn code(&self) -> CompilerErrorCode {
        use self::SourceLoadError::*;
        match *self {
            // File open errors
            SourceNotFound(..) => CompilerErrorCode::SourceNotFound,
            IoOpenError(..) => CompilerErrorCode::IoOpenError,
            IoReadError(..) => CompilerErrorCode::IoReadError,
            CurrentDirectoryError(..) => CompilerErrorCode::CurrentDirectoryError,
            SourceTooLarge(..) => CompilerErrorCode::SourceTooLarge,
        }
    }

    pub fn location(&self, ast: AstRef) -> ErrorLocation {
        use self::SourceLoadError::*;
        use super::exception::ErrorLocation::*;
        match self {
            // File open errors
            CurrentDirectoryError(..) => ErrorLocation::Generic,
            SourceNotFound(..) | IoOpenError(..) | IoReadError(..) | SourceTooLarge(..) => {
                SourceOnly(ast)
            }
        }
    }

    pub fn fmt_display(&self, ast: &AstRef, f: &mut fmt::Formatter) -> fmt::Result {
        use self::SourceLoadError::*;
        match self {
            SourceNotFound(io_error) => write!(
                f,
                "Source not found {}: {}",
                ast.source.name(),
                io_error
            ),
            IoOpenError(io_error) => write!(
                f,
                "I/O error opening {}: {}",
                ast.source.name(),
                io_error
            ),
            IoReadError(io_error) => write!(
                f,
                "I/O error reading {}: {}",
                ast.source.name(),
                io_error
            ),
            CurrentDirectoryError(io_error) => write!(
                f,
                "I/O error getting current directory while opening {}: {}",
                ast.source.name(),
                io_error
            ),
            SourceTooLarge(size) => write!(
                f,
                "SourceSpec file {} too large ({} bytes): source files greater than 4GB are unsupported.",
                ast.source.name(),
                size
            ),
        }
    }
}

impl From<SourceLoadError> for CompilerError {
    fn from(from: SourceLoadError) -> Self {
        CompilerError::SourceLoadError(from)
    }
}

impl From<SourceLoadError> for BergVal {
    fn from(from: SourceLoadError) -> Self {
        BergVal::CompilerError(from.into())
    }
}

impl From<SourceLoadError> for EvalVal {
    fn from(from: SourceLoadError) -> Self {
        BergVal::from(from).into()
    }
}

impl<V: Into<BergVal>> From<V> for EvalException {
    fn from(from: V) -> Self {
        EvalException::Thrown(from.into(), ExpressionPosition::Expression)
    }
}

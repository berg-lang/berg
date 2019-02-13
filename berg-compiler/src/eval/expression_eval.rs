use crate::eval::{ScopeRef, AmbiguousSyntax, AssignmentTarget};
use crate::syntax::{
    ExpressionTreeWalker, ExpressionBoundary, ExpressionBoundaryError, ExpressionRef, ExpressionToken, IdentifierIndex, OperatorToken, Token
};
use crate::syntax::identifiers::*;
use crate::value::implement::*;
use num::BigRational;
use std::fmt;
use std::str::FromStr;

pub type ExpressionEvaluator<'p, 'a> = ExpressionTreeWalker<'p, 'a, &'p ScopeRef<'a>>;

impl<'p, 'a: 'p> From<ExpressionEvaluator<'p, 'a>> for ExpressionRef<'a> {
    fn from(from: ExpressionEvaluator<'p, 'a>) -> Self {
        ExpressionRef::new(from.scope().ast(), from.root_index())
    }
}
impl<'p, 'a: 'p> fmt::Display for ExpressionEvaluator<'p, 'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<'p, 'a: 'p> ExpressionEvaluator<'p, 'a> {
    pub fn scope(self) -> &'p ScopeRef<'a> {
        self.context()
    }
    pub fn evaluate(self) -> BergResult<'a> {
        self.delocalize_result_fully(self.evaluate_local(), ExpressionErrorPosition::Expression)
    }
    fn evaluate_local(self) -> BergResult<'a> {
        let indent = "  ".repeat(self.depth());
        println!("{}Evaluating {} ...", indent, self);
        use ErrorCode::*;
        use ExpressionToken::*;
        use OperatorToken::*;
        let result = match self.token() {
            Token::Expression(token) => match token {
                //
                // Nouns (operands)
                //

                // 1234
                IntegerLiteral(literal) => {
                    let parsed = BigRational::from_str(self.ast().literal_string(literal)).unwrap();
                    Ok(BergVal::BigRational(parsed))
                }
                // VariableName
                // TODO: make it so we don't have to clone, ya?
                FieldReference(field) => AssignmentTarget::LocalFieldReference(self.scope().clone(), field).result(),
                // VariableName
                RawIdentifier(name) => Err(ControlVal::AmbiguousSyntax(AmbiguousSyntax::RawIdentifier(name))),
                // Empty parens or empty block
                // () {}
                MissingExpression => AmbiguousSyntax::MissingExpression.result(),

                //
                // Prefix operators
                //

                // A<op>
                PrefixOperator(operator) => self.evaluate_prefix(operator),

                //
                // Groupings
                //

                // (...)
                Open { error: ExpressionBoundaryError::None, boundary, .. } => self.evaluate_open(boundary),

                // {...}, indent group
                OpenBlock {
                    error: ExpressionBoundaryError::None, index, ..
                } => Ok(self
                    .scope()
                    .create_child_block(self.inner_expression().root_index(), index)
                    .into()),

                //
                // Syntax errors
                //
                ErrorTerm(IdentifierStartsWithNumber, literal) => BergError::IdentifierStartsWithNumber(literal).err(),
                ErrorTerm(UnsupportedCharacters, literal) => BergError::UnsupportedCharacters(literal).err(),
                RawErrorTerm(InvalidUtf8, raw_literal) => BergError::InvalidUtf8(raw_literal).err(),
                // ( and { syntax errors
                Open {
                    error: ExpressionBoundaryError::OpenError, ..
                }
                | OpenBlock {
                    error: ExpressionBoundaryError::OpenError, ..
                } => self.ast().open_error().clone().err(),
                OpenBlock {
                    error: ExpressionBoundaryError::OpenWithoutClose,
                    ..
                }
                | Open {
                    error: ExpressionBoundaryError::OpenWithoutClose,
                    ..
                } => BergError::OpenWithoutClose.err(),
                OpenBlock {
                    error: ExpressionBoundaryError::CloseWithoutOpen,
                    ..
                }
                | Open {
                    error: ExpressionBoundaryError::CloseWithoutOpen,
                    ..
                } => BergError::CloseWithoutOpen.err(),
                OpenBlock {
                    error: ExpressionBoundaryError::EmptyAutoBlock,
                    ..
                }
                | Open {
                    error: ExpressionBoundaryError::EmptyAutoBlock,
                    ..
                } => AmbiguousSyntax::MissingExpression.into_result(),

                // Tokens that should have been handled elsewhere in the stack
                ErrorTerm(..) | RawErrorTerm(..) => unreachable!(),
            }
            Token::Operator(token) => match token {
                //
                // Infix operators
                //

                // A <op> B
                InfixOperator(operator) => self.evaluate_infix(operator),
                // A <op>= B
                InfixAssignment(operator) => self.evaluate_infix_assign(operator),
                // Multiline sequence:
                // A
                // B
                NewlineSequence => self.evaluate_infix(NEWLINE),
                // F Arg
                Apply => self.evaluate_infix(APPLY),

                //
                // Postfix operators
                //
                // <op>A
                PostfixOperator(operator) => self.evaluate_postfix(operator),

                // Tokens that should have been handled elsewhere in the stack
                Close { .. } | CloseBlock { .. }  => unreachable!(),
            }
        };
        println!(
            "{}Evaluated {} to {}",
            indent,
            self,
            match &result {
                Ok(value) => format!("{}", value),
                Err(error) => format!("{}", error),
            }
        );
        result
    }

    fn evaluate_open(self, boundary: ExpressionBoundary) -> BergResult<'a> {
        self.inner_expression().subexpression_result(boundary)
    }

    fn evaluate_infix(self, operator: IdentifierIndex) -> BergResult<'a> {
        self.left_expression().infix(operator, RightOperand::new(self.right_expression()))
    }

    fn evaluate_infix_assign(self, operator: IdentifierIndex) -> BergResult<'a> {
        self.left_expression().infix_assign(operator, RightOperand::new(self.right_expression()))
    }

    fn evaluate_prefix(self, operator: IdentifierIndex) -> BergResult<'a> {
        self.right_expression().prefix(operator)
    }

    fn evaluate_postfix(self, operator: IdentifierIndex) -> BergResult<'a> {
        self.left_expression().postfix(operator)
    }

    pub fn delocalize_error<T, E: From<ControlVal<'a>>>(self, error: BergError<'a>) -> Result<T, E> {
        Err(E::from(ControlVal::Error(error.at_location(self))))
    }

    fn delocalize_result_fully<T: fmt::Debug>(self, result: BergResult<'a, T>, self_position: ExpressionErrorPosition) -> BergResult<'a, T> {
        println!("delocalize({}, {:?}, {:?})", self, result, self_position);
        match result {
            Err(ControlVal::ExpressionError(error, position)) => error.at_location(self.error_location(position, self_position)).err(),
            Ok(_) | Err(ControlVal::Error(_)) | Err(ControlVal::AmbiguousSyntax(_)) => result,
        }
    }

    fn error_location(self, position: ExpressionErrorPosition, self_position: ExpressionErrorPosition) -> ExpressionEvaluator<'p, 'a> {
        use ExpressionErrorPosition::*;
        let expression_position = match self_position {
            Expression => self,
            LeftOperand | RightOperand => self.parent_expression(),
            ImmediateLeftOperand => self.next_expression()
        };
        match position {
            Expression => expression_position,
            ImmediateLeftOperand => expression_position.prev_expression(),
            LeftOperand => expression_position.left_expression(),
            RightOperand => expression_position.right_expression(),
        }
    }

    fn delocalize_result<T>(self, result: BergResult<'a, T>, relative_to: ExpressionErrorPosition) -> BergResult<'a, T> {
        match result {
            Err(ControlVal::ExpressionError(_, ExpressionErrorPosition::Expression)) => result,
            Err(ControlVal::ExpressionError(error, position)) => error.at_location(self.error_location(position, relative_to)).err(),
            _ => result,
        }
    }
}


impl<'p, 'a: 'p> BergValue<'a> for ExpressionEvaluator<'p, 'a> {
    fn into_result(self) -> BergResult<'a> {
        self.delocalize_result(self.evaluate_local(), ExpressionErrorPosition::Expression)
    }

    fn next_val(self) -> BergResult<'a, Option<NextVal<'a>>> {
        self.delocalize_result(self.evaluate_local().next_val(), ExpressionErrorPosition::Expression)
    }

    fn into_native<T: TryFromBergVal<'a>>(self) -> BergResult<'a, T> {
        self.delocalize_result(self.evaluate_local().into_native(), ExpressionErrorPosition::Expression)
    }

    fn try_into_native<T: TryFromBergVal<'a>>(self) -> BergResult<'a, Option<T>> {
        self.delocalize_result(self.evaluate_local().try_into_native(), ExpressionErrorPosition::Expression)
    }

    fn infix(self, operator: IdentifierIndex, right: RightOperand<'a, impl BergValue<'a>>) -> BergResult<'a> {
        let result = self.evaluate_local();
        if let Err(ControlVal::AmbiguousSyntax(AmbiguousSyntax::Target(target))) = result {
            // Handle <identifier>: <value>
            if operator == COLON {
                use AssignmentTarget::*;
                if let LocalFieldReference(scope, name) = target {
                    LocalFieldDeclaration(scope, name).set(self.delocalize_result_fully(right.into_result(), ExpressionErrorPosition::LeftOperand))?;
                    return BergVal::empty_tuple().ok();
                }
            }
            return self.delocalize_result(target.into_result().infix(operator, right), ExpressionErrorPosition::LeftOperand)
        }
        self.delocalize_result(result.infix(operator, right), ExpressionErrorPosition::LeftOperand)
    }

    fn infix_assign(self, operator: IdentifierIndex, right: RightOperand<'a, impl BergValue<'a>>) -> BergResult<'a> {
        let result = self.evaluate_local();
        if let Err(ControlVal::AmbiguousSyntax(AmbiguousSyntax::Target(mut target))) = result {
            if operator == EMPTY_STRING {
                target.set(self.delocalize_result_fully(right.into_result(), ExpressionErrorPosition::LeftOperand))?
            } else {
                target.update(|v| self.delocalize_result_fully(v.infix(operator, right), ExpressionErrorPosition::LeftOperand) )?
            }
            return BergVal::empty_tuple().ok()
        }
        self.delocalize_result(result.infix_assign(operator, right), ExpressionErrorPosition::LeftOperand)
    }

    fn prefix(self, operator: IdentifierIndex) -> BergResult<'a> {
        let result = self.evaluate_local();
        if operator == PLUS_PLUS || operator == DASH_DASH {
            if let Err(ControlVal::AmbiguousSyntax(AmbiguousSyntax::Target(mut target))) = result {
                target.update(|v| self.delocalize_result_fully(v.prefix(operator), ExpressionErrorPosition::RightOperand))?;
                return BergVal::empty_tuple().ok();
            }
        }
        self.delocalize_result(result.prefix(operator), ExpressionErrorPosition::RightOperand)
    }

    fn postfix(self, operator: IdentifierIndex) -> BergResult<'a> {
        let result = self.evaluate_local();
        if operator == PLUS_PLUS || operator == DASH_DASH {
            if let Err(ControlVal::AmbiguousSyntax(AmbiguousSyntax::Target(mut target))) = result {
                target.update(|v| self.delocalize_result_fully(v.postfix(operator), ExpressionErrorPosition::LeftOperand))?;
                return BergVal::empty_tuple().ok();
            }
        }
        self.delocalize_result(result.postfix(operator), ExpressionErrorPosition::LeftOperand)
    }

    fn subexpression_result(self, boundary: ExpressionBoundary) -> BergResult<'a> {
        self.delocalize_result(self.evaluate_local().subexpression_result(boundary), ExpressionErrorPosition::LeftOperand)
    }

    fn into_right_operand(self) -> BergResult<'a> {
        self.delocalize_result(self.evaluate_local(), ExpressionErrorPosition::Expression).into_right_operand()
    }

    fn field(self, name: IdentifierIndex) -> BergResult<'a> {
        self.delocalize_result(self.evaluate_local().field(name), ExpressionErrorPosition::Expression)
    }

    fn set_field(&mut self, name: IdentifierIndex, value: BergResult<'a>) -> BergResult<'a, ()> {
        self.delocalize_result(self.evaluate_local().set_field(name, value), ExpressionErrorPosition::Expression)
    }
}
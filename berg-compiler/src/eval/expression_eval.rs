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
        self.disambiguate(self.evaluate_local())
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
                Open { error: ExpressionBoundaryError::None, boundary, .. } => self.evaluate_inner(boundary),

                // {...}, indent group
                OpenBlock {
                    error: ExpressionBoundaryError::None, index, ..
                } => Ok(self
                    .scope()
                    .create_child_block(self.root_index(), index)
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

    pub fn evaluate_inner(self, boundary: ExpressionBoundary) -> BergResult<'a> {
        let inner = self.inner_expression().evaluate_local();
        let result = inner.subexpression_result(boundary);
        self.delocalize_errors(result)
    }

    fn evaluate_infix(self, operator: IdentifierIndex) -> BergResult<'a> {
        let left = self.left_expression().evaluate_local();
        let right = RightOperand::from(self.right_expression());
        if let Err(ControlVal::AmbiguousSyntax(AmbiguousSyntax::Target(target))) = left {
            // Handle <identifier>: <value>
            if operator == COLON {
                use AssignmentTarget::*;
                if let LocalFieldReference(scope, name) = target {
                    LocalFieldDeclaration(scope, name).set(self.disambiguate(right.into_val()))?;
                    return BergVal::empty_tuple().ok();
                }
            }
            self.delocalize_errors(target.into_val().infix(operator, right))
        } else {
            self.delocalize_errors(left.infix(operator, right))
        }
    }

    fn evaluate_infix_assign(self, operator: IdentifierIndex) -> BergResult<'a> {
        let left = self.left_expression().evaluate_local();
        let right = RightOperand::from(self.right_expression());
        if let Err(ControlVal::AmbiguousSyntax(AmbiguousSyntax::Target(mut target))) = left {
            if operator == EMPTY_STRING {
                target.set(self.disambiguate(right.into_val()))?
            } else {
                target.update(|v| self.disambiguate(v.infix(operator, right)) )?
            }
            BergVal::empty_tuple().ok()
        } else {
            self.delocalize_errors(left.infix_assign(operator, right))
        }
    }

    fn evaluate_prefix(self, operator: IdentifierIndex) -> BergResult<'a> {
        let right = self.right_expression().evaluate_local();
        let result = if operator == PLUS_PLUS || operator == DASH_DASH {
            if let Err(ControlVal::AmbiguousSyntax(AmbiguousSyntax::Target(mut target))) = right {
                target.update(|v| self.disambiguate(v.prefix(operator)))?;
                BergVal::empty_tuple().ok()
            } else {
                BergError::AssignmentTargetMustBeIdentifier.operand_err(ExpressionErrorPosition::RightOperand)
            }
        } else {
            right.prefix(operator)
        };
        self.delocalize_errors(result)
    }

    fn evaluate_postfix(self, operator: IdentifierIndex) -> BergResult<'a> {
        let left = self.left_expression().evaluate_local();
        let result = if operator == PLUS_PLUS || operator == DASH_DASH {
            if let Err(ControlVal::AmbiguousSyntax(AmbiguousSyntax::Target(mut target))) = left {
                target.update(|v| self.disambiguate(v.postfix(operator)))?;
                BergVal::empty_tuple().ok()
            } else {
                BergError::AssignmentTargetMustBeIdentifier.operand_err(ExpressionErrorPosition::LeftOperand)
            }
        } else {
            left.postfix(operator)
        };
        self.delocalize_errors(result)
    }

    ///
    /// Remove ExpressionError from result and point at real error locations.
    /// 
    fn delocalize_errors<T: fmt::Debug>(self, result: BergResult<'a, T>) -> BergResult<'a, T> {
        println!("delocalize_basic({}, {:?}) at token {:?}", self, result, self.token());
        match result {
            Err(ControlVal::ExpressionError(error, position)) => error.at_location(self.error_location(position)).err(),
            _ => result,
        }
    }

    ///
    /// Yield only values that can be used anywherethread::spawn(move || {
    /// Ok(value) and Err(ControlVal::Error(error))
    /// 
    fn disambiguate(self, result: BergResult<'a>) -> BergResult<'a> {
        println!("disambiguate({}, {:?}, {:?})", self, self.token(), result);
        match result {
            Err(ControlVal::ExpressionError(error, position)) => error.at_location(self.error_location(position)).err(),
            Err(ControlVal::AmbiguousSyntax(syntax)) => self.disambiguate(syntax.disambiguate()),
            Ok(v) => Ok(v),
            Err(ControlVal::Error(error)) => Err(ControlVal::Error(error)),
        }
    }

    fn error_location(self, position: ExpressionErrorPosition) -> ExpressionEvaluator<'p, 'a> {
        use ExpressionErrorPosition::*;
        let result = match position {
            Expression => self,
            LeftOperand => self.left_expression(),
            LeftLeft => self.left_expression().left_expression(),
            LeftRight => self.left_expression().right_expression(),
            RightOperand => self.right_expression(),
            RightLeft => self.right_expression().left_expression(),
            RightRight => self.right_expression().right_expression(),
        };
        println!("error_location({:?} [{:?}]): result ({:?})={:?}", self, self.token(), position, result);
        result
    }
}


impl<'p, 'a: 'p> BergValue<'a> for ExpressionEvaluator<'p, 'a> {
    fn into_val(self) -> BergResult<'a> {
        self.delocalize_errors(self.evaluate_local())
    }

    fn into_native<T: TryFromBergVal<'a>>(self) -> BergResult<'a, T> {
        self.delocalize_errors(self.evaluate_local().into_native())
    }

    fn try_into_native<T: TryFromBergVal<'a>>(self) -> BergResult<'a, Option<T>> {
        self.delocalize_errors(self.evaluate_local().try_into_native())
    }

    fn next_val(self) -> BergResult<'a, Option<NextVal<'a>>> {
        unreachable!()
    }

    #[allow(unused_variables)]
    fn infix(self, operator: IdentifierIndex, right: RightOperand<'a, impl BergValue<'a>>) -> BergResult<'a> {
        unreachable!()
    }

    #[allow(unused_variables)]
    fn infix_assign(self, operator: IdentifierIndex, right: RightOperand<'a, impl BergValue<'a>>) -> BergResult<'a> {
        unreachable!()
    }

    #[allow(unused_variables)]
    fn prefix(self, operator: IdentifierIndex) -> BergResult<'a> {
        unreachable!()
    }

    #[allow(unused_variables)]
    fn postfix(self, operator: IdentifierIndex) -> BergResult<'a> {
        unreachable!()
    }

    #[allow(unused_variables)]
    fn subexpression_result(self, boundary: ExpressionBoundary) -> BergResult<'a> {
        unreachable!()
    }

    #[allow(unused_variables)]
    fn field(self, name: IdentifierIndex) -> BergResult<'a> {
        unreachable!()
    }

    #[allow(unused_variables)]
    fn set_field(&mut self, name: IdentifierIndex, value: BergResult<'a>) -> BergResult<'a, ()> {
        unreachable!()
    }
}
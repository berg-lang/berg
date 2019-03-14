use crate::eval::ScopeRef;
use crate::syntax::{
    ErrorTermError, ExpressionTreeWalker, ExpressionBoundary, ExpressionBoundaryError, ExpressionRef, ExpressionToken, IdentifierIndex, OperatorToken, RawErrorTermError, TermToken, Token
};
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
        self.into_val()
    }
    fn evaluate_local(self) -> EvalResult<'a> {
        let indent = "  ".repeat(self.depth());
        println!("{}Evaluating {} ...", indent, self);
        use ExpressionToken::*;
        use TermToken::*;
        use OperatorToken::*;
        use ExpressionBoundaryError::*;
        use ErrorTermError::*;
        use RawErrorTermError::*;
        let result = match self.token() {
            Token::Expression(token) => match token {
                ExpressionToken::Term(token) => match token {
                    //
                    // Nouns (operands)
                    //

                    // 1234
                    IntegerLiteral(literal) => {
                        BigRational::from_str(self.ast().literal_string(literal)).unwrap().ok()
                    }
                    // VariableName
                    // TODO: make it so we don't have to clone, ya?
                    FieldReference(field) => AssignmentTarget::LocalFieldReference(self.scope().clone(), field).ok(),
                    // VariableName
                    RawIdentifier(name) => EvalVal::RawIdentifier(name).ok(),
                    // Empty parens or empty block
                    // () {}
                    MissingExpression => EvalVal::MissingExpression.ok(),
                    //
                    // Syntax errors
                    //
                    ErrorTerm(IdentifierStartsWithNumber, literal) => BergError::IdentifierStartsWithNumber(literal).err(),
                    ErrorTerm(UnsupportedCharacters, literal) => BergError::UnsupportedCharacters(literal).err(),
                    RawErrorTerm(InvalidUtf8, raw_literal) => BergError::InvalidUtf8(raw_literal).err(),
                }

                // A<op>
                PrefixOperator(operator) => self.evaluate_prefix(operator),

                // (...), {...}
                Open(None, boundary, delta) => if boundary.is_block() {
                    let block_index = self.ast().close_block_index(self.root_index() + delta);
                    Ok(self.scope().create_child_block(self.root_index(), block_index).into())
                } else {
                    self.evaluate_inner(boundary)
                }

                // ( and { syntax errors
                Open(Some(OpenError), ..) => self.ast().open_error().clone().err(),
                Open(Some(OpenWithoutClose), ..) => BergError::OpenWithoutClose.err(),
                Open(Some(CloseWithoutOpen), ..) => BergError::CloseWithoutOpen.err(),
            }
            Token::Operator(token) => match token {
                //
                // Infix operators
                //

                // A <op> B
                InfixOperator(operator) => self.evaluate_infix(operator),
                // A <op>= B
                InfixAssignment(operator) => self.evaluate_infix_assign(operator),

                //
                // Postfix operators
                //
                // <op>A
                PostfixOperator(operator) => self.evaluate_postfix(operator),

                // We should never be evaluating Close, only Open.
                Close(..) | CloseBlock(..) => unreachable!(),
            }
        };
        println!(
            "{}Evaluated {} to {}",
            indent,
            self,
            result.display()
        );
        result
    }

    pub fn evaluate_inner(self, boundary: ExpressionBoundary) -> EvalResult<'a> {
        let mut result = self.inner_expression().evaluate_local();
        if boundary.is_required() { result = result.subexpression_result(boundary) }
        self.delocalize_errors(result)
    }

    fn evaluate_infix(self, operator: IdentifierIndex) -> EvalResult<'a> {
        let left = self.left_expression().evaluate_local();
        let right = RightOperand::from(self.right_expression());
        self.delocalize_errors(left.infix(operator, right))
    }

    fn evaluate_infix_assign(self, operator: IdentifierIndex) -> EvalResult<'a> {
        let left = self.left_expression().evaluate_local();
        let right = RightOperand::from(self.right_expression());
        self.delocalize_errors(left.infix_assign(operator, right))
    }

    fn evaluate_prefix(self, operator: IdentifierIndex) -> EvalResult<'a> {
        let right = self.right_expression().evaluate_local();
        self.delocalize_errors(right.prefix(operator))
    }

    fn evaluate_postfix(self, operator: IdentifierIndex) -> EvalResult<'a> {
        let left = self.left_expression().evaluate_local();
        self.delocalize_errors(left.postfix(operator))
    }

    ///
    /// Remove ExpressionError from result and point at real error locations.
    /// 
    fn delocalize_errors<T: fmt::Debug>(self, result: Result<T, ErrorVal<'a>>) -> Result<T, ErrorVal<'a>> {
        match result {
            Err(ErrorVal::ExpressionError(error, position)) => error.at_location(self.error_location(position)).err(),
            Ok(_) | Err(ErrorVal::Error(_)) => result,
        }
    }

    fn skip_implicit_groups(self) -> Self {
        let mut result = self;
        while let Token::Expression(ExpressionToken::Open(_, boundary, _)) = result.token() {
            if boundary.is_required() {
                break;
            }
            result = result.inner_expression();
        }
        result
    }

    fn error_location(self, position: ExpressionErrorPosition) -> ExpressionEvaluator<'p, 'a> {
        use ExpressionErrorPosition::*;
        let expression = self.skip_implicit_groups();
        let result = match position {
            Expression => expression,
            LeftOperand => expression.left_expression(),
            LeftLeft => expression.left_expression().skip_implicit_groups().left_expression(),
            LeftRight => expression.left_expression().skip_implicit_groups().right_expression(),
            RightOperand => expression.right_expression(),
            RightLeft => expression.right_expression().skip_implicit_groups().left_expression(),
            RightRight => expression.right_expression().skip_implicit_groups().right_expression(),
        };
        let result = result.skip_implicit_groups();
        println!("error_location({:?} [{:?}]): result ({:?})={:?}", self, self.token(), position, result);
        result
    }
}


impl<'p, 'a: 'p> BergValue<'a> for ExpressionEvaluator<'p, 'a> {
    fn into_val(self) -> BergResult<'a> {
        self.delocalize_errors(self.evaluate_local().into_val())
    }
    fn eval_val(self) -> EvalResult<'a> {
        self.delocalize_errors(self.evaluate_local())
    }
    fn at_position(self, new_position: ExpressionErrorPosition) -> BergResult<'a> {
        self.into_val().at_position(new_position)
    }

    fn into_native<T: TryFromBergVal<'a>>(self) -> Result<T, ErrorVal<'a>> {
        self.delocalize_errors(self.evaluate_local().into_native())
    }

    fn try_into_native<T: TryFromBergVal<'a>>(self) -> Result<Option<T>, ErrorVal<'a>> {
       self.delocalize_errors(self.evaluate_local().try_into_native())
    }

    fn next_val(self) -> Result<Option<NextVal<'a>>, ErrorVal<'a>> {
        unreachable!()
    }

    #[allow(unused_variables)]
    fn infix(self, operator: IdentifierIndex, right: RightOperand<'a, impl BergValue<'a>>) -> EvalResult<'a> {
        unreachable!()
    }

    #[allow(unused_variables)]
    fn infix_assign(self, operator: IdentifierIndex, right: RightOperand<'a, impl BergValue<'a>>) -> EvalResult<'a> {
        unreachable!()
    }

    #[allow(unused_variables)]
    fn prefix(self, operator: IdentifierIndex) -> EvalResult<'a> {
        unreachable!()
    }

    #[allow(unused_variables)]
    fn postfix(self, operator: IdentifierIndex) -> EvalResult<'a> {
        unreachable!()
    }

    #[allow(unused_variables)]
    fn subexpression_result(self, boundary: ExpressionBoundary) -> EvalResult<'a> {
        unreachable!()
    }

    #[allow(unused_variables)]
    fn field(self, name: IdentifierIndex) -> BergResult<'a> {
        unreachable!()
    }

    #[allow(unused_variables)]
    fn set_field(&mut self, name: IdentifierIndex, value: BergVal<'a>) -> Result<(), ErrorVal<'a>> {
        unreachable!()
    }
}
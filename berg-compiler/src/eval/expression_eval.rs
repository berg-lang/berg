use crate::eval::BlockRef;
use crate::value::implement::*;
use berg_parser::identifiers::APPLY;
use berg_parser::{
    Ast, AstIndex, ErrorTermError, ExpressionBoundary, ExpressionBoundaryError, ExpressionToken,
    ExpressionTreeWalker, IdentifierIndex, OperatorToken, RawErrorTermError, TermToken, Token,
};
use num::BigRational;
use std::fmt;
use std::str::FromStr;

#[derive(Copy, Clone)]
pub struct ExpressionEvaluator<'p, 'a: 'p>(ExpressionTreeWalker<'p, &'p BlockRef<'a>>);

impl<'p, 'a: 'p> From<ExpressionEvaluator<'p, 'a>> for ExpressionRef {
    fn from(from: ExpressionEvaluator<'p, 'a>) -> Self {
        ExpressionRef::new(from.scope().ast(), from.root_index())
    }
}

impl<'p, 'a: 'p> fmt::Debug for ExpressionEvaluator<'p, 'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
impl<'p, 'a: 'p> fmt::Display for ExpressionEvaluator<'p, 'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl<'p, 'a: 'p> ExpressionEvaluator<'p, 'a> {
    pub fn new(context: &'p BlockRef<'a>, ast: &'p Ast, root: AstIndex) -> Self {
        Self(ExpressionTreeWalker::new(context, ast, root))
    }
    pub fn scope(self) -> &'p BlockRef<'a> {
        self.0.context()
    }
    pub fn depth(self) -> usize {
        self.0.depth()
    }
    fn root_index(self) -> AstIndex {
        self.0.root_index()
    }
    fn ast(self) -> &'p Ast {
        self.0.ast()
    }
    fn token(self) -> Token {
        self.0.token()
    }
    pub fn evaluate_block(self, boundary: ExpressionBoundary) -> BergResult<'a> {
        self.evaluate_inner(boundary)
            .lazy_val()
            .map_err(|e| e.at_location(self))
    }
    pub fn inner_expression(self) -> Self {
        Self(self.0.inner_expression())
    }
    pub fn left_expression(self) -> Self {
        Self(self.0.left_expression())
    }
    pub fn right_expression(self) -> Self {
        Self(self.0.right_expression())
    }
    fn evaluate_local(self) -> Result<EvalVal<'a>, Exception<'a>> {
        let indent = "  ".repeat(self.depth());
        println!("{}Evaluating {} ...", indent, self);
        use ErrorTermError::*;
        use ExpressionBoundaryError::*;
        use ExpressionToken::*;
        use OperatorToken::*;
        use RawErrorTermError::*;
        use TermToken::*;
        let result = match self.token() {
            Token::Expression(token) => match token {
                ExpressionToken::Term(token) => match token {
                    //
                    // Nouns (operands)
                    //

                    // 1234
                    IntegerLiteral(literal) => {
                        BigRational::from_str(self.ast().literal_string(literal))
                            .unwrap()
                            .ok()
                    }
                    // VariableName
                    // TODO: make it so we don't have to clone, ya?
                    FieldReference(field) => {
                        AssignmentTarget::LocalFieldReference(self.scope().clone(), field).ok()
                    }
                    // VariableName
                    RawIdentifier(name) => EvalVal::RawIdentifier(name).ok(),
                    // Empty parens or empty block
                    // () {}
                    MissingExpression => EvalVal::MissingExpression.ok(),
                    //
                    // Syntax errors
                    //
                    ErrorTerm(IdentifierStartsWithNumber, literal) => {
                        self.throw(CompilerError::IdentifierStartsWithNumber(literal))
                    }
                    ErrorTerm(UnsupportedCharacters, literal) => {
                        self.throw(CompilerError::UnsupportedCharacters(literal))
                    }
                    RawErrorTerm(InvalidUtf8, raw_literal) => {
                        self.throw(CompilerError::InvalidUtf8(raw_literal))
                    }
                },

                // A<op>
                PrefixOperator(operator) => self.evaluate_prefix(operator),

                // (...), {...}
                Open(None, boundary, delta) => {
                    if boundary.is_block() {
                        let block_index = self.ast().close_block_index(self.root_index() + delta);
                        self.scope()
                            .create_child_block(self.root_index(), block_index)
                            .ok()
                    } else {
                        self.evaluate_inner(boundary)
                    }
                }

                // ( and { syntax errors
                Open(Some(OpenWithoutClose), ..) => self.throw(CompilerError::OpenWithoutClose),
                Open(Some(CloseWithoutOpen), ..) => self.throw(CompilerError::CloseWithoutOpen),
            },
            Token::Operator(token) => match token {
                //
                // Infix operators
                //

                // A <op> B
                InfixOperator(APPLY) => self.evaluate_apply(),
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
            },
        };
        let result = result.map_err(|e| e.at_location(self));
        println!("{}Evaluated {} to {}", indent, self, result.display());
        result
    }

    fn throw<T, E: From<Exception<'a>>>(&self, error: CompilerError<'a>) -> Result<T, E> {
        Err(E::from(error.at_location(*self)))
    }

    fn evaluate_inner(self, boundary: ExpressionBoundary) -> EvalResult<'a> {
        let result = self.inner_expression().evaluate_local();
        if boundary.is_required() {
            result.subexpression_result(boundary)
        } else {
            result.res()
        }
    }

    fn evaluate_apply(self) -> EvalResult<'a> {
        let left = self.left_expression().evaluate_local();
        let right = RightOperand::from(self.right_expression().inner_expression());
        left.infix(APPLY, right)
    }

    fn evaluate_infix(self, operator: IdentifierIndex) -> EvalResult<'a> {
        let left = self.left_expression().evaluate_local();
        let right = RightOperand::from(self.right_expression());
        left.infix(operator, right)
    }

    fn evaluate_infix_assign(self, operator: IdentifierIndex) -> EvalResult<'a> {
        let left = self.left_expression().evaluate_local();
        let right = RightOperand::from(self.right_expression());
        left.infix_assign(operator, right)
    }

    fn evaluate_prefix(self, operator: IdentifierIndex) -> EvalResult<'a> {
        let right = self.right_expression().evaluate_local();
        right.prefix(operator)
    }

    fn evaluate_postfix(self, operator: IdentifierIndex) -> EvalResult<'a> {
        let left = self.left_expression().evaluate_local();
        left.postfix(operator)
    }
}

impl<'p, 'a: 'p> EvaluatableValue<'a> for ExpressionEvaluator<'p, 'a> {
    fn evaluate(self) -> BergResult<'a>
    where
        Self: Sized,
    {
        self.evaluate_local()
            .lazy_val()
            .map_err(|e| e.at_location(self))
            .evaluate()
    }
}

impl<'p, 'a: 'p> Value<'a> for ExpressionEvaluator<'p, 'a> {
    fn lazy_val(self) -> Result<BergVal<'a>, EvalException<'a>>
    where
        Self: Sized,
    {
        self.evaluate_local()
            .lazy_val()
            .map_err(|e| e.at_location(self).into())
    }
    fn eval_val(self) -> EvalResult<'a>
    where
        Self: Sized,
    {
        self.evaluate_local()
            .eval_val()
            .map_err(|e| e.at_location(self).into())
    }

    fn into_native<T: TryFromBergVal<'a>>(self) -> Result<T, EvalException<'a>> {
        self.evaluate_local()
            .into_native()
            .map_err(|e| e.at_location(self).into())
    }

    fn try_into_native<T: TryFromBergVal<'a>>(self) -> Result<Option<T>, EvalException<'a>> {
        self.evaluate_local()
            .try_into_native()
            .map_err(|e| e.at_location(self).into())
    }
    fn display(&self) -> &dyn fmt::Display {
        self
    }
}

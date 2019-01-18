use crate::error::{BergError, BergResult, Error, EvalResult, Raw, TakeError};
use crate::eval::{ExpressionFormatter, ScopeRef};
use crate::syntax::identifiers::{
    APPLY, COLON, COMMA, DASH_DASH, DOT, EMPTY_STRING, NEWLINE, PLUS_PLUS, SEMICOLON,
};
use crate::syntax::{
    AstRef, Expression, ExpressionBoundaryError, FieldIndex,
    IdentifierIndex, Operand, OperandPosition, Token,
};
use crate::util::try_from::TryFrom;
use crate::util::type_name::TypeName;
use crate::value::{BergVal, BergValue};
use num::BigRational;
use std::str::FromStr;

pub trait ExpressionEval: Sized {
    fn evaluate<'a>(self, scope: &mut ScopeRef<'a>, ast: &AstRef<'a>) -> BergResult<'a>;
    fn expression(&self) -> Expression;
    fn infix<'a>(
        self,
        operator: IdentifierIndex,
        scope: &mut ScopeRef<'a>,
        right: Operand,
        ast: &AstRef<'a>,
    ) -> EvalResult<'a> {
        self.evaluate(scope, ast)?.infix(operator, scope, right, ast)
    }
    fn postfix<'a>(
        self,
        operator: IdentifierIndex,
        scope: &mut ScopeRef<'a>,
        ast: &AstRef<'a>,
    ) -> EvalResult<'a> {
        self.evaluate(scope, ast)?.postfix(operator, scope)
    }
    fn prefix<'a>(
        self,
        operator: IdentifierIndex,
        scope: &mut ScopeRef<'a>,
        ast: &AstRef<'a>,
    ) -> EvalResult<'a> {
        self.evaluate(scope, ast)?.prefix(operator, scope)
    }
    fn result<'a>(
        self,
        scope: &mut ScopeRef<'a>,
        ast: &AstRef<'a>
    ) -> BergResult<'a> {
        self.evaluate(scope, ast)?.result(scope)
    }
    fn result_to<'a, T: TypeName + TryFrom<BergVal<'a>, Error = BergVal<'a>>>(
        self,
        scope: &mut ScopeRef<'a>,
        ast: &AstRef<'a>
    ) -> BergResult<'a, T> {
        let expression = self.expression();
        self.evaluate(scope, ast)?
            .result_to::<T>(scope)
            .take_error(ast, expression)
    }
}

pub trait OperandEval: Sized {
    fn evaluate<'a>(self, scope: &mut ScopeRef<'a>, ast: &AstRef<'a>) -> BergResult<'a>;
    fn expression(&self) -> Expression;
    fn infix<'a>(
        self,
        operator: IdentifierIndex,
        scope: &mut ScopeRef<'a>,
        right: Operand,
        ast: &AstRef<'a>,
    ) -> EvalResult<'a> {
        self.evaluate(scope, ast)?.infix(operator, scope, right, ast)
    }
    fn postfix<'a>(
        self,
        operator: IdentifierIndex,
        scope: &mut ScopeRef<'a>,
        ast: &AstRef<'a>,
    ) -> EvalResult<'a> {
        self.evaluate(scope, ast)?.postfix(operator, scope)
    }
    fn prefix<'a>(
        self,
        operator: IdentifierIndex,
        scope: &mut ScopeRef<'a>,
        ast: &AstRef<'a>,
    ) -> EvalResult<'a> {
        self.evaluate(scope, ast)?.prefix(operator, scope)
    }
    fn result<'a>(
        self,
        scope: &mut ScopeRef<'a>,
        ast: &AstRef<'a>
    ) -> BergResult<'a> {
        self.evaluate(scope, ast)?.result(scope)
    }
    fn result_to<'a, T: TypeName + TryFrom<BergVal<'a>, Error = BergVal<'a>>>(
        self,
        scope: &mut ScopeRef<'a>,
        ast: &AstRef<'a>,
    ) -> EvalResult<'a, T>;
}

#[derive(Debug, Clone)]
enum AssignmentTarget<'a> {
    Local(FieldIndex, Expression),
    DeclareLocal(FieldIndex, Expression),
    Object(BergVal<'a>, IdentifierIndex, Operand, Expression),
}

impl ExpressionEval for Expression {
    fn evaluate<'a>(self, scope: &mut ScopeRef<'a>, ast: &AstRef<'a>) -> BergResult<'a> {
        println!("Evaluate {} ...", ExpressionFormatter(self, ast));
        use crate::error::ErrorCode::*;
        use crate::syntax::ExpressionBoundaryError::*;
        use crate::syntax::Token::*;
        let result = match *self.token(ast) {
            //
            // Nouns (operands)
            //

            // 1234
            IntegerLiteral(literal) => {
                let parsed = BigRational::from_str(ast.literal_string(literal)).unwrap();
                Ok(BergVal::BigRational(parsed))
            }
            // VariableName
            FieldReference(field) => scope.local_field(field, ast).take_error(ast, self),
            // VariableName
            RawIdentifier(name) => Ok(name.into()),
            // Empty parens or empty block
            // () {}
            MissingExpression => Ok(BergVal::Nothing),

            //
            // Infix operators
            //

            // A; B
            InfixOperator(SEMICOLON) => self.evaluate_semicolon(scope, ast),
            // Field: Value
            InfixOperator(COLON) => self.evaluate_colon(scope, ast),
            // A, B, C, D
            InfixOperator(COMMA) => self.evaluate_comma(scope, ast),
            // A <op> B
            InfixOperator(operator) => self.evaluate_infix(operator, scope, ast),
            // A <op>= B
            InfixAssignment(operator) => self.evaluate_infix_assign(operator, scope, ast),
            // Multiline sequence:
            // A
            // B
            NewlineSequence => self.evaluate_infix(NEWLINE, scope, ast),
            // F Arg
            Apply => self.evaluate_infix(APPLY, scope, ast),

            //
            // Prefix operators
            //

            // A++
            PrefixOperator(PLUS_PLUS) => self.evaluate_prefix_assign(PLUS_PLUS, scope, ast),
            // A--
            PrefixOperator(DASH_DASH) => self.evaluate_prefix_assign(DASH_DASH, scope, ast),
            // A:
            PrefixOperator(COLON) => self.evaluate_declare(scope, ast),
            // A<op>
            PrefixOperator(operator) => self.evaluate_prefix(operator, scope, ast),

            PostfixOperator(PLUS_PLUS) => self.evaluate_postfix_assign(PLUS_PLUS, scope, ast),
            PostfixOperator(DASH_DASH) => self.evaluate_postfix_assign(DASH_DASH, scope, ast),
            PostfixOperator(operator) => self.evaluate_postfix(operator, scope, ast),

            //
            // Syntax errors
            //
            ErrorTerm(IdentifierStartsWithNumber, literal) => {
                BergError::IdentifierStartsWithNumber(literal).take_error(ast, self)
            }
            ErrorTerm(UnsupportedCharacters, literal) => {
                BergError::UnsupportedCharacters(literal).take_error(ast, self)
            }
            RawErrorTerm(InvalidUtf8, raw_literal) => {
                BergError::InvalidUtf8(raw_literal).take_error(ast, self)
            }
            // ( and { syntax errors
            Open {
                error: OpenError, ..
            }
            | OpenBlock {
                error: OpenError, ..
            } => ast.open_error().clone().take_error(ast, self),
            OpenBlock {
                error: ExpressionBoundaryError::OpenWithoutClose,
                ..
            }
            | Open {
                error: ExpressionBoundaryError::OpenWithoutClose,
                ..
            } => BergError::OpenWithoutClose.take_error(ast, self),
            OpenBlock {
                error: ExpressionBoundaryError::CloseWithoutOpen,
                ..
            }
            | Open {
                error: ExpressionBoundaryError::CloseWithoutOpen,
                ..
            } => BergError::CloseWithoutOpen.take_error(ast, self),

            //
            // Groupings
            //

            // (...)
            Open { error: None, .. } => self.inner_expression(ast).evaluate(scope, ast),

            // {...}, indent group
            OpenBlock {
                error: None, index, ..
            } => Ok(scope
                .create_child_block(self.inner_expression(ast), index)
                .into()),

            // Tokens that should have been handled elsewhere in the stack
            Close { .. } | CloseBlock { .. } | ErrorTerm(..) | RawErrorTerm(..) => unreachable!(),
        };
        println!("Result of {}: {:?}", ExpressionFormatter(self, ast), result);
        result
    }

    fn expression(&self) -> Expression {
        *self
    }
}

impl Expression {
    fn evaluate_semicolon<'a>(self, scope: &mut ScopeRef<'a>, ast: &AstRef<'a>) -> BergResult<'a> {
        let left = self.left_operand(ast)?;
        let left_value = left.evaluate(scope, ast)?;

        // If the left hand side is a semicolon with a missing expression between,
        // raise MissingExpression.
        if let BergVal::Nothing = left_value {
            if left.operator() == self.operator() - 2 {
                if let Token::InfixOperator(SEMICOLON) = *left.token(ast) {
                    let immediate_left = Expression(left.operator() + 1);
                    if let Token::MissingExpression = *immediate_left.token(ast) {
                        return BergError::MissingOperand.take_error(ast, self);
                    }
                }
            }
        }

        let right = Operand {
            position: OperandPosition::Right,
            expression: self.right_expression(ast),
        };
        left_value
            .infix(SEMICOLON, scope, right, ast)
            .take_error(ast, self)
    }

    fn evaluate_comma<'a>(self, scope: &mut ScopeRef<'a>, ast: &AstRef<'a>) -> BergResult<'a> {
        let mut comma = self;
        let mut vec: Vec<BergVal<'a>> = Vec::new();
        while let Token::InfixOperator(COMMA) = comma.token(ast) {
            assert!(
                match comma.left_expression(ast).token(ast) {
                    Token::InfixOperator(COMMA) => false,
                    _ => true,
                },
                "Malformed source tree: comma on the left hand side of a comma!"
            );
            vec.push(comma.left_operand(ast)?.evaluate(scope, ast)?);
            comma = comma.right_expression(ast);
        }
        // Push the final element 1,2,3
        //                            ^
        // (unless it's a trailing comma: (1,2,)
        if let Token::MissingExpression = comma.token(ast) {
        } else {
            vec.push(comma.evaluate(scope, ast)?)
        }
        Ok(vec.into())
    }

    fn evaluate_infix<'a>(
        self,
        operator: IdentifierIndex,
        scope: &mut ScopeRef<'a>,
        ast: &AstRef<'a>,
    ) -> BergResult<'a> {
        let left = self.left_operand(ast)?;
        let right = self.right_operand(ast)?;
        left.infix(operator, scope, right, ast)
            .take_error(ast, self)
    }

    fn evaluate_infix_assign<'a>(
        self,
        operator: IdentifierIndex,
        scope: &mut ScopeRef<'a>,
        ast: &AstRef<'a>,
    ) -> BergResult<'a> {
        let mut target =
            AssignmentTarget::from_expression(self.left_operand(ast)?.expression, scope, ast)?;
        let right = self.right_operand(ast)?;
        let value = match operator {
            EMPTY_STRING => {
                target.initialize(scope, ast)?;
                right.evaluate(scope, ast)
            }
            _ => target
                .get(scope, ast)?
                .infix(operator, scope, right, ast)
                .take_error(ast, self),
        };
        target.set(value, scope, ast)
    }

    fn evaluate_colon<'a>(self, scope: &mut ScopeRef<'a>, ast: &AstRef<'a>) -> BergResult<'a> {
        // Declare the variable so it can self-reference if needed.
        let mut target =
            AssignmentTarget::from_expression(self.left_operand(ast)?.expression, scope, ast)?
                .in_declaration();
        target.declare(scope, ast)?;

        // Because the right operand of colon is a block, we want to test if it has a MissingExpression Right Now.
        let right = self.right_operand(ast)?;
        assert!(match *right.expression.token(ast) {
            Token::OpenBlock { .. } => true,
            _ => false,
        });
        if let Token::MissingExpression = *right.expression.inner_expression(ast).token(ast) {
            return BergError::MissingOperand.take_error(ast, self);
        }

        // Now just evaluate and assign!
        let value = self.right_operand(ast)?.evaluate(scope, ast);
        target.set(value, scope, ast)
    }

    fn evaluate_prefix<'a>(
        self,
        operator: IdentifierIndex,
        scope: &mut ScopeRef<'a>,
        ast: &AstRef<'a>,
    ) -> BergResult<'a> {
        let operand = self.prefix_operand(ast)?;
        operand.prefix(operator, scope, ast).take_error(ast, self)
    }

    fn evaluate_postfix<'a>(
        self,
        operator: IdentifierIndex,
        scope: &mut ScopeRef<'a>,
        ast: &AstRef<'a>,
    ) -> BergResult<'a> {
        let operand = self.postfix_operand(ast)?;
        operand.postfix(operator, scope, ast).take_error(ast, self)
    }

    fn evaluate_prefix_assign<'a>(
        self,
        operator: IdentifierIndex,
        scope: &mut ScopeRef<'a>,
        ast: &AstRef<'a>,
    ) -> BergResult<'a> {
        let operand = self.prefix_operand(ast)?;
        let mut target = AssignmentTarget::from_expression(operand.expression, scope, ast)?;
        target.initialize(scope, ast)?;
        let value = operand.prefix(operator, scope, ast).take_error(ast, self);
        target.set(value, scope, ast)
    }

    fn evaluate_postfix_assign<'a>(
        self,
        operator: IdentifierIndex,
        scope: &mut ScopeRef<'a>,
        ast: &AstRef<'a>,
    ) -> BergResult<'a> {
        let operand = self.postfix_operand(ast)?;
        let mut target = AssignmentTarget::from_expression(operand.expression, scope, ast)?;
        target.initialize(scope, ast)?;
        let value = operand.postfix(operator, scope, ast).take_error(ast, self);
        target.set(value, scope, ast)
    }

    fn evaluate_declare<'a>(self, scope: &mut ScopeRef<'a>, ast: &AstRef<'a>) -> BergResult<'a> {
        let mut target = AssignmentTarget::from_expression(self, scope, ast)?;
        target.declare(scope, ast)?;
        target.get(scope, ast)
    }

    fn operand<'a>(
        self,
        position: OperandPosition,
        ast: &AstRef<'a>,
    ) -> Result<Operand, Error<'a>> {
        let expression = position.get(self, ast);
        match *expression.token(ast) {
            Token::MissingExpression => BergError::MissingOperand.take_error(ast, self),
            _ => Ok(Operand {
                expression,
                position,
            }),
        }
    }

    fn left_operand<'a>(self, ast: &AstRef<'a>) -> Result<Operand, Error<'a>> {
        self.operand(OperandPosition::Left, ast)
    }
    fn right_operand<'a>(self, ast: &AstRef<'a>) -> Result<Operand, Error<'a>> {
        self.operand(OperandPosition::Right, ast)
    }
    fn prefix_operand<'a>(self, ast: &AstRef<'a>) -> Result<Operand, Error<'a>> {
        self.operand(OperandPosition::PrefixOperand, ast)
    }
    fn postfix_operand<'a>(self, ast: &AstRef<'a>) -> Result<Operand, Error<'a>> {
        self.operand(OperandPosition::PostfixOperand, ast)
    }
}

impl<'a> AssignmentTarget<'a> {
    fn from_expression(
        expression: Expression,
        scope: &mut ScopeRef<'a>,
        ast: &AstRef<'a>,
    ) -> BergResult<'a, AssignmentTarget<'a>> {
        use crate::syntax::Token::*;
        match *expression.token(ast) {
            FieldReference(field) => Ok(AssignmentTarget::Local(field, expression)),
            PrefixOperator(COLON) => {
                let colon_operand = expression.prefix_operand(ast)?;
                match *colon_operand.token(ast) {
                    FieldReference(field) => Ok(AssignmentTarget::DeclareLocal(
                        field,
                        colon_operand.expression,
                    )),
                    _ => BergError::AssignmentTargetMustBeIdentifier
                        .take_error(ast, colon_operand.expression),
                }
            }
            Open {
                error: ExpressionBoundaryError::None,
                ..
            } => AssignmentTarget::from_expression(expression.inner_expression(ast), scope, ast),
            InfixOperator(DOT) => {
                let right = expression.right_operand(ast)?;
                match *right.token(ast) {
                    RawIdentifier(name) => {
                        let object = expression.left_operand(ast)?.evaluate(scope, ast)?;
                        Ok(AssignmentTarget::Object(object, name, right, expression))
                    }
                    _ => BergError::AssignmentTargetMustBeIdentifier
                        .take_error(ast, right.expression),
                }
            }
            _ => BergError::AssignmentTargetMustBeIdentifier.take_error(ast, expression),
        }
    }

    fn in_declaration(self) -> Self {
        use crate::eval::expression_eval::AssignmentTarget::*;
        match self {
            Local(field, expression) => DeclareLocal(field, expression),
            value => value,
        }
    }

    fn initialize(&self, scope: &mut ScopeRef<'a>, ast: &AstRef<'a>) -> BergResult<'a, ()> {
        use crate::eval::expression_eval::AssignmentTarget::*;
        match *self {
            DeclareLocal(field, expression) | Local(field, expression) => scope
                .bring_local_field_into_scope(field, ast)
                .take_error(ast, expression),
            Object(..) => Ok(()),
        }
    }

    fn get(&mut self, scope: &mut ScopeRef<'a>, ast: &AstRef<'a>) -> BergResult<'a> {
        use crate::eval::expression_eval::AssignmentTarget::*;
        use crate::syntax::identifiers::DOT;
        self.initialize(scope, ast)?;
        match *self {
            DeclareLocal(field, expression) | Local(field, expression) => {
                scope.local_field(field, ast).take_error(ast, expression)
            }
            Object(ref object, _, right, expression) => {
                // Infix consumes values, but we still need the object around, so we clone the obj (it's cheap at the moment, a reference or primitive)
                let object = object.clone();
                object
                    .infix(DOT, scope, right, ast)
                    .take_error(ast, expression)
            }
        }
    }

    fn set(
        &mut self,
        value: BergResult<'a>,
        scope: &mut ScopeRef<'a>,
        ast: &AstRef<'a>,
    ) -> BergResult<'a> {
        use crate::eval::expression_eval::AssignmentTarget::*;
        match *self {
            Local(field, expression) | DeclareLocal(field, expression) => scope
                .set_local_field(field, value, ast)
                .take_error(ast, expression)?,
            Object(ref mut object, name, _, expression) => {
                object.set_field(name, value).take_error(ast, expression)?
            }
        }
        // If it's a declaration, declare it public now that it's been set.
        self.declare(scope, ast)?;
        Ok(BergVal::Nothing)
    }

    fn declare(&mut self, scope: &mut ScopeRef<'a>, ast: &AstRef<'a>) -> BergResult<'a> {
        use crate::eval::expression_eval::AssignmentTarget::*;
        match *self {
            DeclareLocal(field, expression) => scope
                .declare_field(field, ast)
                .take_error(ast, expression)?,
            Local(..) | Object(..) => {}
        }
        Ok(BergVal::Nothing)
    }
}

impl OperandEval for Operand {
    fn evaluate<'a>(self, scope: &mut ScopeRef<'a>, ast: &AstRef<'a>) -> BergResult<'a> {
        self.expression.evaluate(scope, ast)
    }
    fn expression(&self) -> Expression {
        self.expression
    }
    fn result_to<'a, T: TypeName + TryFrom<BergVal<'a>, Error = BergVal<'a>>>(
        self,
        scope: &mut ScopeRef<'a>,
        ast: &AstRef<'a>,
    ) -> EvalResult<'a, T> {
        let value = self.result(scope, ast)?;
        match value.downcast::<T>() {
            Err(Raw(BergError::BadType(value, expected_type))) => {
                BergError::BadOperandType(self.position, value, expected_type).err()
            }
            result => result
        }
    }

}


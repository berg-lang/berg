pub mod checker_type;

use compiler::source_data::ByteIndex;
use std::ops::Range;
use ast::{IdentifierIndex,LiteralIndex};
use ast::expression::Expression;
use ast::identifiers::*;
use ast::token::Token;
use ast::token::ExpressionBoundary::*;
use ast::token::Token::*;
use checker::OperandPosition::*;
use checker::checker_type::Type;
use checker::checker_type::Type::*;
use compiler::Compiler;
use compiler::source_data::{ParseData,SourceIndex};
use compiler::compile_errors::*;
use fnv::FnvHashMap;
use num::{BigRational,One,Zero};
use std::fmt;
use std::fmt::{Formatter,Display};
use std::str::FromStr;

pub(super) fn check<'ch,'c:'ch>(
    parse_data: &'ch ParseData,
    compiler: &'ch Compiler<'c>,
    source: SourceIndex,
) -> Type {
    let scope = Scope::with_keywords();
    let mut checker = Checker { compiler, source, parse_data, scope };
    let value = checker.check(Expression::from_source(parse_data));
    if value == Missing { return Nothing; }
    value
}

#[derive(Debug)]
struct Scope {
    properties: FnvHashMap<IdentifierIndex,Type>,
}

impl Scope {
    fn with_keywords() -> Self {
        let mut scope = Scope { properties: Default::default() };
        scope.set(TRUE, Boolean(true));
        scope.set(FALSE, Boolean(false));
        scope
    }
    fn get(&self, name: IdentifierIndex) -> Option<&Type> {
        self.properties.get(&name)
    }
    fn set(&mut self, name: IdentifierIndex, value: Type) {
        self.properties.insert(name, value);
    }
}

#[derive(Debug)]
struct Checker<'ch,'c:'ch> {
    compiler: &'ch Compiler<'c>,
    source: SourceIndex,
    parse_data: &'ch ParseData,
    scope: Scope,
}

#[derive(Debug,Copy,Clone,PartialEq)]
pub enum OperandType {
    Any,
    Number,
    Boolean,
    Integer,
}

#[derive(Debug,Copy,Clone,PartialEq)]
enum OperandPosition {
    LeftOperand,
    RightOperand,
    PrefixOperand,
    PostfixOperand,
}

impl Display for OperandType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use checker::OperandType::*;
        let string = match *self {
            Any => "any",
            Number => "number",
            Boolean => "boolean",
            Integer => "integer",
        };
        write!(f, "{}", string)
    }
}

impl OperandType {
    fn matches(self, value: &Type) -> bool {
        match (self, value) {
            (OperandType::Any, _) => true,
            (OperandType::Number,&Rational(_)) => true,
            (OperandType::Integer,&Rational(ref value)) if value.is_integer() => true,
            (OperandType::Boolean,&Boolean(_)) => true,
            (OperandType::Number,_)|(OperandType::Integer,_)|(OperandType::Boolean,_) => false,
        }
    }
}

impl OperandPosition {
    fn get(self, expression: &Expression, checker: &Checker) -> Expression {
        match self {
            LeftOperand|PostfixOperand => expression.left(&checker.parse_data),
            RightOperand|PrefixOperand => expression.right(&checker.parse_data),
        }
    }
    fn range(self, expression: &Expression, checker: &Checker) -> Range<ByteIndex> {
        self.get(expression, checker).range(&checker.parse_data)
    }
}

impl<'ch,'c:'ch> Checker<'ch,'c> {
    fn check(&mut self, expression: Expression) -> Type {
        let token = expression.token(&self.parse_data);
        println!("check({})", expression.debug_string(&self.parse_data));

        match *token {
            IntegerLiteral(literal) => self.check_integer_literal(literal),
            PropertyReference(identifier) => self.check_property_reference(identifier, expression),
            InfixOperator(identifier) => self.check_infix(identifier, expression),
            InfixAssignment(identifier) => self.check_assignment(identifier, expression),
            NewlineSequence => self.check_sequence(expression),
            PrefixOperator(identifier) => self.check_prefix(identifier, expression),
            PostfixOperator(identifier) => self.check_postfix(identifier, expression),
            Open(Parentheses,_) => self.check_parentheses(expression),
            Open(..) => self.check_open(expression),
            Close(..) => unreachable!(),
            MissingInfix => {
                self.check_operand(&expression, LeftOperand, OperandType::Any);
                self.check_operand(&expression, RightOperand, OperandType::Any);
                Error
            },
            SyntaxErrorTerm(_) => Error,
            MissingExpression => Missing,
        }
    }

    fn check_integer_literal(&self, literal: LiteralIndex) -> Type {
        let string = self.parse_data.literal_string(literal);
        let value = BigRational::from_str(string).unwrap();
        Rational(value)
    }

    fn check_property_reference(&self, identifier: IdentifierIndex, expression: Expression) -> Type {
        if let Some(value) = self.scope.get(identifier) {
            value.clone()
        } else {
            self.no_such_property_error(expression)
        }
    }

    fn check_sequence(&mut self, expression: Expression) -> Type {
        self.check_operand(&expression, LeftOperand, OperandType::Any);
        let right = expression.right(&self.parse_data);
        let result = self.check(right);
        if result == Missing {
            Nothing
        } else {
            result
        }
    }

    fn check_infix(&mut self, identifier: IdentifierIndex, expression: Expression) -> Type {
        match identifier {
            PLUS  => self.check_numeric_binary(expression, |left, right| left + right),
            DASH  => self.check_numeric_binary(expression, |left, right| left - right),
            STAR  => self.check_numeric_binary(expression, |left, right| left * right),
            SLASH => self.check_divide(expression),
            EQUAL_TO      => self.check_equal_to(expression),
            NOT_EQUAL_TO  => match self.check_equal_to(expression) { Boolean(value) => Boolean(!value), value => value },
            GREATER_THAN  => self.check_numeric_comparison(expression, |left, right| left > right),
            LESS_THAN     => self.check_numeric_comparison(expression, |left, right| left < right),
            GREATER_EQUAL => self.check_numeric_comparison(expression, |left, right| left >= right),
            LESS_EQUAL    => self.check_numeric_comparison(expression, |left, right| left <= right),
            AND_AND       => self.check_and(expression),
            OR_OR         => self.check_or(expression),
            SEMICOLON     => self.check_sequence(expression),
            _ => self.unrecognized_operator_error(expression),
        }
    }

    fn check_assignment(&mut self, identifier: IdentifierIndex, expression: Expression) -> Type {
        let value = match identifier {
            EMPTY_STRING => self.check_operand(&expression, RightOperand, OperandType::Any),
            _ => self.check_infix(identifier, expression.clone()),
        };
        self.assign_value(expression, LeftOperand, value)
    }

    fn check_prefix(&mut self, identifier: IdentifierIndex, expression: Expression) -> Type {
        match identifier {
            COLON => self.check_declaration(expression),
            PLUS => self.check_numeric_unary(expression, PrefixOperand, |operand| operand),
            DASH => self.check_numeric_unary(expression, PrefixOperand, |operand| -operand),
            EXCLAMATION_POINT => self.check_boolean_unary(expression, PrefixOperand, |operand| !operand),
            PLUS_PLUS => self.assign_integer_unary(expression, PrefixOperand, |operand| operand+BigRational::one()),
            DASH_DASH => self.assign_integer_unary(expression, PrefixOperand, |operand| operand-BigRational::one()),
            _ => self.unrecognized_operator_error(expression),
        }
    }

    fn check_postfix(&mut self, identifier: IdentifierIndex, expression: Expression) -> Type {
        match identifier {
            PLUS_PLUS => self.assign_integer_unary(expression, PostfixOperand, |operand| operand+BigRational::one()),
            DASH_DASH => self.assign_integer_unary(expression, PostfixOperand, |operand| operand-BigRational::one()),
            _ => self.unrecognized_operator_error(expression),
        }
    }

    fn check_declaration(&mut self, expression: Expression) -> Type {
        if let Token::PropertyReference(property) = *expression.right(&self.parse_data).token(&self.parse_data) {
            // TODO nothing and undefined are almost certainly different.
            self.scope.set(property, Nothing);
            Nothing
        } else {
            self.unrecognized_operator_error(expression)
        }
    }

    fn check_parentheses(&mut self, expression: Expression) -> Type {
        let value = self.check_open(expression);
        if value == Missing {
            Nothing
        } else {
            value
        }
    }

    fn check_open(&mut self, expression: Expression) -> Type {
        let inner = expression.inner(&self.parse_data);
        self.check(inner)
    }

    fn check_divide(&mut self, expression: Expression) -> Type {
        let numerator = self.check_operand(&expression, LeftOperand, OperandType::Number);
        let denominator = self.check_operand(&expression, RightOperand, OperandType::Number);
        if let Rational(denominator) = denominator {
            if denominator.is_zero() {
                self.divide_by_zero_error(expression)
            } else if let Rational(numerator) = numerator {
                Rational(numerator / denominator)
            } else {
                Error
            }
        } else {
            assert!(numerator == Error || denominator == Error);
            Error
        }
    }

    fn check_equal_to(&mut self, expression: Expression) -> Type {
        let left = self.check_operand(&expression, LeftOperand, OperandType::Any);
        let right = self.check_operand(&expression, RightOperand, OperandType::Any);
        match (left, right) {
            (Rational(left), Rational(right)) => Boolean(left == right),
            (Boolean(left), Boolean(right)) => Boolean(left == right),
            (Nothing, Nothing) => Boolean(true),
            (Error,_)|(_,Error) => Error,
            (Missing,_)|(_,Missing) => unreachable!(),
            (Rational(_),_)|(Boolean(_),_)|(Nothing,_) => Boolean(false),
        }
    }

    fn check_operand(&mut self, expression: &Expression, position: OperandPosition, expected_type: OperandType) -> Type {
        let operand = position.get(expression, self);
        let value = self.check(operand.clone());
        match value {
            Missing => self.missing_operand_error(expression, position),
            Error => Error,
            _ if !expected_type.matches(&value) => self.bad_type_error(expression, position, expected_type, value),
            _ => value,
        }
    }

    fn check_numeric_binary<F: Fn(BigRational,BigRational)->BigRational>(&mut self, expression: Expression, f: F) -> Type {
        let left = self.check_operand(&expression, LeftOperand, OperandType::Number);
        let right = self.check_operand(&expression, RightOperand, OperandType::Number);
        match (left, right) {
            (Rational(left), Rational(right)) => Rational(f(left, right)),
            (Error,_)|(_,Error) => Error,
            _ => unreachable!(),
        }
    }

    fn check_numeric_comparison<F: Fn(BigRational,BigRational)->bool>(&mut self, expression: Expression, f: F) -> Type {
        let left = self.check_operand(&expression, LeftOperand, OperandType::Number);
        let right = self.check_operand(&expression, RightOperand, OperandType::Number);
        match (left, right) {
            (Rational(left), Rational(right)) => Boolean(f(left, right)),
            (Error,_)|(_,Error) => Error,
            _ => unreachable!(),
        }
    }

    fn check_and(&mut self, expression: Expression) -> Type {
        let left = self.check_operand(&expression, LeftOperand, OperandType::Boolean);
        let right = self.check_operand(&expression, RightOperand, OperandType::Boolean);
        match (left, right) {
            (Error, _) | (_, Error) => Error,
            (Boolean(a), Boolean(b)) => Boolean(a && b),
            _ => unreachable!(),
        }
    }

    fn check_or(&mut self, expression: Expression) -> Type {
        let left = self.check_operand(&expression, LeftOperand, OperandType::Boolean);
        let right = self.check_operand(&expression, RightOperand, OperandType::Boolean);
        match (left, right) {
            (Error, _) | (_, Error) => Error,
            (Boolean(a), Boolean(b)) => Boolean(a || b),
            _ => unreachable!(),
        }
    }

    fn check_numeric_unary<F: Fn(BigRational)->BigRational>(&mut self, expression: Expression, position: OperandPosition, f: F) -> Type {
        let operand = self.check_operand(&expression, position, OperandType::Number);
        match operand {
            Rational(operand) => Rational(f(operand)),
            Error => Error,
            _ => unreachable!(),
        }
    }

    fn check_integer_unary<F: Fn(BigRational)->BigRational>(&mut self, expression: Expression, position: OperandPosition, f: F) -> Type {
        let operand = self.check_operand(&expression, position, OperandType::Integer);
        match operand {
            Rational(operand) => Rational(f(operand)),
            Error => Error,
            _ => unreachable!(),
        }
    }

    fn check_boolean_unary<F: Fn(bool)->bool>(&mut self, expression: Expression, position: OperandPosition, f: F) -> Type {
        let operand = self.check_operand(&expression, position, OperandType::Boolean);
        match operand {
            Boolean(operand) => Boolean(f(operand)),
            Error => Error,
            _ => unreachable!(),
        }
    }

    fn assign_integer_unary<F: Fn(BigRational)->BigRational>(&mut self, expression: Expression, position: OperandPosition, f: F) -> Type {
        let value = self.check_integer_unary(expression.clone(), position, f);
        self.assign_value(expression, position, value)
    }

    fn assign_value(&mut self, expression: Expression, target_position: OperandPosition, value: Type) -> Type {
        let target = target_position.get(&expression, self);
        match *target.token(&self.parse_data) {
            PrefixOperator(COLON) => {
                let token = target.right(&self.parse_data).token(&self.parse_data);
                if let PropertyReference(identifier) = *token {
                    self.scope.set(identifier, value);
                    Nothing
                } else {
                    self.invalid_target_error(&expression, target_position)
                }
            },
            PropertyReference(identifier) => {
                if self.scope.get(identifier).is_none() {
                    self.no_such_property_error(target);
                }
                self.scope.set(identifier, value);
                Nothing
            },
            Token::MissingExpression => self.missing_operand_error(&expression, target_position),
            _ => self.invalid_target_error(&expression, target_position),
        }
    }

    fn report<T: CompileError+'c>(&self, error: T) -> Type {
        self.compiler.report(error);
        Error
    }

    fn divide_by_zero_error(&self, expression: Expression) -> Type {
        self.report(DivideByZero { source: self.source, divide: self.parse_data.token_range(expression.operator()) })
    }

    fn unrecognized_operator_error(&mut self, expression: Expression) -> Type {
        let token = expression.token(self.parse_data);
        if token.has_left_operand() {
            self.check(expression.left(self.parse_data));
        }
        if token.has_right_operand() {
            self.check(expression.right(self.parse_data));
        }
        let operator = expression.operator_range(self.parse_data);
        let fixity = token.fixity();
        self.report(UnrecognizedOperator { source: self.source, operator, fixity })
    }

    fn no_such_property_error(&self, operand: Expression) -> Type {
        self.report(NoSuchProperty { source: self.source, reference: operand.range(self.parse_data) })
    }

    fn invalid_target_error(&self, expression: &Expression, position: OperandPosition) -> Type {
        let source = self.source;
        let target = position.range(&expression, self);
        let operator = self.parse_data.token_range(expression.operator());
        match position {
            LeftOperand => self.report(LeftSideOfAssignmentMustBeIdentifier { source, operator, left: target }),
            PrefixOperand => self.report(RightSideOfIncrementOrDecrementMustBeIdentifier { source, operator, right: target }),
            PostfixOperand => self.report(LeftSideOfIncrementOrDecrementMustBeIdentifier { source, operator, left: target }),
            RightOperand => unreachable!(),
        }
    }

    fn missing_operand_error(&self, expression: &Expression, position: OperandPosition) -> Type {
        let source = self.source;
        let operator = self.parse_data.token_range(expression.operator());
        match position {
            LeftOperand|PostfixOperand => self.report(MissingLeftOperand { source, operator }),
            RightOperand|PrefixOperand => self.report(MissingRightOperand { source, operator }),
        }
    }

    fn bad_type_error(&self, expression: &Expression, position: OperandPosition, expected_type: OperandType, actual_type: Type) -> Type {
        let source = self.source;
        let operand = position.range(&expression, self);
        let operator = self.parse_data.token_range(expression.operator());
        match position {
            LeftOperand|PostfixOperand => self.report(BadTypeLeftOperand { source, operator, operand, expected_type, actual_type }),
            RightOperand|PrefixOperand => self.report(BadTypeRightOperand { source, operator, operand, expected_type, actual_type }),
        }
    }
}

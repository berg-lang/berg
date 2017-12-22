pub mod checker_type;

use ast::expression::OperandPosition;
use ast::expression::OperandType;
use ast::expression::OperandPosition::*;
use source::ByteIndex;
use std::ops::Range;
use ast::{IdentifierIndex,LiteralIndex};
use ast::expression::Expression;
use ast::identifiers::*;
use ast::token::{ExpressionBoundary,Token};
use ast::token::ExpressionBoundary::*;
use ast::token::Token::*;
use checker::checker_type::Type;
use checker::checker_type::Type::*;
use compiler::Compiler;
use source::{ParseResult,SourceIndex};
use source::compile_errors::*;
use fnv::FnvHashMap;
use num::{BigRational,One,Zero};
use std::fmt;
use std::fmt::{Formatter,Display};
use std::str::FromStr;

pub(super) fn check<'ch,'c:'ch>(
    parse_result: &'ch ParseResult,
    compiler: &'ch Compiler,
    source: SourceIndex,
) -> Type {
    let scopes = vec![Scope::with_keywords()];
    let mut checker = Checker { compiler, source, parse_result, scopes };
    let root = Expression::from_source(parse_result);
    checker.check(root)
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
    fn empty() -> Self {
        Scope { properties: Default::default() }
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
    compiler: &'ch Compiler,
    source: SourceIndex,
    parse_result: &'ch ParseResult,
    scopes: Vec<Scope>,
}

impl<'ch,'c:'ch> Checker<'ch,'c> {
    fn check(&mut self, expression: Expression) -> Type {
        let token = expression.token(self.parse_result);
        println!("check({})", expression.debug_string(self.parse_result));

        match *token {
            IntegerLiteral(literal) => self.check_integer_literal(literal),
            VariableReference(identifier) => self.check_field_reference(identifier, expression),
            InfixOperator(identifier) => self.check_infix(identifier, expression),
            InfixAssignment(identifier) => self.check_assignment(identifier, expression),
            NewlineSequence => self.check_sequence(expression),
            PrefixOperator(identifier) => self.check_prefix(identifier, expression),
            PostfixOperator(identifier) => self.check_postfix(identifier, expression),
            Open{..} => unreachable!(),
            Close(boundary,..) => self.check_group(boundary, expression),
            MissingInfix => {
                self.check_operand(expression, Left, OperandType::Any);
                self.check_operand(expression, Right, OperandType::Any);
                Error
            },
            ErrorTerm(_) => Error,
            MissingExpression => Missing,
        }
    }

    fn check_integer_literal(&self, literal: LiteralIndex) -> Type {
        let string = self.parse_result.literal_string(literal);
        let value = BigRational::from_str(string).unwrap();
        Rational(value)
    }

    fn check_field_reference(&mut self, identifier: IdentifierIndex, expression: Expression) -> Type {
        let error = match self.get(identifier) {
            Some(&Nothing) => self.field_not_set_error(expression),
            Some(value) => return value.clone(),
            None => self.no_such_field_error(expression),
        };
        // NOTE: this only sets the error in local scope, suppressing it there.
        // TODO switch this to a separate hash of field access errors we want
        // to suppress, so we can merge the blocks and make better guesses as to
        // which scope the user wanted the error from.
        self.set(identifier, error.clone());
        error
    }

    fn check_sequence(&mut self, expression: Expression) -> Type {
        self.check_operand(expression, Left, OperandType::Any);
        let right = expression.right(self.parse_result);
        self.check(right)
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
            EMPTY_STRING => {
                let left = match *expression.left(self.parse_result).token(self.parse_result) {
                    MissingExpression => self.missing_operand_error(expression, Left),
                    _ => Nothing,
                };
                let right = self.check_operand(expression, Right, OperandType::Any);
                if left == Error && right != Error { left } else { right }
            },
            _ => self.check_infix(identifier, expression),
        };

        self.assign_value(expression, Left, value)
    }

    fn check_prefix(&mut self, identifier: IdentifierIndex, expression: Expression) -> Type {
        match identifier {
            COLON => self.check_expose(expression),
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

    fn check_expose(&mut self, expression: Expression) -> Type {
        let right = expression.right(self.parse_result);

        if let Token::VariableReference(field) = *right.token(self.parse_result) {
            // TODO nothing and undefined are almost certainly different.
            if self.get(field) == None {
                self.set(field, Nothing);
            }
            self.check(right)
        } else {
            self.check(right);
            self.unrecognized_operator_error(expression)
        }
    }

    fn check_group(&mut self, boundary: ExpressionBoundary, expression: Expression) -> Type {
        // Open a scope if we are at the beginning of the source or in curly braces.
        match boundary {
            Source|CurlyBraces => self.scopes.push(Scope::empty()),
            CompoundTerm|Parentheses|PrecedenceGroup => {}
        }

        let value = self.check(expression.inner(self.parse_result));

        // Close the scope we just opened.
        match boundary {
            Source|CurlyBraces => { self.scopes.pop(); },
            CompoundTerm|Parentheses|PrecedenceGroup => {},
        }

        // Empty source, empty curly braces, and empty parentheses all return "nothing"
        // when the value is missing.
        match boundary {
            Source|CurlyBraces|Parentheses if value == Missing => Nothing,
            _ => value
        }
    }

    fn check_divide(&mut self, expression: Expression) -> Type {
        let numerator = self.check_operand(expression, Left, OperandType::Number);
        let denominator = self.check_operand(expression, Right, OperandType::Number);
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
        let left = self.check_operand(expression, Left, OperandType::Any);
        let right = self.check_operand(expression, Right, OperandType::Any);
        match (left, right) {
            (Rational(left), Rational(right)) => Boolean(left == right),
            (Boolean(left), Boolean(right)) => Boolean(left == right),
            (Nothing, Nothing) => Boolean(true),
            (Error,_)|(_,Error) => Error,
            (Missing,_)|(_,Missing) => unreachable!(),
            (Rational(_),_)|(Boolean(_),_)|(Nothing,_) => Boolean(false),
        }
    }

    fn check_operand(&mut self, expression: Expression, position: OperandPosition, expected_type: OperandType) -> Type {
        let operand = position.get(expression, self);
        println!("expression {:?}, position: {:?}, operand: {:?}", expression, position, operand);
        let value = self.check(operand);
        match value {
            Missing => self.missing_operand_error(expression, position),
            Error => Error,
            _ => {
                if expected_type.matches(&value) {
                    value
                } else {
                    self.bad_type_error(expression, position, expected_type, value)
                }
            },
        }
    }

    fn check_numeric_binary<F: Fn(BigRational,BigRational)->BigRational>(&mut self, expression: Expression, f: F) -> Type {
        let left = self.check_operand(expression, Left, OperandType::Number);
        let right = self.check_operand(expression, Right, OperandType::Number);
        match (left, right) {
            (Rational(left), Rational(right)) => Rational(f(left, right)),
            (Error,_)|(_,Error) => Error,
            _ => unreachable!(),
        }
    }

    fn check_numeric_comparison<F: Fn(BigRational,BigRational)->bool>(&mut self, expression: Expression, f: F) -> Type {
        let left = self.check_operand(expression, Left, OperandType::Number);
        let right = self.check_operand(expression, Right, OperandType::Number);
        match (left, right) {
            (Rational(left), Rational(right)) => Boolean(f(left, right)),
            (Error,_)|(_,Error) => Error,
            _ => unreachable!(),
        }
    }

    fn check_and(&mut self, expression: Expression) -> Type {
        let left = self.check_operand(expression, Left, OperandType::Boolean);
        let right = self.check_operand(expression, Right, OperandType::Boolean);
        match (left, right) {
            (Error, _) | (_, Error) => Error,
            (Boolean(a), Boolean(b)) => Boolean(a && b),
            _ => unreachable!(),
        }
    }

    fn check_or(&mut self, expression: Expression) -> Type {
        let left = self.check_operand(expression, Left, OperandType::Boolean);
        let right = self.check_operand(expression, Right, OperandType::Boolean);
        match (left, right) {
            (Error, _) | (_, Error) => Error,
            (Boolean(a), Boolean(b)) => Boolean(a || b),
            _ => unreachable!(),
        }
    }

    fn check_numeric_unary<F: Fn(BigRational)->BigRational>(&mut self, expression: Expression, position: OperandPosition, f: F) -> Type {
        let operand = self.check_operand(expression, position, OperandType::Number);
        match operand {
            Rational(operand) => Rational(f(operand)),
            Error => Error,
            _ => unreachable!(),
        }
    }

    fn check_integer_unary<F: Fn(BigRational)->BigRational>(&mut self, expression: Expression, position: OperandPosition, f: F) -> Type {
        let operand = self.check_operand(expression, position, OperandType::Integer);
        match operand {
            Rational(operand) => Rational(f(operand)),
            Error => Error,
            _ => unreachable!(),
        }
    }

    fn check_boolean_unary<F: Fn(bool)->bool>(&mut self, expression: Expression, position: OperandPosition, f: F) -> Type {
        let operand = self.check_operand(expression, position, OperandType::Boolean);
        match operand {
            Boolean(operand) => Boolean(f(operand)),
            Error => Error,
            _ => unreachable!(),
        }
    }

    fn assign_integer_unary<F: Fn(BigRational)->BigRational>(&mut self, expression: Expression, position: OperandPosition, f: F) -> Type {
        let value = self.check_integer_unary(expression, position, f);
        self.assign_value(expression, position, value)
    }

    fn assign_value(&mut self, expression: Expression, target_position: OperandPosition, value: Type) -> Type {
        let target = target_position.get(expression, self);
        match *target.token(self.parse_result) {
            PrefixOperator(COLON) => {
                let token = target.right(self.parse_result).token(self.parse_result);
                if let VariableReference(identifier) = *token {
                    self.set(identifier, value);
                    Nothing
                } else {
                    self.invalid_target_error(expression, target_position)
                }
            },
            VariableReference(identifier) => {
                self.set(identifier, value);
                Nothing
            },
            // If the left side is missing an expression, THAT error is already reported.
            Token::MissingExpression => {
                assert!(value == Error);
                Error
            },
            _ => self.invalid_target_error(expression, target_position),
        }
    }

    fn report<T: CompileError+'c>(&self, error: T) -> Type {
        self.compiler.report(error);
        Error
    }

    fn divide_by_zero_error(&self, expression: Expression) -> Type {
        self.report(DivideByZero { source: self.source, divide: self.parse_result.token_range(expression.operator()) })
    }

    fn unrecognized_operator_error(&mut self, expression: Expression) -> Type {
        let token = expression.token(self.parse_result);
        if token.has_left_operand() {
            self.check(expression.left(self.parse_result));
        }
        if token.has_right_operand() {
            self.check(expression.right(self.parse_result));
        }
        let operator = expression.operator_range(self.parse_result);
        let fixity = token.fixity();
        self.report(UnrecognizedOperator { source: self.source, operator, fixity })
    }

    fn no_such_field_error(&self, operand: Expression) -> Type {
        self.report(NoSuchField { source: self.source, reference: operand.range(self.parse_result) })
    }

    fn field_not_set_error(&self, operand: Expression) -> Type {
        self.report(FieldNotSet { source: self.source, reference: operand.range(self.parse_result) })
    }

    fn invalid_target_error(&self, expression: Expression, position: OperandPosition) -> Type {
        let source = self.source;
        let target = position.range(expression, self);
        let operator = self.parse_result.token_range(expression.operator());
        match position {
            Left => self.report(LeftSideOfAssignmentMustBeIdentifier { source, operator, left: target }),
            PrefixOperand => self.report(RightSideOfIncrementOrDecrementMustBeIdentifier { source, operator, right: target }),
            PostfixOperand => self.report(LeftSideOfIncrementOrDecrementMustBeIdentifier { source, operator, left: target }),
            Right => unreachable!(),
        }
    }

    fn missing_operand_error(&self, expression: Expression, position: OperandPosition) -> Type {
        let source = self.source;
        let operator = self.parse_result.token_range(expression.operator());
        self.report(MissingOperand { source, operator, position })
    }

    fn bad_type_error(&self, expression: Expression, position: OperandPosition, expected_type: OperandType, actual_type: Type) -> Type {
        let source = self.source;
        let operand = position.range(expression, self);
        let operator = self.parse_result.token_range(expression.operator());
        self.report(BadType { source, operator, operand, expected_type, actual_type, position })
    }

    fn get(&self, name: IdentifierIndex) -> Option<&Type> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Some(value);
            }
        }
        None
    }

    fn set(&mut self, name: IdentifierIndex, value: Type) {
        for scope in self.scopes.iter_mut().rev() {
            if scope.get(name).is_some() {
                return scope.set(name, value);
            }
        }
        self.scopes.last_mut().unwrap().set(name, value);
    }
}

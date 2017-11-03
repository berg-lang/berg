pub mod checker_type;

use public::*;

use ast::{AstIndex,IdentifierIndex};
use ast::ast_walker::{AstWalkerMut,AstVisitorMut};
use ast::token::TermToken::*;
use ast::token::InfixToken::*;
use compiler::compile_errors::SourceCompileErrors;
use num::BigRational;
use num::Zero;
use std::str::FromStr;

pub(super) fn check(
    parse_data: &ParseData,
    errors: &mut SourceCompileErrors
) -> Type {
    let mut checker = Checker { errors };
    AstWalkerMut::walk(&mut checker, parse_data)
}

struct Checker<'a> {
    errors: &'a mut SourceCompileErrors,
}

use Type::*;

impl<'a> Checker<'a> {
    fn check_numeric_binary_arguments(&mut self, left: Type, right: Type, index: AstIndex, parse_data: &ParseData) -> Option<(BigRational, BigRational)> {
        match (left, right) {
            (Rational(left), Rational(right)) => Some((left, right)),
            (Error, _)|(_, Error) => None,
            (Rational(_), _) => { self.report(CompileErrorType::BadTypeRightOperand, index, parse_data); None },
            (_, Rational(_)) => { self.report(CompileErrorType::BadTypeLeftOperand, index, parse_data); None },
            (_, _) => { self.report(CompileErrorType::BadTypeBothOperands, index, parse_data); None },
        }
    }
    fn check_numeric_binary<F: Fn(BigRational,BigRational)->BigRational>(&mut self, left: Type, right: Type, index: AstIndex, parse_data: &ParseData, f: F) -> Type {
        match self.check_numeric_binary_arguments(left, right, index, parse_data) {
            Some((left, right)) => Rational(f(left, right)),
            None => Error,
        }
    }
    fn check_numeric_prefix_argument(&mut self, operand: Type, index: AstIndex, parse_data: &ParseData) -> Option<BigRational> {
        match operand {
            Rational(operand) => Some(operand),
            Error => None,
            _ => { self.report(CompileErrorType::BadTypeRightOperand, index, parse_data); None }
        }
    }
    fn check_numeric_prefix<F: Fn(BigRational)->BigRational>(&mut self, operand: Type, index: AstIndex, parse_data: &ParseData, f: F) -> Type {
        match self.check_numeric_prefix_argument(operand, index, parse_data) {
            Some(operand) => Rational(f(operand)),
            None => Error,
        }
    }
    // fn check_numeric_postfix<F: Fn(BigRational)->BigRational>(operand: Type, index: AstIndex, f: F) -> Type {
    //     match operand {
    //         Rational(operand) => Rational(f(operand)),
    //         Error => Error,
    //         _ => self.report(index, CompileErrorType::BadTypeLeftOperand),
    //     }
    // }
    fn report(&mut self, error_type: CompileErrorType, index: AstIndex, parse_data: &ParseData) -> Type {
        let range = parse_data.token_range(index);
        let string = parse_data.token_string(index);
        self.errors.report_at(error_type, range, string);
        Error
    }
}

impl<'a> AstVisitorMut<Type> for Checker<'a> {
    fn visit_term(&mut self, token: TermToken, _: AstIndex, parse_data: &ParseData) -> Type {
        match token {
            IntegerLiteral(literal) => {
                let string = parse_data.literal_string(literal);
                let value = BigRational::from_str(string).unwrap();
                Rational(value)
            },
            MissingOperand => Error,
            NoExpression => Nothing,
        }
    }

    fn visit_infix(&mut self, token: InfixToken, left: Type, right: Type, index: AstIndex, parse_data: &ParseData) -> Type {
        use ast::operators::*;
        match token {
            InfixOperator(identifier) => match identifier {
                PLUS  => self.check_numeric_binary(left, right, index, parse_data, |left, right| left + right),
                DASH  => self.check_numeric_binary(left, right, index, parse_data, |left, right| left - right),
                STAR  => self.check_numeric_binary(left, right, index, parse_data, |left, right| left * right),
                SLASH => match self.check_numeric_binary_arguments(left, right, index, parse_data) {
                    Some((_, ref right)) if right.is_zero() => {
                        self.report(CompileErrorType::DivideByZero, index, parse_data);
                        Error
                    },
                    Some((ref left, ref right)) => Rational(left / right),
                    None => Error,
                }
                _ => self.report(CompileErrorType::UnrecognizedOperator, index, parse_data),
            },
            MissingInfix => Error,
        }
    }

    fn visit_prefix(&mut self, prefix: IdentifierIndex, operand: Type, index: AstIndex, parse_data: &ParseData) -> Type {
        use ast::operators::*;
        match prefix {
            PLUS => self.check_numeric_prefix(operand, index, parse_data, |operand| operand),
            DASH => self.check_numeric_prefix(operand, index, parse_data, |operand| -operand),
            _ => self.report(CompileErrorType::UnrecognizedOperator, index, parse_data),
        }
    }

    fn visit_postfix(&mut self, postfix: IdentifierIndex, _: Type, index: AstIndex, parse_data: &ParseData) -> Type {
        match postfix {
            _ => self.report(CompileErrorType::UnrecognizedOperator, index, parse_data),
        }
    }
}


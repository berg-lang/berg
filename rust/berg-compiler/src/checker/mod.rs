pub mod checker_type;

use public::*;

use ast::{AstIndex,IdentifierIndex};
use ast::ast_walker::{AstWalkerMut,AstVisitorMut};
use ast::token::TermToken::*;
use ast::token::InfixToken::*;
use ast::operators::Operators;
use ast::operators::Operators::*;
use compiler::compile_errors::SourceCompileErrors;
use num::BigRational;
use num::Zero;
use std::str::FromStr;

pub(super) fn check<'c>(
    source_data: &SourceData<'c>,
    errors: &mut SourceCompileErrors
) -> Type {
    let mut checker = Checker { errors };
    AstWalkerMut::walk(&mut checker, source_data)
}

struct Checker<'a> {
    errors: &'a mut SourceCompileErrors,
}

use Type::*;

impl<'a> Checker<'a> {
    fn check_numeric_binary_arguments(&mut self, left: Type, right: Type, index: AstIndex, source_data: &SourceData) -> Option<(BigRational, BigRational)> {
        match (left, right) {
            (Rational(left), Rational(right)) => Some((left, right)),
            (Error, _)|(_, Error) => None,
            (Rational(_), _) => { self.report(CompileErrorType::BadTypeRightOperand, index, source_data); None },
            (_, Rational(_)) => { self.report(CompileErrorType::BadTypeLeftOperand, index, source_data); None },
            (_, _) => { self.report(CompileErrorType::BadTypeBothOperands, index, source_data); None },
        }
    }
    fn check_numeric_binary<F: Fn(BigRational,BigRational)->BigRational>(&mut self, left: Type, right: Type, index: AstIndex, source_data: &SourceData, f: F) -> Type {
        match self.check_numeric_binary_arguments(left, right, index, source_data) {
            Some((left, right)) => Rational(f(left, right)),
            None => Error,
        }
    }
    fn check_numeric_prefix_argument(&mut self, operand: Type, index: AstIndex, source_data: &SourceData) -> Option<BigRational> {
        match operand {
            Rational(operand) => Some(operand),
            Error => None,
            _ => { self.report(CompileErrorType::BadTypeRightOperand, index, source_data); None }
        }
    }
    fn check_numeric_prefix<F: Fn(BigRational)->BigRational>(&mut self, operand: Type, index: AstIndex, source_data: &SourceData, f: F) -> Type {
        match self.check_numeric_prefix_argument(operand, index, source_data) {
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
    fn report(&mut self, error_type: CompileErrorType, index: AstIndex, source_data: &SourceData) -> Type {
        let range = source_data.token_range(index);
        let string = source_data.token_string(index);
        self.errors.report_at(error_type, range, string);
        Error
    }
}

impl<'a> AstVisitorMut<Type> for Checker<'a> {
    fn visit_term(&mut self, token: TermToken, _: AstIndex, source_data: &SourceData) -> Type {
        match token {
            IntegerLiteral(literal) => {
                let string = source_data.literal_string(literal);
                let value = BigRational::from_str(string).unwrap();
                Rational(value)
            },
            MissingOperand => Error,
            NoExpression => Nothing,
        }
    }

    fn visit_infix(&mut self, token: InfixToken, left: Type, right: Type, index: AstIndex, source_data: &SourceData) -> Type {
        match token {
            InfixOperator(identifier) => match Operators::from(identifier) {
                Plus  => self.check_numeric_binary(left, right, index, source_data, |left, right| left + right),
                Dash  => self.check_numeric_binary(left, right, index, source_data, |left, right| left - right),
                Star  => self.check_numeric_binary(left, right, index, source_data, |left, right| left * right),
                Slash => match self.check_numeric_binary_arguments(left, right, index, source_data) {
                    Some((_, ref right)) if right.is_zero() => {
                        self.report(CompileErrorType::DivideByZero, index, source_data);
                        Error
                    },
                    Some((ref left, ref right)) => Rational(left / right),
                    None => Error,
                }
                _ => self.report(CompileErrorType::UnrecognizedOperator, index, source_data),
            },
            MissingInfix => Error,
        }
    }

    fn visit_prefix(&mut self, prefix: IdentifierIndex, operand: Type, index: AstIndex, source_data: &SourceData) -> Type {
        match Operators::from(prefix) {
            Plus => self.check_numeric_prefix(operand, index, source_data, |operand| operand),
            Dash => self.check_numeric_prefix(operand, index, source_data, |operand| -operand),
            _ => self.report(CompileErrorType::UnrecognizedOperator, index, source_data),
        }
    }

    fn visit_postfix(&mut self, postfix: IdentifierIndex, _: Type, index: AstIndex, source_data: &SourceData) -> Type {
        match Operators::from(postfix) {
            _ => self.report(CompileErrorType::UnrecognizedOperator, index, source_data),
        }
    }

    fn open_without_close(&mut self, _: Operators, open_index: AstIndex, _missing_close_index: AstIndex, source_data: &SourceData) {
        self.report(CompileErrorType::OpenWithoutClose, open_index, source_data);
    }

    fn close_without_open(&mut self, _: Operators, close_index: AstIndex, _missing_open_index: AstIndex, source_data: &SourceData) {
        self.report(CompileErrorType::CloseWithoutOpen, close_index, source_data);
    }
}


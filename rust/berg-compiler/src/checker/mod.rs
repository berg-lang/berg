pub mod checker_type;

use ast::{AstIndex,IdentifierIndex};
use ast::ast_walker::{AstWalkerMut,AstVisitorMut};
use ast::identifiers::*;
use ast::token::{TermToken,InfixToken};
use ast::token::TermToken::*;
use ast::token::InfixToken::*;
use checker::checker_type::Type;
use checker::checker_type::Type::*;
use compiler::Compiler;
use compiler::source_data::{ParseData,SourceIndex};
use compiler::compile_errors;
use compiler::compile_errors::CompileError;
use num::BigRational;
use num::Zero;
use std::str::FromStr;

pub(super) fn check<'ch,'c:'ch>(
    parse_data: &ParseData,
    compiler: &'ch Compiler<'c>,
    source: SourceIndex,
) -> Type {
    let mut checker = Checker { compiler, source };
    let value = AstWalkerMut::walk(&mut checker, parse_data);
    if value == Missing { return Nothing; }
    value
}

struct Checker<'ch,'c:'ch> {
    compiler: &'ch Compiler<'c>,
    source: SourceIndex,
}

impl<'ch,'c:'ch> Checker<'ch,'c> {
    fn check_numeric_binary_arguments(&mut self, left: Type, right: Type, index: AstIndex, parse_data: &ParseData) -> Option<(BigRational, BigRational)> {
        match (left, right) {
            (Rational(left), Rational(right)) => Some((left, right)),
            (Error, _)|(_, Error) => None,
            (Rational(_), right) => {
                self.report(compile_errors::BadTypeRightOperand { source: self.source(), operator: parse_data.token_ranges[index].clone(), right });
                None
            },
            (left, Rational(_)) => {
                self.report(compile_errors::BadTypeLeftOperand { source: self.source(), operator: parse_data.token_ranges[index].clone(), left });
                None
            },
            (left, right) => {
                self.report(compile_errors::BadTypeLeftOperand { source: self.source(), operator: parse_data.token_ranges[index].clone(), left });
                self.report(compile_errors::BadTypeRightOperand { source: self.source(), operator: parse_data.token_ranges[index].clone(), right });
                None
            },
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
            _ => { self.report(compile_errors::BadTypeRightOperand { source: self.source(), operator: parse_data.token_ranges[index].clone(), right: operand }); None },
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
    fn report<T: CompileError+'c>(&self, error: T) -> Type {
        self.compiler.report(error);
        Error
    }
    fn source(&self) -> SourceIndex {
        self.source
    }
}

impl<'ch,'c:'ch> AstVisitorMut<Type> for Checker<'ch,'c> {
    fn visit_term(&mut self, token: TermToken, index: AstIndex, parse_data: &ParseData) -> Type {
        match token {
            IntegerLiteral(literal) => {
                let string = parse_data.literal_string(literal);
                let value = BigRational::from_str(string).unwrap();
                Rational(value)
            },
            PropertyReference(NOTHING) => Nothing,
            PropertyReference(TRUE) => Boolean(true),
            PropertyReference(FALSE) => Boolean(false),
            PropertyReference(_) => {
                self.report(compile_errors::NoSuchProperty { source: self.source(), reference: parse_data.token_range(index) });
                Error
            },
            SyntaxErrorTerm(_) => Error,
            MissingExpression => Missing,
        }
    }

    fn visit_infix(&mut self, token: InfixToken, mut left: Type, mut right: Type, index: AstIndex, parse_data: &ParseData) -> Type {
        use ast::identifiers::*;
        if left == Missing {
            self.report(compile_errors::MissingLeftOperand { source: self.source(), operator: parse_data.token_range(index) });
            left = Error;
        }
        if right == Missing {
            self.report(compile_errors::MissingRightOperand { source: self.source(), operator: parse_data.token_range(index) });
            right = Error;
        }
        match token {
            InfixOperator(identifier) => match identifier {
                PLUS  => self.check_numeric_binary(left, right, index, parse_data, |left, right| left + right),
                DASH  => self.check_numeric_binary(left, right, index, parse_data, |left, right| left - right),
                STAR  => self.check_numeric_binary(left, right, index, parse_data, |left, right| left * right),
                SLASH => match self.check_numeric_binary_arguments(left, right, index, parse_data) {
                    Some((_, ref right)) if right.is_zero() => 
                        self.report(compile_errors::DivideByZero { source: self.source(), divide: parse_data.token_range(index) }),
                    Some((ref left, ref right)) => Rational(left / right),
                    None => Error,
                }
                _ => self.report(compile_errors::UnrecognizedOperator { source: self.source(), operator: parse_data.token_range(index) }),
            },
            NewlineSequence => if left == Error { left } else { right },
            MissingInfix => Error,
        }
    }

    fn visit_prefix(&mut self, prefix: IdentifierIndex, mut operand: Type, index: AstIndex, parse_data: &ParseData) -> Type {
        use ast::identifiers::*;
        if operand == Missing {
            self.report(compile_errors::MissingRightOperand { source: self.source(), operator: parse_data.token_range(index) });
            operand = Error;
        }
        match prefix {
            PLUS => self.check_numeric_prefix(operand, index, parse_data, |operand| operand),
            DASH => self.check_numeric_prefix(operand, index, parse_data, |operand| -operand),
            _ => self.report(compile_errors::UnrecognizedOperator { source: self.source(), operator: parse_data.token_ranges[index].clone() }),
        }
    }

    fn visit_postfix(&mut self, postfix: IdentifierIndex, _: Type, index: AstIndex, parse_data: &ParseData) -> Type {
        match postfix {
            _ => self.report(compile_errors::UnrecognizedOperator { source: self.source(), operator: parse_data.token_ranges[index].clone() }),
        }
    }

    fn visit_parentheses(&mut self, value: Type, _: AstIndex, _: AstIndex, _: &ParseData) -> Type {
        if value == Missing {
            Nothing
        } else {
            value
        }
    }
}


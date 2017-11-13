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
    fn check_equal_to(&mut self, left: Type, right: Type) -> Type {
        match (left, right) {
            (Error, _)|(_, Error) => Error,
            (Rational(left), Rational(right)) => Boolean(left == right),
            (Boolean(left), Boolean(right)) => Boolean(left == right),
            (Nothing, Nothing) => Boolean(true),
            (_, _) => Boolean(false),
        }
    }
    fn check_numeric_binary_arguments(&mut self, left: Type, right: Type, index: AstIndex, parse_data: &ParseData) -> Option<(BigRational, BigRational)> {
        match (left, right) {
            (Rational(left), Rational(right)) => Some((left, right)),
            (Error, Error)|(Rational(_), Error)|(Error, Rational(_)) => None,
            (Rational(_), right)|(Error, right) => {
                self.report(compile_errors::BadTypeRightOperand { source: self.source(), operator: parse_data.token_ranges[index].clone(), right });
                None
            },
            (left, Rational(_))|(left, Error) => {
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
    fn check_boolean_binary_arguments(&mut self, left: Type, right: Type, index: AstIndex, parse_data: &ParseData) -> Option<(bool, bool)> {
        match (left, right) {
            (Boolean(left), Boolean(right)) => Some((left, right)),
            (Error, Error)|(Boolean(_), Error)|(Error, Boolean(_)) => None,
            (Boolean(_), right)|(Error, right) => {
                self.report(compile_errors::BadTypeRightOperand { source: self.source(), operator: parse_data.token_ranges[index].clone(), right });
                None
            },
            (left, Boolean(_))|(left, Error) => {
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
    fn check_numeric_comparison<F: Fn(BigRational,BigRational)->bool>(&mut self, left: Type, right: Type, index: AstIndex, parse_data: &ParseData, f: F) -> Type {
        match self.check_numeric_binary_arguments(left, right, index, parse_data) {
            Some((left, right)) => Boolean(f(left, right)),
            None => Error,
        }
    }
    fn check_boolean_binary<F: Fn(bool,bool)->bool>(&mut self, left: Type, right: Type, index: AstIndex, parse_data: &ParseData, f: F) -> Type {
        match self.check_boolean_binary_arguments(left, right, index, parse_data) {
            Some((left, right)) => Boolean(f(left, right)),
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
    fn check_boolean_prefix_argument(&mut self, operand: Type, index: AstIndex, parse_data: &ParseData) -> Option<bool> {
        match operand {
            Boolean(operand) => Some(operand),
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
    fn check_boolean_prefix<F: Fn(bool)->bool>(&mut self, operand: Type, index: AstIndex, parse_data: &ParseData, f: F) -> Type {
        match self.check_boolean_prefix_argument(operand, index, parse_data) {
            Some(operand) => Boolean(f(operand)),
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
            if let InfixOperator(SEMICOLON) = token {
            } else {
                self.report(compile_errors::MissingRightOperand { source: self.source(), operator: parse_data.token_range(index) });
                right = Error;
            }
        }
        if let InfixOperator(SEMICOLON) = token {
            println!("Semicolon {:?} {:?}", left, right);
        }
        match token {
            InfixOperator(PLUS)  => self.check_numeric_binary(left, right, index, parse_data, |left, right| left + right),
            InfixOperator(DASH)  => self.check_numeric_binary(left, right, index, parse_data, |left, right| left - right),
            InfixOperator(STAR)  => self.check_numeric_binary(left, right, index, parse_data, |left, right| left * right),
            InfixOperator(SLASH) => match self.check_numeric_binary_arguments(left, right, index, parse_data) {
                Some((_, ref right)) if right.is_zero() => 
                    self.report(compile_errors::DivideByZero { source: self.source(), divide: parse_data.token_range(index) }),
                Some((ref left, ref right)) => Rational(left / right),
                None => Error,
            },
            InfixOperator(EQUAL_TO)      => self.check_equal_to(left, right),
            InfixOperator(NOT_EQUAL_TO)  => match self.check_equal_to(left, right) { Boolean(value) => Boolean(!value), value => value },
            InfixOperator(GREATER_THAN)  => self.check_numeric_comparison(left, right, index, parse_data, |left, right| left > right),
            InfixOperator(LESS_THAN)     => self.check_numeric_comparison(left, right, index, parse_data, |left, right| left < right),
            InfixOperator(GREATER_EQUAL) => self.check_numeric_comparison(left, right, index, parse_data, |left, right| left >= right),
            InfixOperator(LESS_EQUAL)    => self.check_numeric_comparison(left, right, index, parse_data, |left, right| left <= right),
            InfixOperator(AND_AND)       => self.check_boolean_binary(left, right, index, parse_data, |left, right| left && right),
            InfixOperator(OR_OR)         => self.check_boolean_binary(left, right, index, parse_data, |left, right| left || right),
            InfixOperator(SEMICOLON)|NewlineSequence => right,
            InfixOperator(_) => self.report(compile_errors::UnrecognizedOperator { source: self.source(), operator: parse_data.token_range(index) }),
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
            NOT => self.check_boolean_prefix(operand, index, parse_data, |operand| !operand),
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


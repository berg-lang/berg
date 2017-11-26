pub mod checker_type;

use fnv::FnvHashMap;
use ast::{AstIndex,IdentifierIndex};
use ast::ast_walker::{AstWalkerMut,AstVisitorMut};
use ast::identifiers::*;
use ast::token::{Token,TermToken,InfixToken};
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
use num::One;
use std::str::FromStr;

pub(super) fn check<'ch,'c:'ch>(
    parse_data: &ParseData,
    compiler: &'ch Compiler<'c>,
    source: SourceIndex,
) -> Type {
    let scope = Scope::with_keywords();
    let mut checker = Checker { compiler, source, scope };
    let value = AstWalkerMut::walk(&mut checker, parse_data);
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
    scope: Scope,
}

impl<'ch,'c:'ch> AstVisitorMut<Type> for Checker<'ch,'c> {
    fn visit_term(&mut self, token: TermToken, index: AstIndex, parse_data: &ParseData) -> Type {
        match token {
            IntegerLiteral(literal) => {
                let string = parse_data.literal_string(literal);
                let value = BigRational::from_str(string).unwrap();
                Rational(value)
            },
            // If we're introducing the identifier into scope right now,
            // all subsequent expressions can use it.
            PropertyDeclaration(identifier) => {
                let undefined = Undefined { reference_source: self.source(), reference_index: index };
                self.scope.set(identifier, undefined.clone());
                undefined
            },
            // If it's a target, we don't *set* it yet; the old value rules.
            PropertyDeclarationTarget(identifier) => {
                if let Some(value) = self.scope.get(identifier) {
                    value.clone()
                } else {
                    Undefined { reference_source: self.source(), reference_index: index }
                }
            },
            PropertyReference(identifier) => {
                if let Some(value) = self.scope.get(identifier) {
                    value.clone()
                } else {
                    self.report(compile_errors::NoSuchProperty { source: self.source(), reference: parse_data.token_range(index) });
                    Error
                }
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
       self.check_infix(token, left, right, index, parse_data)
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
            PLUS_PLUS => self.assign_integer_prefix(operand, index, parse_data, |operand| operand+BigRational::one()),
            DASH_DASH => self.assign_integer_prefix(operand, index, parse_data, |operand| operand-BigRational::one()),
            _ => self.report(compile_errors::UnrecognizedOperator { source: self.source(), operator: parse_data.token_ranges[index].clone() }),
        }
    }

    fn visit_postfix(&mut self, postfix: IdentifierIndex, operand: Type, index: AstIndex, parse_data: &ParseData) -> Type {
        match postfix {
            PLUS_PLUS => self.assign_integer_postfix(operand, index, parse_data, |operand| operand+BigRational::one()),
            DASH_DASH => self.assign_integer_postfix(operand, index, parse_data, |operand| operand-BigRational::one()),
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

impl<'ch,'c:'ch> Checker<'ch,'c> {
    fn check_infix(&mut self, token: InfixToken, left: Type, right: Type, index: AstIndex, parse_data: &ParseData) -> Type {
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
            InfixOperator(EQUAL_TO)      => self.check_equal_to(left, right, index, parse_data),
            InfixOperator(NOT_EQUAL_TO)  => match self.check_equal_to(left, right, index, parse_data) { Boolean(value) => Boolean(!value), value => value },
            InfixOperator(GREATER_THAN)  => self.check_numeric_comparison(left, right, index, parse_data, |left, right| left > right),
            InfixOperator(LESS_THAN)     => self.check_numeric_comparison(left, right, index, parse_data, |left, right| left < right),
            InfixOperator(GREATER_EQUAL) => self.check_numeric_comparison(left, right, index, parse_data, |left, right| left >= right),
            InfixOperator(LESS_EQUAL)    => self.check_numeric_comparison(left, right, index, parse_data, |left, right| left <= right),
            InfixOperator(AND_AND)       => self.check_boolean_binary(left, right, index, parse_data, |left, right| left && right),
            InfixOperator(OR_OR)         => self.check_boolean_binary(left, right, index, parse_data, |left, right| left || right),
            InfixOperator(ASSIGN)        => self.assign(right, index, parse_data),
            InfixOperator(ASSIGN_PLUS)   => self.assign_operation(PLUS, left, right, index, parse_data),
            InfixOperator(ASSIGN_DASH)   => self.assign_operation(DASH, left, right, index, parse_data),
            InfixOperator(ASSIGN_STAR)   => self.assign_operation(STAR, left, right, index, parse_data),
            InfixOperator(ASSIGN_SLASH)  => self.assign_operation(SLASH, left, right, index, parse_data),
            InfixOperator(ASSIGN_AND_AND)=> self.assign_operation(AND_AND, left, right, index, parse_data),
            InfixOperator(ASSIGN_OR_OR)  => self.assign_operation(OR_OR, left, right, index, parse_data),
            InfixOperator(SEMICOLON)|NewlineSequence => right,
            InfixOperator(_) => self.report(compile_errors::UnrecognizedOperator { source: self.source(), operator: parse_data.token_range(index) }),
            MissingInfix => Error,
        }
    }

    fn check_equal_to(&mut self, left: Type, right: Type, index: AstIndex, parse_data: &ParseData) -> Type {
        match (left, right) {
            (Rational(left), Rational(right)) => Boolean(left == right),
            (Boolean(left), Boolean(right)) => Boolean(left == right),
            (Nothing, Nothing)|(Undefined{..},Undefined{..}) => Boolean(true),
            (Missing,Missing) => {
                self.report(compile_errors::MissingLeftOperand { source: self.source(), operator: parse_data.token_range(index) });
                self.report(compile_errors::MissingRightOperand { source: self.source(), operator: parse_data.token_range(index) });
                Error
            },
            (Missing,_) => {
                self.report(compile_errors::MissingLeftOperand { source: self.source(), operator: parse_data.token_range(index) });
                Error
            },
            (_,Missing) => {
                self.report(compile_errors::MissingRightOperand { source: self.source(), operator: parse_data.token_range(index) });
                Error
            },
            (Error,_)|(_,Error) => Error,
            (Rational(_),_)|(Boolean(_),_)|(Nothing,_)|(Undefined{..},_) => Boolean(false),
        }
    }

    fn check_numeric_binary_arguments(&mut self, left: Type, right: Type, index: AstIndex, parse_data: &ParseData) -> Option<(BigRational, BigRational)> {
        match left {
            Error|Rational(_) => {},
            Missing => { self.report(compile_errors::MissingLeftOperand { source: self.source(), operator: parse_data.token_range(index) }); },
            // TODO this isn't the absolute best place to report against, if it comes from another source--on the other hand, it's an important place to mention
            // because it is where the undefined value was introduced.
            Undefined{reference_source,reference_index} => { self.report(compile_errors::PropertyNotSet { source: reference_source, reference: parse_data.token_ranges[reference_index].clone() }); },
            Nothing|Boolean(_) => { self.report(compile_errors::BadTypeLeftOperand { source: self.source(), operator: parse_data.token_ranges[index].clone(), left: left.clone() }); },
        }
        match right {
            Error|Rational(_) => {},
            Missing => { self.report(compile_errors::MissingRightOperand { source: self.source(), operator: parse_data.token_range(index) }); },
            // TODO this isn't the absolute best place to report against, if it comes from another source--on the other hand, it's an important place to mention
            // because it is where the undefined value was introduced to the expression.
            Undefined{reference_source,reference_index} => { self.report(compile_errors::PropertyNotSet { source: reference_source, reference: parse_data.token_ranges[reference_index].clone() }); },
            Nothing|Boolean(_) => { self.report(compile_errors::BadTypeRightOperand { source: self.source(), operator: parse_data.token_ranges[index].clone(), right: right.clone() }); },
        }
        match (left, right) {
            (Rational(left), Rational(right)) => Some((left, right)),
            _ => None
        }
    }

    fn check_boolean_binary_arguments(&mut self, left: Type, right: Type, index: AstIndex, parse_data: &ParseData) -> Option<(bool, bool)> {
        match left {
            Error|Boolean(_) => {},
            Missing => { self.report(compile_errors::MissingLeftOperand { source: self.source(), operator: parse_data.token_range(index) }); },
            // TODO this isn't the absolute best place to report against, if it comes from another source--on the other hand, it's an important place to mention
            // because it is where the undefined value was introduced.
            Undefined{reference_source,reference_index} => { self.report(compile_errors::PropertyNotSet { source: reference_source, reference: parse_data.token_ranges[reference_index].clone() }); },
            Nothing|Rational(_) => { self.report(compile_errors::BadTypeLeftOperand { source: self.source(), operator: parse_data.token_ranges[index].clone(), left: left.clone() }); },
        }
        match right {
            Error|Boolean(_) => {},
            Missing => { self.report(compile_errors::MissingRightOperand { source: self.source(), operator: parse_data.token_range(index) }); },
            // TODO this isn't the absolute best place to report against, if it comes from another source--on the other hand, it's an important place to mention
            // because it is where the undefined value was introduced to the expression.
            Undefined{reference_source,reference_index} => { self.report(compile_errors::PropertyNotSet { source: reference_source, reference: parse_data.token_ranges[reference_index].clone() }); },
            Nothing|Rational(_) => { self.report(compile_errors::BadTypeRightOperand { source: self.source(), operator: parse_data.token_ranges[index].clone(), right: right.clone() }); },
        }
        match (left, right) {
            (Boolean(left), Boolean(right)) => Some((left, right)),
            _ => None
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
            // TODO this isn't the absolute best place to report against, if it comes from another source--on the other hand, it's an important place to mention
            // because it is where the undefined value was introduced.
            Undefined{reference_source,reference_index} => { self.report(compile_errors::PropertyNotSet { source: reference_source, reference: parse_data.token_ranges[reference_index].clone() }); None },
            // TODO bad type should be on the operand itself, not the operation.
            Boolean(_)|Missing|Nothing => { self.report(compile_errors::BadTypeRightOperand { source: self.source(), operator: parse_data.token_ranges[index].clone(), right: operand }); None },
        }
    }

    fn check_integer_prefix_argument(&mut self, operand: Type, index: AstIndex, parse_data: &ParseData) -> Option<BigRational> {
        match operand {
            Rational(operand) => if operand.is_integer() { Some(operand) } else { None },
            Error => None,
            // TODO this isn't the absolute best place to report against, if it comes from another source--on the other hand, it's an important place to mention
            // because it is where the undefined value was introduced.
            Undefined{reference_source,reference_index} => { self.report(compile_errors::PropertyNotSet { source: reference_source, reference: parse_data.token_ranges[reference_index].clone() }); None },
            // TODO bad type should be on the operand itself, not the operation.
            Boolean(_)|Missing|Nothing => { self.report(compile_errors::BadTypeRightOperand { source: self.source(), operator: parse_data.token_ranges[index].clone(), right: operand }); None },
        }
    }

    fn check_integer_postfix_argument(&mut self, operand: Type, index: AstIndex, parse_data: &ParseData) -> Option<BigRational> {
        match operand {
            Rational(operand) => if operand.is_integer() { Some(operand) } else { None },
            Error => None,
            // TODO this isn't the absolute best place to report against, if it comes from another source--on the other hand, it's an important place to mention
            // because it is where the undefined value was introduced.
            Undefined{reference_source,reference_index} => { self.report(compile_errors::PropertyNotSet { source: reference_source, reference: parse_data.token_ranges[reference_index].clone() }); None },
            // TODO bad type should be on the operand itself, not the operation.
            Boolean(_)|Missing|Nothing => { self.report(compile_errors::BadTypeLeftOperand { source: self.source(), operator: parse_data.token_ranges[index].clone(), left: operand }); None },
        }
    }

    fn check_boolean_prefix_argument(&mut self, operand: Type, index: AstIndex, parse_data: &ParseData) -> Option<bool> {
        match operand {
            Boolean(operand) => Some(operand),
            Error => None,
            // TODO this isn't the absolute best place to report against, if it comes from another source--on the other hand, it's an important place to mention
            // because it is where the undefined value was introduced.
            Undefined{reference_source,reference_index} => { self.report(compile_errors::PropertyNotSet { source: reference_source, reference: parse_data.token_ranges[reference_index].clone() }); None },
            // TODO bad type should be on the operand itself, not the operation.
            Rational(_)|Missing|Nothing => { self.report(compile_errors::BadTypeRightOperand { source: self.source(), operator: parse_data.token_ranges[index].clone(), right: operand }); None },
        }
    }

    fn check_numeric_prefix<F: Fn(BigRational)->BigRational>(&mut self, operand: Type, index: AstIndex, parse_data: &ParseData, f: F) -> Type {
        match self.check_numeric_prefix_argument(operand, index, parse_data) {
            Some(operand) => Rational(f(operand)),
            None => Error,
        }
    }

    fn check_integer_prefix<F: Fn(BigRational)->BigRational>(&mut self, operand: Type, index: AstIndex, parse_data: &ParseData, f: F) -> Type {
        match self.check_integer_prefix_argument(operand, index, parse_data) {
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

    fn check_integer_postfix<F: Fn(BigRational)->BigRational>(&mut self, operand: Type, index: AstIndex, parse_data: &ParseData, f: F) -> Type {
        match self.check_integer_postfix_argument(operand, index, parse_data) {
            Some(operand) => Rational(f(operand)),
            None => Error,
        }
    }

    fn assign(&mut self, right: Type, index: AstIndex, parse_data: &ParseData) -> Type {
        let value = right;
        match parse_data.tokens[index-1] {
            Token::PropertyReference(identifier)|Token::PropertyDeclarationTarget(identifier) => {
                self.scope.set(identifier, value);
                Nothing
            },
            _ => {
                self.report(compile_errors::LeftSideOfAssignmentMustBeIdentifier { source: self.source(), left: parse_data.token_range(index-1), operator: parse_data.token_range(index) });
                Error
            },
        }
    }

    fn assign_operation(&mut self, operation: IdentifierIndex, left: Type, right: Type, index: AstIndex, parse_data: &ParseData) -> Type {
        let value = self.check_infix(InfixOperator(operation), left, right, index, parse_data);
        match parse_data.tokens[index-1] {
            Token::PropertyReference(identifier)|Token::PropertyDeclarationTarget(identifier) => {
                self.scope.set(identifier, value);
                Nothing
            },
            _ => {
                self.report(compile_errors::LeftSideOfAssignmentMustBeIdentifier { source: self.source(), left: parse_data.token_range(index-1), operator: parse_data.token_range(index) });
                Error
            },
        }
    }

    fn assign_integer_postfix<F: Fn(BigRational)->BigRational>(&mut self, operand: Type, index: AstIndex, parse_data: &ParseData, f: F) -> Type {
        let value = self.check_integer_postfix(operand, index, parse_data, f);
        match parse_data.tokens[index-1] {
            Token::PropertyReference(identifier)|Token::PropertyDeclarationTarget(identifier) => {
                self.scope.set(identifier, value);
                Nothing
            },
            _ => {
                self.report(compile_errors::LeftSideOfAssignmentMustBeIdentifier { source: self.source(), left: parse_data.token_range(index-1), operator: parse_data.token_range(index) });
                Error
            },
        }
    }

    fn assign_integer_prefix<F: Fn(BigRational)->BigRational>(&mut self, operand: Type, index: AstIndex, parse_data: &ParseData, f: F) -> Type {
        let value = self.check_integer_prefix(operand, index, parse_data, f);
        match parse_data.tokens[index+1] {
            Token::PropertyReference(identifier)|Token::PropertyDeclarationTarget(identifier) => {
                self.scope.set(identifier, value);
                Nothing
            },
            // TODO if left side is a parenthetical, this will look silly. Fix by including the whole range of the expression.
            _ => {
                self.report(compile_errors::RightSideOfIncrementOrDecrementOperandMustBeIdentifier { source: self.source(), right: parse_data.token_range(index+1), operator: parse_data.token_range(index) });
                Error
            },
        }
    }

    fn report<T: CompileError+'c>(&self, error: T) -> Type {
        self.compiler.report(error);
        Error
    }

    fn source(&self) -> SourceIndex {
        self.source
    }
}

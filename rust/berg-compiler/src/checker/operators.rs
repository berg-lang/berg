use parser::AstIndex;
use parser::IdentifierIndex;
use public::*;
use checker::Checker;
use checker::operators::Precedence::*;
use checker::checker_type::Type::*;
use num::BigRational;
use num::traits::*;

#[derive(Debug, PartialEq)]
pub(super) enum Infix {
    Add,
    Subtract,
    Multiply,
    Divide,
    Unrecognized,
}
#[derive(Debug, PartialEq)]
pub(super) enum Prefix {
    Negative,
    Positive,
    Unrecognized,
}
#[derive(Debug, PartialEq)]
pub(super) enum Postfix {
    Unrecognized,
}

#[derive(Debug, PartialEq, PartialOrd)]
pub enum Precedence {
    Other,
    MathNegativePositive,
    MathMultiplyDivide,
    MathAddSubtract,
}

pub(super) fn infix(source_data: &SourceData, identifier: IdentifierIndex) -> Infix {
    use checker::operators::Infix::*;
    match source_data.identifier_string(identifier) {
        "+" => Add,
        "-" => Subtract,
        "*" => Multiply,
        "/" => Divide,
        _ => Unrecognized,
    }
}

pub(super) fn prefix(source_data: &SourceData, identifier: IdentifierIndex) -> Prefix {
    use checker::operators::Prefix::*;
    match source_data.identifier_string(identifier) {
        "+" => Positive,
        "-" => Negative,
        _ => Unrecognized,
    }
}

pub(super) fn postfix(source_data: &SourceData, identifier: IdentifierIndex) -> Postfix {
    use checker::operators::Postfix::*;
    match source_data.identifier_string(identifier) {
        _ => Unrecognized,
    }
}

impl Infix {
    pub(super) fn precedence(&self) -> Precedence {
        use checker::operators::Infix::*;
        match *self {
            Add | Subtract => MathAddSubtract,
            Multiply | Divide => MathMultiplyDivide,
            Unrecognized => Other,
        }
    }

    pub(super) fn check(
        &self,
        checker: &mut Checker,
        index: AstIndex,
        last_precedence: Precedence,
        left: Type,
        right: Type,
    ) -> Type {
        use CompileErrorType::*;
        use checker::operators::Infix::*;
        if *self == Unrecognized {
            println!("Unrecognized!!!");
            checker.report_at_token(UnrecognizedOperator, index)
        } else if self.precedence() < last_precedence && left != Error && right != Error {
            println!(
                "Precedence error: ${:?}.precedence({:?}) > {:?}",
                self,
                self.precedence(),
                last_precedence
            );
            checker.report_at_token(OperatorsOutOfPrecedenceOrder, index)
        } else {
            match *self {
                Add => {
                    self.check_numeric(left, right, |left, right| left + right)
                }
                Subtract => {
                    self.check_numeric(left, right, |left, right| left - right)
                }
                Multiply => {
                    self.check_numeric(left, right, |left, right| left * right)
                }
                Divide => {
                    if let Rational(ref denom) = right {
                        if denom.is_zero() {
                            checker.report_at_token(DivideByZero, index);
                            return Error;
                        }
                    }
                    self.check_numeric(left, right, |left, right| left / right)
                }
                Unrecognized => unreachable!(),
            }
        }
    }

    fn check_numeric<F: FnOnce(BigRational, BigRational) -> BigRational>(
        &self,
        left: Type,
        right: Type,
        f: F,
    ) -> Type {
        match (left, right) {
            (Nothing, _) => unreachable!(), //checker.report_at_token(BadType, index),
            (_, Nothing) => unreachable!(), //checker.report_at_token(BadType, index),
            (Error, _) | (_, Error) => Error,
            (Rational(left), Rational(right)) => Rational(f(left, right)),
        }
    }
}

impl Prefix {
    pub fn precedence(&self) -> Precedence {
        use checker::operators::Prefix::*;
        match *self {
            Negative | Positive => MathNegativePositive,
            Unrecognized => Other,
        }
    }

    pub fn check(&self, checker: &mut Checker, index: AstIndex, right: Type) -> Type {
        use checker::operators::Prefix::*;
        use CompileErrorType::*;
        if *self == Unrecognized {
            checker.report_at_token(UnrecognizedOperator, index)
        } else {
            match *self {
                Positive => self.check_numeric(checker, index, right, |right| right),
                Negative => self.check_numeric(checker, index, right, |right| -right),
                Unrecognized => unreachable!(),
            }
        }
    }

    fn check_numeric<F: FnOnce(BigRational) -> BigRational>(
        &self,
        checker: &mut Checker,
        index: AstIndex,
        right: Type,
        f: F,
    ) -> Type {
        match right {
            Nothing => checker.report_at_token(CompileErrorType::MissingRightOperand, index),
            Error => Error,
            Rational(right) => Rational(f(right)),
        }
    }
}

impl Postfix {
    pub fn precedence(&self) -> Precedence {
        use checker::operators::Postfix::*;
        match *self {
            Unrecognized => Other,
        }
    }

    pub fn check(
        &self,
        checker: &mut Checker,
        index: AstIndex,
        last_precedence: Precedence,
        left: Type,
    ) -> Type {
        use checker::operators::Postfix::*;
        use CompileErrorType::*;
        if *self == Unrecognized {
            checker.report_at_token(UnrecognizedOperator, index)
        } else if self.precedence() < last_precedence && left != Error {
            checker.report_at_token(OperatorsOutOfPrecedenceOrder, index)
        } else {
            match *self {
                Unrecognized => unreachable!(),
            }
        }
    }
}

use public::*;
use checker::Checker;
use checker::operators::Precedence::*;
use checker::checker_type::Type::*;
use num::traits::*;

#[derive(Debug,PartialEq)]
pub enum Infix {
    Add,
    Subtract,
    Multiply,
    Divide,
    Unrecognized,
}
#[derive(Debug,PartialEq)]
pub enum Prefix {
    Negative,
    Positive,
    Unrecognized,
}
#[derive(Debug,PartialEq)]
pub enum Postfix {
    Unrecognized,
}

#[derive(Debug,PartialEq,PartialOrd)]
pub enum Precedence {
    Other,
    MathNegativePositive,
    MathMultiplyDivide,
    MathAddSubtract,
}

pub fn infix(string: &str) -> Infix {
    use checker::operators::Infix::*;
    match string {
        "+" => Add,
        "-" => Subtract,
        "*" => Multiply,
        "/" => Divide,
        _   => Unrecognized,
    }
}

pub fn prefix(string: &str) -> Prefix {
    use checker::operators::Prefix::*;
    match string {
        "+" => Positive,
        "-" => Negative,
        _   => Unrecognized,
    }
}

pub fn postfix(string: &str) -> Postfix {
    use checker::operators::Postfix::*;
    match string {
        _   => Unrecognized,
    }
}

impl Infix {
    pub fn precedence(&self) -> Precedence {
        use checker::operators::Infix::*;
        match *self {
            Add|Subtract => MathAddSubtract,
            Multiply|Divide => MathMultiplyDivide,
            Unrecognized => Other,
        }
    }

    pub fn evaluate(&self, checker: &Checker, index: TokenIndex, last_precedence: Precedence, left: Type, right: Type) -> Type {
        use checker::operators::Infix::*;
        if *self == Unrecognized {
            println!("Unrecognized!!!");
            checker.report_at_token(UnrecognizedOperator, index)
        } else if self.precedence() < last_precedence && left != Error && right != Error {
            println!("Precedence error: ${:?}.precedence({:?}) > {:?}", self, self.precedence(), last_precedence);
            checker.report_at_token(OperatorsOutOfPrecedenceOrder, index)
        } else {
            match *self {
                Add      => self.evaluate_numeric(checker, index, left, right, |left,right| left+right),
                Subtract => self.evaluate_numeric(checker, index, left, right, |left,right| left-right),
                Multiply => self.evaluate_numeric(checker, index, left, right, |left,right| left*right),
                Divide   => {
                    if let Rational(ref denom) = right {
                        if denom.is_zero() {
                            checker.report_at_token(DivideByZero, index);
                            return Error;
                        }
                    }
                    self.evaluate_numeric(checker, index, left, right, |left,right| left/right)
                }
                Unrecognized => unreachable!(),
            }
        }
    }

    fn evaluate_numeric<F: FnOnce(BigRational,BigRational)->BigRational>(&self, evaluator: &Checker, index: TokenIndex, left: Type, right: Type, f: F) -> Type {
        match (left, right) {
            (Nothing, Nothing) => evaluator.report_at_token(MissingBothOperands, index),
            (_, Nothing) => evaluator.report_at_token(MissingLeftOperand, index),
            (Nothing, _) => evaluator.report_at_token(MissingRightOperand, index),
            (Error, _)|(_, Error) => Error,
            (Rational(left), Rational(right)) => Rational(f(left, right)),
        }
    }
}

impl Prefix {

    pub fn precedence(&self) -> Precedence {
        use checker::operators::Prefix::*;
        match *self {
            Negative|Positive => MathNegativePositive,
            Unrecognized => Other,
        }
    }

    pub fn evaluate(&self, evaluator: &Checker, index: TokenIndex, right: Type) -> Type {
        use checker::operators::Prefix::*;
        if *self == Unrecognized {
            println!("Unrecognized!!!");
            evaluator.report_at_token(UnrecognizedOperator, index)
        } else {
            println!("Recognized! {:?}({:?}", self, right);
            let result = match *self {
                Positive => self.evaluate_numeric(evaluator, index, right, |right| right),
                Negative => self.evaluate_numeric(evaluator, index, right, |right| -right),
                Unrecognized => unreachable!(),
            };
            println!("----> Result: {:?}", result);
            result
        }
    }

    fn evaluate_numeric<F: FnOnce(BigRational)->BigRational>(&self, evaluator: &Checker, index: TokenIndex, right: Type, f: F) -> Type {
        match right {
            Nothing => evaluator.report_at_token(MissingRightOperand, index),
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

    pub fn evaluate(&self, evaluator: &Checker, index: TokenIndex, last_precedence: Precedence, left: Type) -> Type {
        use checker::operators::Postfix::*;
        if *self == Unrecognized {
            evaluator.report_at_token(UnrecognizedOperator, index)
        } else if self.precedence() < last_precedence && left != Error {
            evaluator.report_at_token(OperatorsOutOfPrecedenceOrder, index)
        } else {
            match *self {
                Unrecognized => unreachable!(),
            }
        }
    }
}

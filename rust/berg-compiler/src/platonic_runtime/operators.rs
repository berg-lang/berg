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

use platonic_runtime::*;
use platonic_runtime::PlatonicValue::*;
use platonic_runtime::operators::Precedence::*;
use num::traits::*;

pub fn infix(string: &str) -> Infix {
    use platonic_runtime::operators::Infix::*;
    match string {
        "+" => Add,
        "-" => Subtract,
        "*" => Multiply,
        "/" => Divide,
        _   => Unrecognized,
    }
}

pub fn prefix(string: &str) -> Prefix {
    use platonic_runtime::operators::Prefix::*;
    match string {
        "+" => Positive,
        "-" => Negative,
        _   => Unrecognized,
    }
}

pub fn postfix(string: &str) -> Postfix {
    use platonic_runtime::operators::Postfix::*;
    match string {
        _   => Unrecognized,
    }
}

impl Infix {
    pub fn precedence(&self) -> Precedence {
        use platonic_runtime::operators::Infix::*;
        match *self {
            Add|Subtract => MathAddSubtract,
            Multiply|Divide => MathMultiplyDivide,
            Unrecognized => Other,
        }
    }

    pub fn evaluate(&self, evaluator: &PlatonicEvaluator, index: TokenIndex, last_precedence: Precedence, left: PlatonicValue, right: PlatonicValue) -> PlatonicValue {
        use platonic_runtime::operators::Infix::*;
        if *self == Unrecognized {
            println!("Unrecognized!!!");
            evaluator.report_at_token(UnrecognizedOperator, index)
        } else if self.precedence() < last_precedence && left != Error && right != Error {
            println!("Precedence error: ${:?}.precedence({:?}) > {:?}", self, self.precedence(), last_precedence);
            evaluator.report_at_token(OperatorsOutOfPrecedenceOrder, index)
        } else {
            match *self {
                Add      => self.evaluate_numeric(evaluator, index, left, right, |left,right| left+right),
                Subtract => self.evaluate_numeric(evaluator, index, left, right, |left,right| left-right),
                Multiply => self.evaluate_numeric(evaluator, index, left, right, |left,right| left*right),
                Divide   => {
                    if let Rational(ref denom) = right {
                        if denom.is_zero() {
                            evaluator.report_at_token(DivideByZero, index);
                            return Error;
                        }
                    }
                    self.evaluate_numeric(evaluator, index, left, right, |left,right| left/right)
                }
                Unrecognized => unreachable!(),
            }
        }
    }

    fn evaluate_numeric<F: FnOnce(BigRational,BigRational)->BigRational>(&self, evaluator: &PlatonicEvaluator, index: TokenIndex, left: PlatonicValue, right: PlatonicValue, f: F) -> PlatonicValue {
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
        use platonic_runtime::operators::Prefix::*;
        match *self {
            Negative|Positive => MathNegativePositive,
            Unrecognized => Other,
        }
    }

    pub fn evaluate(&self, evaluator: &PlatonicEvaluator, index: TokenIndex, right: PlatonicValue) -> PlatonicValue {
        use platonic_runtime::operators::Prefix::*;
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

    fn evaluate_numeric<F: FnOnce(BigRational)->BigRational>(&self, evaluator: &PlatonicEvaluator, index: TokenIndex, right: PlatonicValue, f: F) -> PlatonicValue {
        match right {
            Nothing => evaluator.report_at_token(MissingRightOperand, index),
            Error => Error,
            Rational(right) => Rational(f(right)),
        }
    }
}

impl Postfix {
    pub fn precedence(&self) -> Precedence {
        use platonic_runtime::operators::Postfix::*;
        match *self {
            Unrecognized => Other,
        }
    }

    pub fn evaluate(&self, evaluator: &PlatonicEvaluator, index: TokenIndex, last_precedence: Precedence, left: PlatonicValue) -> PlatonicValue {
        use platonic_runtime::operators::Postfix::*;
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

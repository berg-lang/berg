use std::str::FromStr;
use public::*;

pub struct PlatonicRuntime;

pub struct PlatonicEvaluator<'r, 'c: 'r> {
    compiler: &'r Compiler<'c>,
    source: SourceIndex,
    source_data: &'r SourceData<'c>,
}

#[derive(Debug,PartialEq,PartialOrd)]
pub enum PlatonicValue {
    Integer(BigInt),
    Error,
    Nothing,
}

use PlatonicValue::*;

impl PlatonicRuntime {
    pub fn run(compiler: &Compiler, source: SourceIndex) -> PlatonicValue {
        compiler.with_source(source, |source_data| {
            let evaluator = PlatonicEvaluator { compiler, source, source_data };
            evaluator.evaluate()
        })
    }
}

impl<'r, 'c: 'r> PlatonicEvaluator<'r, 'c> {
    pub fn evaluate(&self) -> PlatonicValue {
        let (mut index, mut value) = self.evaluate_one(0);
        while index < self.source_data.num_tokens() {
            let token = self.source_data.token(index);
            match token.token_type {
                Postfix => {
                    value = self.evaluate_postfix(index, &token.string, value);
                    index += 1;
                },
                Infix => {
                    let (next_index, right_operand) = self.evaluate_one(index+1);
                    value = self.evaluate_infix(index, &token.string, value, right_operand);
                    index = next_index;
                },
                _ => unreachable!(),
            }
        }
        value
    }
    fn evaluate_one(&self, index: TokenIndex) -> (TokenIndex, PlatonicValue) {
        if index >= self.source_data.num_tokens() {
            return (index, PlatonicValue::Nothing);
        }

        let token = self.source_data.token(index);
        match token.token_type {
            Term(ref term_type) => {
                let value = self.evaluate_term(term_type, &token.string);
                (index+1, value)
            },
            Prefix => {
                let (next_index, value) = self.evaluate_one(index+1);
                let value = self.evaluate_prefix(index, &token.string, value);
                (next_index, value)
            },
            _ => unreachable!(),
        }
    }

    fn evaluate_term(&self, term_type: &TermType, string: &str) -> PlatonicValue {
        match *term_type {
            IntegerLiteral => PlatonicValue::Integer(BigInt::from_str(string).unwrap()),
        }
    }
    fn evaluate_prefix(&self, index: TokenIndex, string: &str, operand: PlatonicValue) -> PlatonicValue {
        match string {
            "+" => self.evaluate_numeric_prefix(index, operand, |num| num),
            "-" => self.evaluate_numeric_prefix(index, operand, |num| -num),
            _ => self.report_at_token(UnrecognizedOperator, index),
        }
    }
    fn evaluate_postfix(&self, index: TokenIndex, string: &str, _: PlatonicValue) -> PlatonicValue {
        match string {
            _ => self.report_at_token(UnrecognizedOperator, index),
        }
    }
    fn evaluate_infix(&self, index: TokenIndex, string: &str, left: PlatonicValue, right: PlatonicValue) -> PlatonicValue {
        match string {
            "+" => self.evaluate_numeric_infix(index, left, right, |a,b| a+b),
            "-" => self.evaluate_numeric_infix(index, left, right, |a,b| a-b),
            _ => self.report_at_token(UnrecognizedOperator, index),
        }
    }

    fn evaluate_numeric_prefix<F: Fn(BigInt)->BigInt>(&self, index: TokenIndex, operand: PlatonicValue, f: F) -> PlatonicValue {
        match operand {
            Nothing => self.report_at_token(MissingRightOperand, index),
            Error => Error,
            Integer(value) => Integer(f(value)),
        }
    }
    fn evaluate_numeric_infix<F: Fn(BigInt,BigInt)->BigInt>(&self, index: TokenIndex, left: PlatonicValue, right: PlatonicValue, f: F) -> PlatonicValue {
        match (left, right) {
            (Nothing, Nothing) => self.report_at_token(MissingBothOperands, index),
            (_, Nothing) => self.report_at_token(MissingLeftOperand, index),
            (Nothing, _) => self.report_at_token(MissingRightOperand, index),
            (Integer(left), Integer(right)) => Integer(f(left, right)),
            _ => Error
        }
    }

    fn report_at_token(&self, error_type: CompileErrorType, token: TokenIndex) -> PlatonicValue {
        let start = self.source_data.token_start(token);
        let string = &self.source_data.token(token).string;
        self.compiler.report_at(error_type, self.source, start, string);
        Error
    }
}

impl<T: Into<BigInt>> From<T> for PlatonicValue { fn from(value: T) -> PlatonicValue { PlatonicValue::Integer(value.into()) } }

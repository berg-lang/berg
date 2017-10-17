mod operators;

use public::*;
use std::str::FromStr;

pub struct PlatonicRuntime;

pub struct PlatonicEvaluator<'r, 'c: 'r> {
    compiler: &'r Compiler<'c>,
    source: SourceIndex,
    source_data: &'r SourceData<'c>,
}

#[derive(Debug,PartialEq,PartialOrd)]
pub enum PlatonicValue {
    Rational(BigRational),
    Error,
    Nothing,
}

use platonic_runtime::PlatonicValue::*;
use platonic_runtime::operators::Precedence;

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
        let (mut index, mut value, mut last_precedence) = self.evaluate_one(0, Precedence::Other);
        while index < self.source_data.num_tokens() {
            let token = self.source_data.token(index);
            match token.token_type {
                Postfix => {
                    let operator = operators::postfix(&token.string);
                    value = operator.evaluate(self, index, last_precedence, value);
                    last_precedence = operator.precedence();
                    index += 1;
                },
                Infix => {
                    let operator = operators::infix(&token.string);
                    let (next_index, right_operand, next_precedence) = self.evaluate_one(index+1, operator.precedence());
                    value = operator.evaluate(self, index, last_precedence, value, right_operand);
                    last_precedence = next_precedence;
                    index = next_index;
                },
                _ => unreachable!(),
            }
        }
        value
    }

    fn evaluate_one(&self, index: TokenIndex, last_precedence: Precedence) -> (TokenIndex, PlatonicValue, Precedence) {
        if index >= self.source_data.num_tokens() {
            return (index, PlatonicValue::Nothing, last_precedence);
        }

        let token = self.source_data.token(index);
        match token.token_type {
            Term(ref term_type) => {
                let value = self.evaluate_term(term_type, &token.string);
                (index+1, value, last_precedence)
            },
            Prefix => {
                let operator = operators::prefix(&token.string);
                let (next_index, right_operand, last_precedence) = self.evaluate_one(index+1, operator.precedence());
                let value = operator.evaluate(self, index, right_operand);
                (next_index, value, last_precedence)
            },
            _ => unreachable!(),
        }
    }

    fn evaluate_term(&self, term_type: &TermType, string: &str) -> PlatonicValue {
        match *term_type {
            IntegerLiteral => PlatonicValue::Rational(BigRational::from_str(string).unwrap()),
        }
    }

    fn report_at_token(&self, error_type: CompileErrorType, token: TokenIndex) -> PlatonicValue {
        let start = self.source_data.token_start(token);
        let string = &self.source_data.token(token).string;
        self.compiler.report_at(error_type, self.source, start, string);
        Error
    }
}

use num::bigint::BigInt;
impl From<i64> for PlatonicValue { fn from(value: i64) -> PlatonicValue { BigInt::from(value).into() } }
impl From<BigInt> for PlatonicValue { fn from(value: BigInt) -> PlatonicValue { BigRational::from(value).into() } }
impl From<BigRational> for PlatonicValue { fn from(value: BigRational) -> PlatonicValue { PlatonicValue::Rational(value) } }

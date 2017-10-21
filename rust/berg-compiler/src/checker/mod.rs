pub(crate) mod checker_type;
pub(crate) mod operators;

use public::*;

use checker::checker_type::Type::*;
use checker::operators::*;
use num::BigRational;
use std::str::FromStr;

pub struct Checker<'ch, 'c: 'ch> {
    compiler: &'ch Compiler<'c>,
    source: SourceIndex,
    source_data: &'ch SourceData<'c>,
}

pub fn check<'c>(
    compiler: &Compiler<'c>,
    source: SourceIndex,
    source_data: &SourceData<'c>,
) -> Type {
    let checker = Checker::new(compiler, source, source_data);
    checker.check()
}

impl<'ch, 'c: 'ch> Checker<'ch, 'c> {
    fn new(
        compiler: &'ch Compiler<'c>,
        source: SourceIndex,
        source_data: &'ch SourceData<'c>,
    ) -> Self {
        Checker {
            compiler,
            source,
            source_data,
        }
    }

    fn check(&self) -> Type {
        let (mut index, mut value, mut last_precedence) =
            self.evaluate_one(0, Precedence::Other);
        while index < self.source_data.num_tokens() {
            let token = self.source_data.token(index);
            match *token {
                Token::Postfix(token_index) => {
                    let string = self.source_data.token_string(token_index);
                    let operator = operators::postfix(string);
                    value = operator.evaluate(self, index, last_precedence, value);
                    last_precedence = operator.precedence();
                    index += 1;
                }
                Token::Infix(token_index) => {
                    let string = self.source_data.token_string(token_index);
                    let operator = operators::infix(string);
                    let (next_index, right_operand, next_precedence) =
                        self.evaluate_one(index + 1, operator.precedence());
                    value = operator.evaluate(self, index, last_precedence, value, right_operand);
                    last_precedence = next_precedence;
                    index = next_index;
                }
                _ => unreachable!(),
            }
        }
        value
    }

    fn evaluate_one(
        &self,
        index: usize,
        last_precedence: Precedence,
    ) -> (usize, Type, Precedence) {
        if index >= self.source_data.num_tokens() {
            return (index, Type::Nothing, last_precedence);
        }

        let token = self.source_data.token(index);
        match *token {
            Token::Term(ref term_type) => {
                let value = self.evaluate_term(term_type);
                (index + 1, value, last_precedence)
            }
            Token::Prefix(token_index) => {
                let string = self.source_data.token_string(token_index);
                let operator = operators::prefix(string);
                let (next_index, right_operand, last_precedence) =
                    self.evaluate_one(index + 1, operator.precedence());
                let value = operator.evaluate(self, index, right_operand);
                (next_index, value, last_precedence)
            }
            _ => unreachable!(),
        }
    }

    fn evaluate_term(&self, term_type: &TermType) -> Type {
        match *term_type {
            TermType::IntegerLiteral(ref string) => Type::Rational(BigRational::from_str(string).unwrap()),
        }
    }

    pub fn report_at_token(&self, error_type: CompileErrorType, index: usize) -> Type {
        let token = self.source_data.token(index);
        let range = self.source_data.token_range(index);
        self.compiler
            .report_at(error_type, self.source, range, token.string(self.source_data));
        Error
    }
}

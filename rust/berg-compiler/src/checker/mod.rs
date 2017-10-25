pub mod checker_type;
mod operators;

use public::*;

use checker::checker_type::Type::*;
use checker::operators::*;
use num::BigRational;
use compiler::compile_errors::*;
use std::str::FromStr;

pub(super) fn check<'c>(
    errors: &mut SourceCompileErrors,
    source_data: &SourceData<'c>
) -> Type {
    let mut checker = Checker::new(errors, source_data);
    checker.check()
}

struct Checker<'ch, 'c: 'ch> {
    errors: &'ch mut SourceCompileErrors,
    source_data: &'ch SourceData<'c>,
}

impl<'ch, 'c: 'ch> Checker<'ch, 'c> {
    fn new(
        errors: &'ch mut SourceCompileErrors,
        source_data: &'ch SourceData<'c>,
    ) -> Self {
        Checker {
            errors,
            source_data,
        }
    }

    fn check(&mut self) -> Type {
        use Token::*;
        let (mut index, mut value, mut last_precedence) =
            self.check_one(0, Precedence::Other);
        while index < self.source_data.num_tokens() {
            let token = self.source_data.token(index);
            match *token {
                Postfix(operator) => {
                    let operator = operators::postfix(self.source_data, operator);
                    value = operator.check(self, index, last_precedence, value);
                    last_precedence = operator.precedence();
                    index += 1;
                },
                Infix(operator) => {
                    let operator = operators::infix(self.source_data, operator);
                    let (next_index, right_operand, next_precedence) =
                        self.check_one(index + 1, operator.precedence());
                    value = operator.check(self, index, last_precedence, value, right_operand);
                    last_precedence = next_precedence;
                    index = next_index;
                },
                MissingInfix => {
                    let (next_index, _, next_precedence) =
                        self.check_one(index + 1, Precedence::Other);
                    value = self.report_at_token(CompileErrorType::UnrecognizedOperator, index);
                    last_precedence = next_precedence;
                    index = next_index;
                },
                IntegerLiteral(_)|Prefix(_)|Nothing|MissingTerm => unreachable!(),
            }
        }
        value
    }

    fn check_one(
        &mut self,
        index: usize,
        last_precedence: Precedence,
    ) -> (usize, Type, Precedence) {
        if index >= self.source_data.num_tokens() {
            return (index, Type::Nothing, last_precedence);
        }

        use Token::*;

        let token = self.source_data.token(index);
        match *token {
            IntegerLiteral(literal) => {
                let string = self.source_data.literal_string(literal);
                let value = Type::Rational(BigRational::from_str(string).unwrap());
                (index + 1, value, last_precedence)
            },
            Prefix(operator) => {
                let operator = operators::prefix(self.source_data, operator);
                let (next_index, right_operand, last_precedence) =
                    self.check_one(index + 1, operator.precedence());
                let value = operator.check(self, index, right_operand);
                (next_index, value, last_precedence)
            },
            MissingTerm => {
                // TODO these are wrong, MissingBothOperands applies to the operator, not the missing spot.
                // Attach the error to whichever operand we think is most likely, instead.
                let left = index > 0 && self.source_data.token(index-1).has_right_operand();
                let right = index+1 < self.source_data.num_tokens() && self.source_data.token(index+1).has_left_operand();
                let value = match (left, right) {
                    (true, true) => {
                        let start = self.source_data.token_range(index-1).start;
                        let end = self.source_data.token_range(index+1).end;
                        let string = format!("{} {}",
                            self.source_data.token_string(index-1),
                            self.source_data.token_string(index+1)
                        );
                        self.errors.report_at(CompileErrorType::MissingOperandsBetween, start..end, &string);
                        Error
                    },
                    (true, false) => self.report_at_token(CompileErrorType::MissingLeftOperand, index-1),
                    (false, true) => self.report_at_token(CompileErrorType::MissingRightOperand, index-2),
                    (false, false) => Type::Nothing,
                };
                (index + 1, value, last_precedence)
            },
            Nothing => (index + 1, Type::Nothing, Precedence::Other),
            Postfix(_)|Infix(_)|MissingInfix => unreachable!(),
        }
    }

    pub fn report_at_token(&mut self, error_type: CompileErrorType, token: usize) -> Type {
        let range = self.source_data.token_range(token);
        let string = self.source_data.token_string(token);
        self.errors
            .report_at(error_type, range, string);
        Error
    }
}

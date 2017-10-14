use std::str::FromStr;
use public::*;

pub struct PlatonicRuntime {
}

#[derive(Debug,PartialEq,PartialOrd)]
pub enum PlatonicValue {
    Integer(BigInt),
    Nothing,
}

impl PlatonicRuntime {
    pub fn run<'c>(compiler: &Compiler<'c>, main_source: SourceIndex) -> PlatonicValue {
        compiler.with_source(main_source, |source| Self::run_source(source))
    }
    pub fn run_source<'c>(source: &SourceData<'c>) -> PlatonicValue {
        match source.num_tokens() {
            0 => PlatonicValue::Nothing,
            1 => Self::run_expression(source.token(0)),
            _ => panic!("Too many expressions, I don't understand"),
        }
    }
    pub fn run_expression(expression: &Token) -> PlatonicValue {
        match expression.token_type {
            IntegerLiteral => PlatonicValue::Integer(BigInt::from_str(&expression.string).unwrap()),
        }
    }
}

impl<T: Into<BigInt>> From<T> for PlatonicValue { fn from(value: T) -> PlatonicValue { PlatonicValue::Integer(value.into()) } }

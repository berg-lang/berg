use public::*;
use std::u32;

use Token::*;
use TermType::*;

index_type! { pub struct TokenIndex(pub u32) }

#[derive(Debug)]
pub enum Token {
    Term(TermType),
    Prefix(TokenIndex),
    Postfix(TokenIndex),
    Infix(TokenIndex),
}

impl Token {
    pub fn string<'s>(&'s self, source_data: &'s SourceData) -> &'s str {
        match *self {
            Term(IntegerLiteral(ref string)) => string,
            Prefix(index)|Postfix(index)|Infix(index) => source_data.token_string(index),
        }
    }
}

#[derive(Debug)]
pub enum TermType {
    IntegerLiteral(String),
}

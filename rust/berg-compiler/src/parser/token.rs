use public::*;

use parser::IdentifierTokenIndex;
use parser::LiteralTokenIndex;
use Token::*;

#[derive(Debug)]
pub enum Token {
    IntegerLiteral(LiteralTokenIndex),
    Prefix(IdentifierTokenIndex),
    Postfix(IdentifierTokenIndex),
    Infix(IdentifierTokenIndex),
}

impl Token {
    pub fn string<'s>(&'s self, source_data: &'s SourceData) -> &'s str {
        match *self {
            IntegerLiteral(index) => source_data.literal_token_string(index),
            Prefix(index)|Postfix(index)|Infix(index) => source_data.identifier_token_string(index),
        }
    }
}

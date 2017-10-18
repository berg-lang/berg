use indexed_vec::IndexedVec;
use std::u32;
use public::*;

// ExpressionType, String, LeftChild, RightChild
#[derive(Debug)]
pub struct Token {
    pub token_type: TokenType,
    pub string: String,
}

// Tokens is a list of tokens, indexable by indexes of type `TokenIndex`.
// TokenStarts is a list of indexes tokens start at.
index_type!(pub struct TokenIndex(u32));
pub type Tokens = IndexedVec<Token, TokenIndex>;
pub type TokenStarts = IndexedVec<ByteIndex, TokenIndex>;

impl Token {
    pub fn new(token_type: TokenType, string: String) -> Token {
        Token { token_type, string }
    }
}

#[derive(Debug)]
pub enum TokenType {
    Term(TermType),
    Prefix,
    Postfix,
    Infix,
}

impl From<TermType> for TokenType {
    fn from(term_type: TermType) -> TokenType {
        TokenType::Term(term_type)
    }
}

#[derive(Debug)]
pub enum TermType {
    IntegerLiteral,
}

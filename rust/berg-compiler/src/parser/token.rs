pub type TokenIndex = u32;

// ExpressionType, String, LeftChild, RightChild
#[derive(Debug)]
pub struct Token {
    pub token_type: TokenType,
    pub string: String,
}

impl Token {
    pub fn new(
        token_type: TokenType,
        string: String,
    ) -> Token {
        Token {
            token_type,
            string,
        }
    }
}

#[derive(Debug)]
pub enum TokenType {
    IntegerLiteral,
}

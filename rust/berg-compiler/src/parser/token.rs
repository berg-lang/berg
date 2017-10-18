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

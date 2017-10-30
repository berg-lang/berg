use ast::{IdentifierIndex,LiteralIndex};
use ast::operators::Operators;
use ast::token::InfixToken::*;
use ast::token::PrefixToken::*;
use ast::token::PostfixToken::*;

// ExpressionType, String, LeftChild, RightChild
#[derive(Debug,Copy,Clone,PartialEq)]
pub enum Token {
    IntegerLiteral(LiteralIndex),
    MissingOperand,
    NoExpression,

    InfixOperator(IdentifierIndex),
    MissingInfix,

    PrefixOperator(IdentifierIndex),
    Open(IdentifierIndex),

    PostfixOperator(IdentifierIndex),
    Close(IdentifierIndex),
}

#[derive(Debug,Copy,Clone,PartialEq)]
pub enum TermToken {
    IntegerLiteral(LiteralIndex),
    MissingOperand,
    NoExpression,
}

#[derive(Debug,Copy,Clone,PartialEq)]
pub enum InfixToken {
    InfixOperator(IdentifierIndex),
    MissingInfix,
}

#[derive(Debug,Copy,Clone,PartialEq)]
pub enum PrefixToken {
    PrefixOperator(IdentifierIndex),
    Open(IdentifierIndex),
}

#[derive(Debug,Copy,Clone,PartialEq)]
pub enum PostfixToken {
    PostfixOperator(IdentifierIndex),
    Close(IdentifierIndex),
}

#[derive(Debug,Copy,Clone,PartialEq)]
pub enum Fixity {
    Term,
    Infix,
    Prefix,
    Postfix,
}

impl Token {
    pub fn fixity(&self) -> Fixity {
        use Token::*;
        match *self {
            IntegerLiteral(_)|MissingOperand|NoExpression => Fixity::Term,
            InfixOperator(_)|MissingInfix => Fixity::Infix,
            PrefixOperator(_)|Open(_) => Fixity::Prefix,
            PostfixOperator(_)|Close(_) => Fixity::Postfix,
        }
    }
    pub fn to_term(self) -> Option<TermToken> { TermToken::try_from(self) }
    pub fn to_infix(self) -> Option<InfixToken> { InfixToken::try_from(self) }
    pub fn to_postfix(self) -> Option<PostfixToken> { PostfixToken::try_from(self) }
    pub fn to_prefix(self) -> Option<PrefixToken> { PrefixToken::try_from(self) }
    pub fn num_operands(&self) -> u8 { self.fixity().num_operands() }
    pub fn has_left_operand(&self) -> bool { self.fixity().has_left_operand() }
    pub fn has_right_operand(&self) -> bool { self.fixity().has_right_operand() }
}

impl Fixity {
    pub fn num_operands(&self) -> u8 {
        use ast::token::Fixity::*;
        match *self {
            Term => 0,
            Prefix|Postfix => 1,
            Infix => 2,
        }
    }
    pub fn has_left_operand(&self) -> bool {
        use ast::token::Fixity::*;
        match *self {
            Term|Prefix => false,
            Infix|Postfix => true,
        }
    }
    pub fn has_right_operand(&self) -> bool {
        use ast::token::Fixity::*;
        match *self {
            Term|Postfix => false,
            Infix|Prefix => true,
        }
    }
}

impl TermToken {
    pub fn try_from(token: Token) -> Option<Self> {
        match token {
            Token::IntegerLiteral(literal) => Some(TermToken::IntegerLiteral(literal)),
            Token::MissingOperand => Some(TermToken::MissingOperand),
            Token::NoExpression => Some(TermToken::NoExpression),
            _ => { assert_ne!(token.fixity(), Fixity::Term); None }
        }
    }
}

impl InfixToken {
    pub fn operator(self) -> Operators {
        match self {
            InfixOperator(identifier) => Operators::from(identifier),
            MissingInfix => Operators::Unrecognized,
        }
    }

    pub fn try_from(token: Token) -> Option<Self> {
        match token {
            Token::InfixOperator(identifier) => Some(InfixToken::InfixOperator(identifier)),
            Token::MissingInfix => Some(InfixToken::MissingInfix),
            _ => { assert_ne!(token.fixity(), Fixity::Infix); None }
        }
    }
}

impl PostfixToken {
    pub fn operator(self) -> Operators {
        match self {
            PostfixOperator(identifier)|Close(identifier) => Operators::from(identifier),
        }
    }

    pub fn try_from(token: Token) -> Option<Self> {
        match token {
            Token::PostfixOperator(identifier) => Some(PostfixToken::PostfixOperator(identifier)),
            Token::Close(identifier) => Some(PostfixToken::Close(identifier)),
            _ => { assert_ne!(token.fixity(), Fixity::Postfix); None }
        }
    }
}

impl PrefixToken {
    pub fn operator(self) -> Operators {
        match self {
            PrefixOperator(identifier)|Open(identifier) => Operators::from(identifier)
        }
    }
    pub fn try_from(token: Token) -> Option<Self> {
        match token {
            Token::PrefixOperator(identifier) => Some(PrefixToken::PrefixOperator(identifier)),
            Token::Open(identifier) => Some(PrefixToken::Open(identifier)),
            _ => { assert_ne!(token.fixity(), Fixity::Prefix); None }
        }
    }
}
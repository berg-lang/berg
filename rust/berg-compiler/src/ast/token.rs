use ast::{IdentifierIndex,LiteralIndex};
use ast::operators::Operators;
use ast::token::InfixToken::*;
use ast::token::PrefixToken::*;
use ast::token::PostfixToken::*;

// ExpressionType, String, LeftChild, RightChild
#[derive(Debug,Copy,Clone,PartialEq)]
pub enum Token {
    Term(TermToken),
    Infix(InfixToken),
    Prefix(PrefixToken),
    Postfix(PostfixToken),
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
            Term(_) => Fixity::Term,
            Infix(_) => Fixity::Infix,
            Prefix(_) => Fixity::Prefix,
            Postfix(_) => Fixity::Postfix,
        }
    }
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

impl InfixToken {
    pub fn operator(self) -> Operators {
        match self {
            InfixOperator(identifier) => Operators::from(identifier),
            MissingInfix => Operators::Unrecognized,
        }
    }
}

impl PrefixToken {
    pub fn operator(self) -> Operators {
        match self {
            PrefixOperator(identifier)|Open(identifier) => Operators::from(identifier),
        }
    }
}

impl PostfixToken {
    pub fn operator(self) -> Operators {
        match self {
            PostfixOperator(identifier)|Close(identifier) => Operators::from(identifier),
        }
    }
}

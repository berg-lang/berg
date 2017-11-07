use ast::{AstDelta,IdentifierIndex,LiteralIndex};
use ast::token::Token::*;

// ExpressionType, String, LeftChild, RightChild
#[derive(Debug,Copy,Clone,PartialEq)]
pub enum Token {
    IntegerLiteral(LiteralIndex),
    MissingExpression,

    InfixOperator(IdentifierIndex),
    MissingInfix,

    PrefixOperator(IdentifierIndex),
    OpenParen(AstDelta),
    OpenCompoundTerm(AstDelta),

    PostfixOperator(IdentifierIndex),
    CloseParen(AstDelta),
    CloseCompoundTerm(AstDelta),
}

#[derive(Debug,Copy,Clone,PartialEq)]
pub enum TermToken {
    IntegerLiteral(LiteralIndex),
    MissingExpression,
}

#[derive(Debug,Copy,Clone,PartialEq)]
pub enum InfixToken {
    InfixOperator(IdentifierIndex),
    MissingInfix,
}

#[derive(Debug,Copy,Clone,PartialEq)]
pub enum PrefixToken {
    PrefixOperator(IdentifierIndex),
    OpenParen(AstDelta),
    OpenCompoundTerm(AstDelta),
}

#[derive(Debug,Copy,Clone,PartialEq)]
pub enum OpenToken {
    OpenParen(AstDelta),
    OpenCompoundTerm(AstDelta),
}

#[derive(Debug,Copy,Clone,PartialEq)]
pub enum PostfixToken {
    PostfixOperator(IdentifierIndex),
    CloseParen(AstDelta),
    CloseCompoundTerm(AstDelta),
}

#[derive(Debug,Copy,Clone,PartialEq)]
pub enum CloseToken {
    CloseParen(AstDelta),
    CloseCompoundTerm(AstDelta),
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
            IntegerLiteral(_)|MissingExpression => Fixity::Term,
            InfixOperator(_)|MissingInfix => Fixity::Infix,
            PrefixOperator(_)|OpenParen(_)|OpenCompoundTerm(_) => Fixity::Prefix,
            PostfixOperator(_)|CloseParen(_)|CloseCompoundTerm(_) => Fixity::Postfix,
        }
    }
    pub fn to_term(self) -> Option<TermToken> { TermToken::try_from(self) }
    pub fn to_infix(self) -> Option<InfixToken> { InfixToken::try_from(self) }
    pub fn to_prefix(self) -> Option<PrefixToken> { PrefixToken::try_from(self) }
    pub fn to_open(self) -> Option<OpenToken> { OpenToken::try_from(self) }
    pub fn to_postfix(self) -> Option<PostfixToken> { PostfixToken::try_from(self) }
    pub fn to_close(self) -> Option<CloseToken> { CloseToken::try_from(self) }
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

impl From<TermToken> for Token {
    fn from(token: TermToken) -> Self {
        match token {
            TermToken::IntegerLiteral(literal) => IntegerLiteral(literal),
            TermToken::MissingExpression => MissingExpression,
        }
    }
}
impl TermToken {
    pub fn try_from(token: Token) -> Option<Self> {
        match token {
            IntegerLiteral(literal) => Some(TermToken::IntegerLiteral(literal)),
            MissingExpression => Some(TermToken::MissingExpression),
            InfixOperator(_)|MissingInfix|PrefixOperator(_)|OpenParen(_)|OpenCompoundTerm(_)|PostfixOperator(_)|CloseParen(_)|CloseCompoundTerm(_) => None,
        }
    }
}

impl From<InfixToken> for Token {
    fn from(token: InfixToken) -> Self {
        match token {
            InfixToken::InfixOperator(identifier) => InfixOperator(identifier),
            InfixToken::MissingInfix => MissingInfix,
        }
    }
}
impl InfixToken {
    pub fn try_from(token: Token) -> Option<Self> {
        match token {
            InfixOperator(identifier) => Some(InfixToken::InfixOperator(identifier)),
            MissingInfix => Some(InfixToken::MissingInfix),
            IntegerLiteral(_)|MissingExpression|PrefixOperator(_)|OpenParen(_)|OpenCompoundTerm(_)|PostfixOperator(_)|CloseParen(_)|CloseCompoundTerm(_) => None,
        }
    }
}

impl From<PrefixToken> for Token {
    fn from(token: PrefixToken) -> Self {
        match token {
            PrefixToken::PrefixOperator(identifier) => PrefixOperator(identifier),
            PrefixToken::OpenParen(delta) => OpenParen(delta),
            PrefixToken::OpenCompoundTerm(delta) => OpenCompoundTerm(delta),
        }
    }
}
impl PrefixToken {
    pub fn try_from(token: Token) -> Option<Self> {
        match token {
            PrefixOperator(identifier) => Some(PrefixToken::PrefixOperator(identifier)),
            OpenParen(delta) => Some(PrefixToken::OpenParen(delta)),
            OpenCompoundTerm(delta) => Some(PrefixToken::OpenCompoundTerm(delta)),
            IntegerLiteral(_)|MissingExpression|InfixOperator(_)|MissingInfix|PostfixOperator(_)|CloseParen(_)|CloseCompoundTerm(_) => None,
        }
    }
}

impl From<OpenToken> for Token {
    fn from(token: OpenToken) -> Self {
        match token {
            OpenToken::OpenParen(delta) => OpenParen(delta),
            OpenToken::OpenCompoundTerm(delta) => OpenCompoundTerm(delta),
        }
    }
}
impl OpenToken {
    pub fn try_from(token: Token) -> Option<Self> {
        match token {
            OpenParen(delta) => Some(OpenToken::OpenParen(delta)),
            OpenCompoundTerm(delta) => Some(OpenToken::OpenCompoundTerm(delta)),
            IntegerLiteral(_)|MissingExpression|InfixOperator(_)|MissingInfix|PrefixOperator(_)|PostfixOperator(_)|CloseParen(_)|CloseCompoundTerm(_) => None,
        }
    }
}

impl From<PostfixToken> for Token {
    fn from(token: PostfixToken) -> Self {
        match token {
            PostfixToken::PostfixOperator(identifier) => PostfixOperator(identifier),
            PostfixToken::CloseParen(delta) => CloseParen(delta),
            PostfixToken::CloseCompoundTerm(delta) => CloseCompoundTerm(delta),
        }
    }
}
impl PostfixToken {
    pub fn try_from(token: Token) -> Option<Self> {
        match token {
            PostfixOperator(identifier) => Some(PostfixToken::PostfixOperator(identifier)),
            CloseParen(delta) => Some(PostfixToken::CloseParen(delta)),
            CloseCompoundTerm(delta) => Some(PostfixToken::CloseCompoundTerm(delta)),
            IntegerLiteral(_)|MissingExpression|InfixOperator(_)|MissingInfix|PrefixOperator(_)|OpenParen(_)|OpenCompoundTerm(_) => None,
        }
    }
}

impl From<CloseToken> for Token {
    fn from(token: CloseToken) -> Self {
        match token {
            CloseToken::CloseParen(delta) => CloseParen(delta),
            CloseToken::CloseCompoundTerm(delta) => CloseCompoundTerm(delta),
        }
    }
}
impl CloseToken {
    pub fn try_from(token: Token) -> Option<Self> {
        match token {
            CloseParen(delta) => Some(CloseToken::CloseParen(delta)),
            CloseCompoundTerm(delta) => Some(CloseToken::CloseCompoundTerm(delta)),
            IntegerLiteral(_)|MissingExpression|InfixOperator(_)|MissingInfix|PrefixOperator(_)|OpenParen(_)|OpenCompoundTerm(_)|PostfixOperator(_) => None,
        }
    }
}

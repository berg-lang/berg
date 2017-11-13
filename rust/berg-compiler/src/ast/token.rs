use ast::{AstDelta,IdentifierIndex,LiteralIndex};
use ast::precedence::Precedence;
use ast::token::Token::*;

// ExpressionType, String, LeftChild, RightChild
#[derive(Debug,Copy,Clone,PartialEq)]
pub enum Token {
    IntegerLiteral(LiteralIndex),
    PropertyReference(IdentifierIndex),
    SyntaxErrorTerm(LiteralIndex),
    MissingExpression,

    InfixOperator(IdentifierIndex),
    NewlineSequence,
    MissingInfix,

    Open(ExpressionBoundary,AstDelta),
    PrefixOperator(IdentifierIndex),

    Close(ExpressionBoundary,AstDelta),
    PostfixOperator(IdentifierIndex),
}

#[derive(Debug,Copy,Clone,PartialEq)]
pub enum TermToken {
    IntegerLiteral(LiteralIndex),
    PropertyReference(IdentifierIndex),
    SyntaxErrorTerm(LiteralIndex),
    MissingExpression,
}

#[derive(Debug,Copy,Clone,PartialEq)]
pub enum InfixToken {
    InfixOperator(IdentifierIndex),
    NewlineSequence,
    MissingInfix,
}

#[derive(Debug,Copy,Clone,PartialEq)]
pub enum PrefixToken {
    Open(ExpressionBoundary,AstDelta),
    PrefixOperator(IdentifierIndex),
}

#[derive(Debug,Copy,Clone,PartialEq)]
pub enum PostfixToken {
    Close(ExpressionBoundary,AstDelta),
    PostfixOperator(IdentifierIndex),
}

#[derive(Debug,Copy,Clone,PartialEq,PartialOrd)]
pub enum ExpressionBoundary {
    PrecedenceGroup,
    CompoundTerm,
    Parentheses,
    File,
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
        use ast::token::Token::*;
        match *self {
            IntegerLiteral(_)|PropertyReference(_)|SyntaxErrorTerm(_)|MissingExpression => Fixity::Term,
            InfixOperator(_)|NewlineSequence|MissingInfix => Fixity::Infix,
            PrefixOperator(_)|Open(..) => Fixity::Prefix,
            PostfixOperator(_)|Close(..) => Fixity::Postfix,
        }
    }
    pub fn to_term(self) -> Option<TermToken> { TermToken::try_from(self) }
    pub fn to_infix(self) -> Option<InfixToken> { InfixToken::try_from(self) }
    pub fn to_prefix(self) -> Option<PrefixToken> { PrefixToken::try_from(self) }
    pub fn to_postfix(self) -> Option<PostfixToken> { PostfixToken::try_from(self) }
    pub fn num_operands(self) -> u8 { self.fixity().num_operands() }
    pub fn has_left_operand(self) -> bool { self.fixity().has_left_operand() }
    pub fn has_right_operand(self) -> bool { self.fixity().has_right_operand() }
}

impl ExpressionBoundary {
    pub(crate) fn placeholder_open_token(self) -> Token {
        Open(self, Default::default())
    }
    pub(crate) fn placeholder_close_token(self) -> Token {
        Close(self, Default::default())
    }
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
            TermToken::PropertyReference(identifier) => PropertyReference(identifier),
            TermToken::SyntaxErrorTerm(literal) => SyntaxErrorTerm(literal),
            TermToken::MissingExpression => MissingExpression,
        }
    }
}
impl TermToken {
    pub fn try_from(token: Token) -> Option<Self> {
        match token {
            IntegerLiteral(literal) => Some(TermToken::IntegerLiteral(literal)),
            SyntaxErrorTerm(literal) => Some(TermToken::SyntaxErrorTerm(literal)),
            MissingExpression => Some(TermToken::MissingExpression),
            PropertyReference(identifier) => Some(TermToken::PropertyReference(identifier)),
            InfixOperator(_)|NewlineSequence|MissingInfix|PrefixOperator(_)|Open(..)|PostfixOperator(_)|Close(..) => None,
        }
    }
}

impl From<InfixToken> for Token {
    fn from(token: InfixToken) -> Self {
        match token {
            InfixToken::InfixOperator(identifier) => InfixOperator(identifier),
            InfixToken::NewlineSequence => NewlineSequence,
            InfixToken::MissingInfix => MissingInfix,
        }
    }
}
impl InfixToken {
    pub fn try_from(token: Token) -> Option<Self> {
        match token {
            InfixOperator(identifier) => Some(InfixToken::InfixOperator(identifier)),
            NewlineSequence => Some(InfixToken::NewlineSequence),
            MissingInfix => Some(InfixToken::MissingInfix),
            IntegerLiteral(_)|PropertyReference(_)|SyntaxErrorTerm(_)|MissingExpression|PrefixOperator(_)|Open(..)|PostfixOperator(_)|Close(..) => None,
        }
    }
    pub fn precedence(self) -> Precedence {
        match self {
            InfixToken::InfixOperator(identifier) => Precedence::from(identifier),
            InfixToken::MissingInfix => Precedence::default(),
            InfixToken::NewlineSequence => Precedence::NewlineSequence,
        }
    }
    pub fn takes_right_child(self, right: InfixToken) -> bool {
        self.precedence().takes_right_child(right.precedence())
    }
    pub fn takes_left_child(self, left: InfixToken) -> bool {
        self.precedence().takes_left_child(left.precedence())
    }
}

impl From<PrefixToken> for Token {
    fn from(token: PrefixToken) -> Self {
        match token {
            PrefixToken::PrefixOperator(identifier) => PrefixOperator(identifier),
            PrefixToken::Open(boundary,delta) => Open(boundary,delta),
        }
    }
}
impl PrefixToken {
    pub fn try_from(token: Token) -> Option<Self> {
        match token {
            PrefixOperator(identifier) => Some(PrefixToken::PrefixOperator(identifier)),
            Open(boundary,delta) => Some(PrefixToken::Open(boundary,delta)),
            IntegerLiteral(_)|PropertyReference(_)|SyntaxErrorTerm(_)|MissingExpression|InfixOperator(_)|NewlineSequence|MissingInfix|PostfixOperator(_)|Close(..) => None,
        }
    }
}

impl From<PostfixToken> for Token {
    fn from(token: PostfixToken) -> Self {
        match token {
            PostfixToken::PostfixOperator(identifier) => PostfixOperator(identifier),
            PostfixToken::Close(boundary,delta) => Close(boundary,delta),
        }
    }
}
impl PostfixToken {
    pub fn try_from(token: Token) -> Option<Self> {
        match token {
            PostfixOperator(identifier) => Some(PostfixToken::PostfixOperator(identifier)),
            Close(boundary,delta) => Some(PostfixToken::Close(boundary,delta)),
            IntegerLiteral(_)|PropertyReference(_)|SyntaxErrorTerm(_)|MissingExpression|InfixOperator(_)|NewlineSequence|MissingInfix|PrefixOperator(_)|Open(..) => None,
        }
    }
}

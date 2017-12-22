use ast::{AstDelta,IdentifierIndex,LiteralIndex};
use ast::identifiers::*;
use ast::precedence::Precedence;
use ast::token::ExpressionBoundary::*;
use ast::token::Token::*;
use std::fmt;
use source::compile_errors::CompileErrorCode;

// ExpressionType, String, LeftChild, RightChild
#[derive(Debug,Copy,Clone,PartialEq)]
pub enum Token {
    IntegerLiteral(LiteralIndex),
    VariableReference(IdentifierIndex),
    ErrorTerm(CompileErrorCode),
    MissingExpression,

    InfixOperator(IdentifierIndex),
    InfixAssignment(IdentifierIndex),
    NewlineSequence,
    MissingInfix,

    PrefixOperator(IdentifierIndex),
    Open { delta: AstDelta, boundary: ExpressionBoundary, error: ExpressionBoundaryError },

    PostfixOperator(IdentifierIndex),
    Close { delta: AstDelta, boundary: ExpressionBoundary, error: ExpressionBoundaryError }
}

#[derive(Debug,Copy,Clone,PartialEq)]
pub enum TermToken {
    IntegerLiteral(LiteralIndex),
    VariableReference(IdentifierIndex),
    ErrorTerm(CompileErrorCode),
    MissingExpression,
}

#[derive(Debug,Copy,Clone,PartialEq)]
pub enum InfixToken {
    InfixOperator(IdentifierIndex),
    InfixAssignment(IdentifierIndex),
    NewlineSequence,
    MissingInfix,
}

#[derive(Debug,Copy,Clone,PartialEq)]
pub enum PrefixToken {
    Open { delta: AstDelta, boundary: ExpressionBoundary, error: ExpressionBoundaryError },
    PrefixOperator(IdentifierIndex),
}

#[derive(Debug,Copy,Clone,PartialEq)]
pub enum PostfixToken {
    Close { delta: AstDelta, boundary: ExpressionBoundary, error: ExpressionBoundaryError },
    PostfixOperator(IdentifierIndex),
}

#[derive(Debug,Copy,Clone,PartialEq,PartialOrd)]
pub enum ExpressionBoundary {
    PrecedenceGroup,
    CompoundTerm,
    Parentheses,
    CurlyBraces,
    Source,
}

#[derive(Debug,Copy,Clone,PartialEq)]
pub enum Fixity {
    Term,
    Infix,
    Prefix,
    Postfix,
}

#[derive(Debug,Copy,Clone,PartialEq)]
pub enum ExpressionBoundaryError {
    CloseWithoutOpen,
    OpenWithoutClose,
    OpenError, // Error opening or reading the source file
    None
}

impl Token {
    pub fn fixity(self) -> Fixity {
        match self {
            IntegerLiteral(_)|VariableReference(_)|ErrorTerm(_)|MissingExpression => Fixity::Term,
            InfixOperator(_)|InfixAssignment(_)|NewlineSequence|MissingInfix => Fixity::Infix,
            PrefixOperator(_)|Open{..} => Fixity::Prefix,
            PostfixOperator(_)|Close{..} => Fixity::Postfix,
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
    pub(crate) fn placeholder_open_token(self, error: ExpressionBoundaryError) -> Token {
        Open { boundary: self, delta: Default::default(), error }
    }
    pub(crate) fn placeholder_close_token(self, error: ExpressionBoundaryError) -> Token {
        Close { boundary: self, delta: Default::default(), error }
    }
    pub(crate) fn open_string(self) -> &'static str {
        match self {
            CurlyBraces => identifier_string(OPEN_CURLY),
            Parentheses => identifier_string(OPEN_PAREN),
            PrecedenceGroup|CompoundTerm|Source => "",
        }
    }
    pub(crate) fn close_string(self) -> &'static str {
        match self {
            CurlyBraces => identifier_string(CLOSE_CURLY),
            Parentheses => identifier_string(CLOSE_PAREN),
            PrecedenceGroup|CompoundTerm|Source => "",
        }
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

impl fmt::Display for Fixity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ast::token::Fixity::*;
        let fixity = match *self {
            Term => "term",
            Prefix => "unary",
            Infix => "binary",
            Postfix => "postfix"
        };
        write!(f, "{}", fixity)
    }
}

impl From<TermToken> for Token {
    fn from(token: TermToken) -> Self {
        match token {
            TermToken::IntegerLiteral(literal) => IntegerLiteral(literal),
            TermToken::VariableReference(identifier) => VariableReference(identifier),
            TermToken::ErrorTerm(code) => ErrorTerm(code),
            TermToken::MissingExpression => MissingExpression,
        }
    }
}

impl TermToken {
    pub fn try_from(token: Token) -> Option<Self> {
        match token {
            IntegerLiteral(literal) => Some(TermToken::IntegerLiteral(literal)),
            ErrorTerm(code) => Some(TermToken::ErrorTerm(code)),
            MissingExpression => Some(TermToken::MissingExpression),
            VariableReference(identifier) => Some(TermToken::VariableReference(identifier)),
            InfixOperator(_)|InfixAssignment(_)|NewlineSequence|MissingInfix|PrefixOperator(_)|Open{..}|PostfixOperator(_)|Close{..} => None,
        }
    }
}

impl From<InfixToken> for Token {
    fn from(token: InfixToken) -> Self {
        match token {
            InfixToken::InfixOperator(identifier) => InfixOperator(identifier),
            InfixToken::InfixAssignment(identifier) => InfixAssignment(identifier),
            InfixToken::NewlineSequence => NewlineSequence,
            InfixToken::MissingInfix => MissingInfix,
        }
    }
}
impl InfixToken {
    pub fn try_from(token: Token) -> Option<Self> {
        match token {
            InfixOperator(identifier) => Some(InfixToken::InfixOperator(identifier)),
            InfixAssignment(identifier) => Some(InfixToken::InfixAssignment(identifier)),
            NewlineSequence => Some(InfixToken::NewlineSequence),
            MissingInfix => Some(InfixToken::MissingInfix),
            IntegerLiteral(_)|VariableReference(_)|ErrorTerm(_)|MissingExpression|PrefixOperator(_)|Open{..}|PostfixOperator(_)|Close{..} => None,
        }
    }
    pub fn precedence(self) -> Precedence {
        use ast::token::InfixToken::*;
        use ast::token::Precedence::*;
        use ast::identifiers::*;
        match self {
            InfixOperator(operator) => match operator {
                STAR|SLASH => TimesDivide,
                EQUAL_TO|NOT_EQUAL_TO|GREATER_THAN|GREATER_EQUAL|LESS_THAN|LESS_EQUAL => Comparison,
                AND_AND => And,
                OR_OR => Or,
                SEMICOLON => SemicolonSequence,
                _ => Precedence::default(),
            },
            InfixAssignment(_) => Assign,
            InfixToken::NewlineSequence => Precedence::NewlineSequence,
            _ => Precedence::default(),
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
            PrefixToken::Open{boundary,delta,error} => Open{boundary,delta,error},
        }
    }
}
impl PrefixToken {
    pub fn try_from(token: Token) -> Option<Self> {
        match token {
            PrefixOperator(identifier) => Some(PrefixToken::PrefixOperator(identifier)),
            Open{boundary,delta,error} => Some(PrefixToken::Open{boundary,delta,error}),
            IntegerLiteral(_)|VariableReference(_)|ErrorTerm(_)|MissingExpression|InfixOperator(_)|InfixAssignment(_)|NewlineSequence|MissingInfix|PostfixOperator(_)|Close{..} => None,
        }
    }
}

impl From<PostfixToken> for Token {
    fn from(token: PostfixToken) -> Self {
        match token {
            PostfixToken::PostfixOperator(identifier) => PostfixOperator(identifier),
            PostfixToken::Close{boundary,delta,error} => Close{boundary,delta,error},
        }
    }
}
impl PostfixToken {
    pub fn try_from(token: Token) -> Option<Self> {
        match token {
            PostfixOperator(identifier) => Some(PostfixToken::PostfixOperator(identifier)),
            Close{boundary,delta,error} => Some(PostfixToken::Close{boundary,delta,error}),
            IntegerLiteral(_)|VariableReference(_)|ErrorTerm(_)|MissingExpression|InfixOperator(_)|InfixAssignment(_)|NewlineSequence|MissingInfix|PrefixOperator(_)|Open{..} => None,
        }
    }
}

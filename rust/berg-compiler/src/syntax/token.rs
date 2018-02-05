use error::ErrorCode;
use std::borrow::Cow;
use syntax::AstRef;
use syntax::BlockIndex;
use syntax::{AstDelta, FieldIndex, IdentifierIndex, LiteralIndex};
use syntax::ExpressionBoundary::*;
use syntax::Precedence;
use syntax::token::Token::*;
use syntax::identifiers::*;
use std::fmt;

// ExpressionType, String, LeftChild, RightChild
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Token {
    IntegerLiteral(LiteralIndex),
    FieldReference(FieldIndex),
    RawIdentifier(IdentifierIndex),
    ErrorTerm(ErrorCode),
    MissingExpression,

    InfixOperator(IdentifierIndex),
    InfixAssignment(IdentifierIndex),
    NewlineSequence,
    MissingInfix,

    PrefixOperator(IdentifierIndex),
    Open {
        delta: AstDelta,
        boundary: ExpressionBoundary,
        error: ExpressionBoundaryError,
    },
    OpenBlock {
        delta: AstDelta,
        index: BlockIndex,
        error: ExpressionBoundaryError,
    },

    PostfixOperator(IdentifierIndex),
    Close {
        delta: AstDelta,
        boundary: ExpressionBoundary,
        error: ExpressionBoundaryError,
    },
    CloseBlock {
        delta: AstDelta,
        index: BlockIndex,
        error: ExpressionBoundaryError,
    },
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ExpressionBoundaryError {
    CloseWithoutOpen,
    OpenWithoutClose,
    OpenError, // BergError opening or reading the source file
    None,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Fixity {
    Term,
    Infix,
    Prefix,
    Postfix,
    Open,
    Close,
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub enum ExpressionBoundary {
    PrecedenceGroup,
    CompoundTerm,
    Parentheses,
    CurlyBraces,
    Source,
    Root,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TermToken {
    IntegerLiteral(LiteralIndex),
    RawIdentifier(IdentifierIndex),
    FieldReference(FieldIndex),
    ErrorTerm(ErrorCode),
    MissingExpression,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum InfixToken {
    InfixOperator(IdentifierIndex),
    InfixAssignment(IdentifierIndex),
    NewlineSequence,
    MissingInfix,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PrefixToken {
    Open {
        delta: AstDelta,
        boundary: ExpressionBoundary,
        error: ExpressionBoundaryError,
    },
    OpenBlock {
        delta: AstDelta,
        index: BlockIndex,
        error: ExpressionBoundaryError,
    },
    PrefixOperator(IdentifierIndex),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PostfixToken {
    Close {
        delta: AstDelta,
        boundary: ExpressionBoundary,
        error: ExpressionBoundaryError,
    },
    CloseBlock {
        delta: AstDelta,
        index: BlockIndex,
        error: ExpressionBoundaryError,
    },
    PostfixOperator(IdentifierIndex),
}

impl Token {
    pub fn fixity(self) -> Fixity {
        match self {
            IntegerLiteral(_) | RawIdentifier(_) | FieldReference(_) | ErrorTerm(_)
            | MissingExpression => Fixity::Term,
            InfixOperator(_) | InfixAssignment(_) | NewlineSequence | MissingInfix => Fixity::Infix,
            PrefixOperator(_) => Fixity::Prefix,
            PostfixOperator(_) => Fixity::Postfix,
            Open { .. } | OpenBlock { .. } => Fixity::Open,
            Close { .. } | CloseBlock { .. } => Fixity::Close,
        }
    }
    pub fn to_term(self) -> Option<TermToken> {
        TermToken::try_from(self)
    }
    pub fn to_infix(self) -> Option<InfixToken> {
        InfixToken::try_from(self)
    }
    pub fn to_prefix(self) -> Option<PrefixToken> {
        PrefixToken::try_from(self)
    }
    pub fn to_postfix(self) -> Option<PostfixToken> {
        PostfixToken::try_from(self)
    }
    pub fn num_operands(self) -> u8 {
        self.fixity().num_operands()
    }
    pub fn has_left_operand(self) -> bool {
        self.fixity().has_left_operand()
    }
    pub fn has_right_operand(self) -> bool {
        self.fixity().has_right_operand()
    }
    pub fn to_string<'p, 'a: 'p>(&'p self, ast: &'p AstRef<'a>) -> Cow<'p, str> {
        match *self {
            IntegerLiteral(literal) => ast.literal_string(literal).into(),
            ErrorTerm(_) => "error".into(),

            FieldReference(field) => ast.identifier_string(ast.fields()[field].name).into(),

            RawIdentifier(identifier)
            | InfixOperator(identifier)
            | PostfixOperator(identifier)
            | PrefixOperator(identifier) => ast.identifier_string(identifier).into(),

            InfixAssignment(identifier) => format!("{}=", ast.identifier_string(identifier)).into(),

            NewlineSequence => "\\n".into(),
            Open { boundary, .. } => match boundary {
                Parentheses => ast.identifier_string(OPEN_PAREN).into(),
                CurlyBraces => ast.identifier_string(OPEN_CURLY).into(),
                CompoundTerm | PrecedenceGroup | Source | Root => "".into(),
            },
            OpenBlock { index, .. } => match ast.blocks()[index].boundary {
                Parentheses => ast.identifier_string(OPEN_PAREN).into(),
                CurlyBraces => ast.identifier_string(OPEN_CURLY).into(),
                CompoundTerm | PrecedenceGroup | Source | Root => "".into(),
            }
            Close { boundary, .. } => match boundary {
                Parentheses => ast.identifier_string(CLOSE_PAREN).into(),
                CurlyBraces => ast.identifier_string(CLOSE_CURLY).into(),
                CompoundTerm | PrecedenceGroup | Source | Root => "".into(),
            },
            CloseBlock { index, .. } => match ast.blocks()[index].boundary {
                Parentheses => ast.identifier_string(CLOSE_PAREN).into(),
                CurlyBraces => ast.identifier_string(CLOSE_CURLY).into(),
                CompoundTerm | PrecedenceGroup | Source | Root => "".into(),
            },
            MissingExpression | MissingInfix => "".into(),
        }
    }
}

impl ExpressionBoundary {
    pub(crate) fn is_scope(&self) -> bool {
        match *self {
            CurlyBraces | Source | Root => true,
            Parentheses | PrecedenceGroup | CompoundTerm => false,
        }
    }
    pub(crate) fn placeholder_open_token(self, error: ExpressionBoundaryError) -> Token {
        Open {
            boundary: self,
            delta: Default::default(),
            error,
        }
    }
    pub(crate) fn placeholder_close_token(self, error: ExpressionBoundaryError) -> Token {
        Close {
            boundary: self,
            delta: Default::default(),
            error,
        }
    }
    pub(crate) fn open_string(self) -> &'static str {
        match self {
            CurlyBraces => identifier_string(OPEN_CURLY),
            Parentheses => identifier_string(OPEN_PAREN),
            PrecedenceGroup | CompoundTerm | Source | Root => "",
        }
    }
    pub(crate) fn close_string(self) -> &'static str {
        match self {
            CurlyBraces => identifier_string(CLOSE_CURLY),
            Parentheses => identifier_string(CLOSE_PAREN),
            PrecedenceGroup | CompoundTerm | Source | Root => "",
        }
    }
}

impl Fixity {
    pub fn num_operands(&self) -> u8 {
        use syntax::Fixity::*;
        match *self {
            Term => 0,
            Prefix | Postfix | Open | Close=> 1,
            Infix => 2,
        }
    }
    pub fn has_left_operand(&self) -> bool {
        use syntax::Fixity::*;
        match *self {
            Term | Prefix | Open => false,
            Infix | Postfix | Close => true,
        }
    }
    pub fn has_right_operand(&self) -> bool {
        use syntax::Fixity::*;
        match *self {
            Term | Postfix | Close => false,
            Infix | Prefix | Open => true,
        }
    }
}

impl fmt::Display for Fixity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use syntax::Fixity::*;
        let fixity = match *self {
            Term => "term",
            Prefix => "unary",
            Infix => "binary",
            Open => "open",
            Close => "close",
            Postfix => "postfix",
        };
        write!(f, "{}", fixity)
    }
}

impl From<TermToken> for Token {
    fn from(token: TermToken) -> Self {
        match token {
            TermToken::IntegerLiteral(literal) => IntegerLiteral(literal),
            TermToken::RawIdentifier(field) => RawIdentifier(field),
            TermToken::FieldReference(field) => FieldReference(field),
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
            RawIdentifier(identifier) => Some(TermToken::RawIdentifier(identifier)),
            FieldReference(field) => Some(TermToken::FieldReference(field)),
            InfixOperator(_)
            | InfixAssignment(_)
            | NewlineSequence
            | MissingInfix
            | PrefixOperator(_)
            | Open { .. }
            | OpenBlock { .. }
            | PostfixOperator(_)
            | Close { .. }
            | CloseBlock { .. } => None,
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
            IntegerLiteral(_)
            | RawIdentifier(_)
            | FieldReference(_)
            | ErrorTerm(_)
            | MissingExpression
            | PrefixOperator(_)
            | Open { .. }
            | OpenBlock { .. }
            | PostfixOperator(_)
            | Close { .. }
            | CloseBlock { .. } => None,
        }
    }
    pub fn precedence(self) -> Precedence {
        use syntax::InfixToken::*;
        use syntax::Precedence::*;
        use syntax::identifiers::*;
        match self {
            InfixOperator(operator) => match operator {
                STAR | SLASH => TimesDivide,
                EQUAL_TO | NOT_EQUAL_TO | GREATER_THAN | GREATER_EQUAL | LESS_THAN | LESS_EQUAL => {
                    Comparison
                }
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
            PrefixToken::Open {
                boundary,
                delta,
                error,
            } => Open {
                boundary,
                delta,
                error,
            },
            PrefixToken::OpenBlock {
                delta,
                index,
                error,
            } => OpenBlock {
                delta,
                index,
                error,
            },
        }
    }
}
impl PrefixToken {
    pub fn try_from(token: Token) -> Option<Self> {
        match token {
            PrefixOperator(identifier) => Some(PrefixToken::PrefixOperator(identifier)),
            Open {
                boundary,
                delta,
                error,
            } => Some(PrefixToken::Open {
                boundary,
                delta,
                error,
            }),
            OpenBlock {
                delta,
                index,
                error,
            } => Some(PrefixToken::OpenBlock {
                delta,
                index,
                error,
            }),
            IntegerLiteral(_)
            | RawIdentifier(_)
            | FieldReference(_)
            | ErrorTerm(_)
            | MissingExpression
            | InfixOperator(_)
            | InfixAssignment(_)
            | NewlineSequence
            | MissingInfix
            | PostfixOperator(_)
            | Close { .. }
            | CloseBlock { .. } => None,
        }
    }
}

impl From<PostfixToken> for Token {
    fn from(token: PostfixToken) -> Self {
        match token {
            PostfixToken::PostfixOperator(identifier) => PostfixOperator(identifier),
            PostfixToken::Close {
                boundary,
                delta,
                error,
            } => Close {
                boundary,
                delta,
                error,
            },
            PostfixToken::CloseBlock {
                delta,
                index,
                error,
            } => CloseBlock {
                delta,
                index,
                error,
            },
        }
    }
}
impl PostfixToken {
    pub fn try_from(token: Token) -> Option<Self> {
        match token {
            PostfixOperator(identifier) => Some(PostfixToken::PostfixOperator(identifier)),
            Close {
                boundary,
                delta,
                error,
            } => Some(PostfixToken::Close {
                boundary,
                delta,
                error,
            }),
            CloseBlock {
                delta,
                index,
                error,
            } => Some(PostfixToken::CloseBlock {
                delta,
                index,
                error,
            }),
            IntegerLiteral(_)
            | RawIdentifier(_)
            | FieldReference(_)
            | ErrorTerm(_)
            | MissingExpression
            | InfixOperator(_)
            | InfixAssignment(_)
            | NewlineSequence
            | MissingInfix
            | PrefixOperator(_)
            | Open { .. }
            | OpenBlock { .. } => None,
        }
    }
}

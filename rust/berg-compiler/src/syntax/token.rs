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
    AutoBlock,
    CompoundTerm,
    Parentheses,
    CurlyBraces,
    Source,
    Root,
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
                CompoundTerm | PrecedenceGroup | AutoBlock | Source | Root => "".into(),
            },
            OpenBlock { index, .. } => match ast.blocks()[index].boundary {
                Parentheses => ast.identifier_string(OPEN_PAREN).into(),
                CurlyBraces => ast.identifier_string(OPEN_CURLY).into(),
                CompoundTerm | PrecedenceGroup | AutoBlock | Source | Root => "".into(),
            }
            Close { boundary, .. } => match boundary {
                Parentheses => ast.identifier_string(CLOSE_PAREN).into(),
                CurlyBraces => ast.identifier_string(CLOSE_CURLY).into(),
                CompoundTerm | PrecedenceGroup | AutoBlock | Source | Root => "".into(),
            },
            CloseBlock { index, .. } => match ast.blocks()[index].boundary {
                Parentheses => ast.identifier_string(CLOSE_PAREN).into(),
                CurlyBraces => ast.identifier_string(CLOSE_CURLY).into(),
                CompoundTerm | PrecedenceGroup | AutoBlock | Source | Root => "".into(),
            },
            MissingExpression | MissingInfix => "".into(),
        }
    }
    pub fn takes_right_child(self, right: Token) -> bool {
        use syntax::Fixity::*;
        match self.fixity() {
            Infix => match right.fixity() {
                Infix => Precedence::from(self).takes_right_child(Precedence::from(right)),
                _ => true,
            },
            Prefix => match right.fixity() {
                Prefix|Term|Open|Close => true,
                Infix|Postfix => false,
            }
            Term|Postfix => false,
            Open|Close => unreachable!(),
        }
    }
    pub fn takes_left_child(self, left: Token) -> bool {
        use syntax::Fixity::*;
        match self.fixity() {
            Infix => match left.fixity() {
                Infix => Precedence::from(self).takes_left_child(Precedence::from(left)),
                _ => true,
            },
            Postfix => match left.fixity() {
                Prefix|Postfix|Term|Open|Close => true,
                Infix => false,
            }
            Term|Prefix => false,
            Open|Close => unreachable!(),
        }
    }
    pub fn delta(self) -> AstDelta {
        match self {
            Open { delta, .. } | OpenBlock { delta, .. } | Close { delta, .. } | CloseBlock { delta, .. } => delta,
            _ => unreachable!(),
        }
    }
}

impl ExpressionBoundary {
    /// Tells whether this expression boundary represents a scope.
    pub(crate) fn is_scope(&self) -> bool {
        match *self {
            CurlyBraces | Source | Root | AutoBlock => true,
            Parentheses | PrecedenceGroup | CompoundTerm => false,
        }
    }
    /// Tells whether this boundary type MUST be in the expression tree (because
    /// it represents actual user syntax, or opens a scope).
    pub(crate) fn is_required(&self) -> bool {
        match *self {
            Root | AutoBlock | Source | CurlyBraces | Parentheses => true,
            PrecedenceGroup | CompoundTerm => false,
        }
    }
    /// Tells whether we expect a close token for this boundary or if it's handled
    /// by the grouper automatically.
    pub(crate) fn is_closed_automatically(&self) -> bool {
        match *self {
            PrecedenceGroup | CompoundTerm | AutoBlock => true,
            Root | Source | CurlyBraces | Parentheses => false,
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
            PrecedenceGroup | AutoBlock | CompoundTerm | Source | Root => "",
        }
    }
    pub(crate) fn close_string(self) -> &'static str {
        match self {
            CurlyBraces => identifier_string(CLOSE_CURLY),
            Parentheses => identifier_string(CLOSE_PAREN),
            PrecedenceGroup | AutoBlock | CompoundTerm | Source | Root => "",
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

use crate::syntax::identifiers::*;
use crate::syntax::precedence::Precedence;
use crate::syntax::ExpressionBoundary::*;
use crate::syntax::{
    Ast, AstDelta, BlockIndex, FieldIndex, Fixity, ExpressionFixity, OperatorFixity, IdentifierIndex, LiteralIndex, RawLiteralIndex,
};
use crate::value::ErrorCode;
use std::borrow::Cow;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Token {
    Expression(ExpressionToken),
    Operator(OperatorToken)
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ExpressionToken {
    IntegerLiteral(LiteralIndex),
    FieldReference(FieldIndex),
    RawIdentifier(IdentifierIndex),
    ErrorTerm(ErrorCode, LiteralIndex),
    RawErrorTerm(ErrorCode, RawLiteralIndex),
    MissingExpression,

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
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum OperatorToken {
    InfixOperator(IdentifierIndex),
    InfixAssignment(IdentifierIndex),
    NewlineSequence,
    Apply,

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

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub enum ExpressionBoundary {
    PrecedenceGroup,
    CompoundTerm,
    Parentheses,
    AutoBlock,
    CurlyBraces,
    Source,
    Root,
}

impl Token {
    pub fn fixity(self) -> Fixity {
        match self {
            Token::Expression(token) => token.fixity().into(),
            Token::Operator(token) => token.fixity().into(),
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
    pub fn to_string<'p, 'a: 'p>(&self, ast: &'p Ast<'a>) -> Cow<'p, str> {
        match *self {
            Token::Expression(token) => token.to_string(ast),
            Token::Operator(token) => token.to_string(ast),
        }
    }
    pub fn takes_right_child(self, right: impl Into<Token>) -> bool {
        match self {
            Token::Operator(token) => token.takes_right_child(right),
            Token::Expression(token) => token.takes_right_child(right),
        }
    }
    pub fn delta(self) -> AstDelta {
        match self {
            Token::Expression(token) => token.delta(),
            Token::Operator(token) => token.delta(),
        }
    }
}

impl ExpressionToken {
    pub fn fixity(self) -> ExpressionFixity {
        use ExpressionToken::*;
        match self {
            IntegerLiteral(_) | RawIdentifier(_) | FieldReference(_) | ErrorTerm(..)
            | RawErrorTerm(..) | MissingExpression => ExpressionFixity::Term,
            PrefixOperator(_) => ExpressionFixity::Prefix,
            Open { .. } | OpenBlock { .. } => ExpressionFixity::Open,
        }
    }
    pub fn num_operands(self) -> u8 {
        self.fixity().num_operands()
    }
    pub fn has_right_operand(self) -> bool {
        self.fixity().has_right_operand()
    }
    pub fn to_string<'p, 'a: 'p>(&self, ast: &'p Ast<'a>) -> Cow<'p, str> {
        use ExpressionToken::*;
        match *self {
            IntegerLiteral(literal) => ast.literal_string(literal).into(),
            ErrorTerm(code, ..) | RawErrorTerm(code, ..) => format!("error({:?})", code).into(),

            FieldReference(field) => ast.identifier_string(ast.fields[field].name).into(),

            RawIdentifier(identifier)
            | PrefixOperator(identifier) => ast.identifier_string(identifier).into(),

            Open { boundary, .. } => boundary.open_string().into(),
            OpenBlock { index, .. } => ast.blocks[index].boundary.open_string().into(),
            MissingExpression  => "".into(),
        }
    }
    pub fn takes_right_child(self, right: impl Into<Token>) -> bool {
        self.fixity().takes_right_child(right.into().fixity())
    }
    pub fn delta(self) -> AstDelta {
        use ExpressionToken::*;
        match self {
            Open { delta, .. }
            | OpenBlock { delta, .. } => delta,
            _ => unreachable!(),
        }
    }
}

impl OperatorToken {
    pub fn fixity(self) -> OperatorFixity {
        use OperatorToken::*;
        match self {
            InfixOperator(_) | InfixAssignment(_) | Apply | NewlineSequence => OperatorFixity::Infix,
            PostfixOperator(_) => OperatorFixity::Postfix,
            Close { .. } | CloseBlock { .. } => OperatorFixity::Close,
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
    pub fn to_string<'p, 'a: 'p>(&self, ast: &'p Ast<'a>) -> Cow<'p, str> {
        use OperatorToken::*;
        match *self {
            InfixOperator(identifier)
            | PostfixOperator(identifier) => ast.identifier_string(identifier).into(),

            InfixAssignment(identifier) => format!("{}=", ast.identifier_string(identifier)).into(),

            NewlineSequence => "\\n".into(),
            Close { boundary, .. } => boundary.close_string().into(),
            CloseBlock { index, .. } => ast.blocks[index].boundary.close_string().into(),
            Apply => "".into(),
        }
    }
    pub fn takes_right_child(self, right: impl Into<Token>) -> bool {
        use OperatorFixity::*;
        match (self.fixity(), right.into()) {
            (Infix, Token::Operator(right)) if right.fixity() == Infix => Precedence::from(self).takes_right_child(Precedence::from(right)),
            (left, right) => left.takes_right_child(right.fixity())
        }
    }
    pub fn delta(self) -> AstDelta {
        use OperatorToken::*;
        match self {
            Close { delta, .. }
            | CloseBlock { delta, .. } => delta,
            _ => unreachable!(),
        }
    }
}

impl ExpressionBoundary {
    /// Tells whether this expression boundary represents a scope.
    pub(crate) fn is_scope(self) -> bool {
        match self {
            CurlyBraces | Source | Root | AutoBlock => true,
            Parentheses | PrecedenceGroup | CompoundTerm => false,
        }
    }
    /// Tells whether this boundary type MUST be in the expression tree (because
    /// it represents actual user syntax, or opens a scope).
    pub(crate) fn is_required(self) -> bool {
        match self {
            Root | AutoBlock | Source | CurlyBraces | Parentheses => true,
            PrecedenceGroup | CompoundTerm => false,
        }
    }
    /// Tells whether we expect a close token for this boundary or if it's handled
    /// by the grouper automatically.
    pub(crate) fn is_closed_automatically(self) -> bool {
        match self {
            PrecedenceGroup | CompoundTerm | AutoBlock => true,
            Root | Source | CurlyBraces | Parentheses => false,
        }
    }
    pub(crate) fn placeholder_open_token(self, error: ExpressionBoundaryError) -> ExpressionToken {
        ExpressionToken::Open {
            boundary: self,
            delta: Default::default(),
            error,
        }
    }
    pub(crate) fn placeholder_close_token(self, error: ExpressionBoundaryError) -> OperatorToken {
        OperatorToken::Close {
            boundary: self,
            delta: Default::default(),
            error,
        }
    }
    pub(crate) fn open_string(self) -> &'static str {
        match self {
            CurlyBraces => OPEN_CURLY.well_known_str(),
            Parentheses => OPEN_PAREN.well_known_str(),
            PrecedenceGroup | AutoBlock | CompoundTerm | Source | Root => "",
        }
    }
    pub(crate) fn close_string(self) -> &'static str {
        match self {
            CurlyBraces => CLOSE_CURLY.well_known_str(),
            Parentheses => CLOSE_PAREN.well_known_str(),
            PrecedenceGroup | AutoBlock | CompoundTerm | Source | Root => "",
        }
    }
}

impl From<ExpressionToken> for Token {
    fn from(from: ExpressionToken) -> Token {
        Token::Expression(from)
    }
}

impl From<OperatorToken> for Token {
    fn from(from: OperatorToken) -> Token {
        Token::Operator(from)
    }
}

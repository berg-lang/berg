use berg_util::Delta;
use std::borrow::Cow;
use std::fmt;

use crate::bytes::ByteIndex;

use super::{
    ast::{Ast, AstDelta, LiteralIndex, RawLiteralIndex},
    block::{BlockIndex, FieldIndex},
    identifiers,
    precedence::Precedence,
};
use identifiers::*;
use ExpressionBoundary::*;

///
/// One atomic unit of an expression.
///
/// This is what is stored in [`AstData::tokens`].
///
#[derive(Copy, Clone, PartialEq)]
pub enum Token {
    Expression(ExpressionToken),
    Operator(OperatorToken),
}

// TODO because of the nested enums, Token is 16 bytes, even though all variants are < 6 bytes.
// Rust core is thinking about (but has not fixed) this case, but we can fix it internally by not
// nesting enums :(
// This test exists to make sure we don't regress further.
#[test]
fn token_size_is_12bytes_even_though_we_want_it_to_be_8() {
    use std::mem::size_of;
    assert_eq!(size_of::<Token>(), 12);
    assert_eq!(size_of::<ExpressionToken>(), 12);
    assert_eq!(size_of::<TermToken>(), 8);
    assert_eq!(size_of::<OperatorToken>(), 8);
}

///
/// An atomic unit of an expression that has no left operand.
///
/// For example, `1`, `true` and the `-` in `-1`.
///
/// Returned by [`AstData::expression_token()`], which is generally used
/// by forward-moving walkers that know for certain that the next token is
/// an expression token since the previous one requires an operand. Since the
/// tree is well-formed, the fact that it is a token is absolutely certain.
///
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ExpressionToken {
    ///
    /// A term, or atomic noun, such as `123` or `abc`.
    ///
    Term(TermToken),
    ///
    /// A prefix operator, such as the `-` in `-1`.
    ///
    /// The [`IdentifierIndex`] refers to the operator itself (like `-`). For a
    /// list of standard operators to compare against, look in
    /// [`syntax::identifiers`].
    ///
    PrefixOperator(IdentifierIndex),
    ///
    /// An open operator, such as `(` or `{`.
    ///
    /// The [`ExpressionBoundaryError`] indicates that there was an error parsing
    /// the operation, such as an open operator without a close, or a close operator
    /// without an open.
    ///
    /// The [`ExpressionBoundary`] indicates the kind of group (whether it was
    /// parentheses, curly braces, or even automatic blocks like compound terms,
    /// the automatic block after `:`, and precedence groups.
    ///
    /// The [`AstDelta`] is the distance to the close token. Use
    /// `ast.close_token(index + delta)` or `ast.close_block_token(index + delta)`
    /// to get to the close token (depending on boundary.is_block()).
    ///
    Open(
        Option<ExpressionBoundaryError>,
        ExpressionBoundary,
        AstDelta,
    ),
}

///
/// An atomic unit of an expression that has a left operand.
///
/// For example, `)`, the `+` in `1 + 2` and the `++` in `a++`.
///
/// Returned by [`AstData::operator_token()`], which is generally used
/// by forward-moving walkers that know for certain that the next token is
/// an expression token since the previous one requires an operand. Since the
/// tree is well-formed, the fact that it is a token is absolutely certain.
///
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum OperatorToken {
    ///
    /// An infix operator.
    ///
    /// For example, the `+` in `a + b`.
    ///
    /// The [`IdentifierIndex`] refers to the operator itself (like `+`). For a
    /// list of standard operators to compare against, look in
    /// [`syntax::identifiers`].
    ///
    InfixOperator(IdentifierIndex),
    ///
    /// An infix assignment operator.
    ///
    /// For example, the `+=` in `a += b`.
    ///
    /// The [`IdentifierIndex`] refers to the non-assignment version of the
    /// operation (i.e. `+` instead of `+=`), so that one can easily
    /// invoke the readonly version of the operation and then perform the
    /// assignment. In `a = b`, the identifier will refer to
    /// [`syntax::identifiers::EMPTY_STRING`].
    ///
    /// For a list of standard operators to compare against, look in
    /// [`syntax::identifiers`].
    ///
    InfixAssignment(IdentifierIndex),
    ///
    /// Delimiter indicating an inline block.
    ///
    /// Tuple is (level, repeat) where repeat is the number of times the = or - sign was
    /// repeated, and level is the level of the header (1 or 2).
    ///
    InlineBlockDelimiter(InlineBlockLevel, Delta<ByteIndex>),
    ///
    /// A postfix operator, such as the `++` in `a++`.
    ///
    /// The [`IdentifierIndex`] refers to the operator itself (like `++`). For a
    /// list of standard operators to compare against, look in
    /// [`syntax::identifiers`].
    ///
    PostfixOperator(IdentifierIndex),
    ///
    /// A close token for a *non-block*, such as `()` or a compound term.
    ///
    /// The [`AstDelta`] gives the distance to the corresponding
    /// `PrefixToken::Open`. Use `ast.open_token(index - delta)` to get to the
    /// open token.
    ///
    /// The [`ExpressionBoundary`] indicates the kind of group (whether it was
    /// parentheses, or even automatic blocks like compound terms,
    /// the automatic block after `:`, and precedence groups.
    ///
    Close(AstDelta, ExpressionBoundary),
    ///
    /// The [`AstDelta`] gives the distance to the corresponding
    /// `PostfixToken::Close` or `PostfixToken::CloseBlock`.
    ///
    /// The [`BlockIndex`] can be used to look up block-specific data like
    /// fields from [`ast::blocks`].
    ///
    /// Use `ast.open_token(index - ast.blocks[block_index].delta)` to get to
    /// the open token.
    ///
    CloseBlock(BlockIndex, ExpressionBoundary),
}

///
/// An atomic unit of an expression with no operands (a noun).
///
/// For example, `a` or `1`.
///
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TermToken {
    ///
    /// Integer (`1` or `1210312`).
    ///
    /// The [`LiteralIndex`] can be used to look up the actual string of digits
    /// in [`AstData::literals`].
    ///
    /// # Note
    ///
    /// `-1` is not an integer literal: it is prefix `-` followed by integer
    /// literal `1`.
    ///
    IntegerLiteral(LiteralIndex),
    ///
    /// A reference to a field.
    ///
    /// The [`FieldIndex`] can be used to look up the field name and publicity
    /// from [`AstData::fields`].
    ///
    /// This differs from [`RawIdentifier`] in that it is bound to a particular
    /// block: in `{ a: 1; 1 + a + { a: 2; 2 + a } }, `1 + a` refers to the first
    /// `a`, and `2 + a` refers to the second `a`.
    ///
    FieldReference(FieldIndex),
    ///
    /// An identifier.
    ///
    /// Used as the right hand side of `.`, for example the `a` in `x.a`. For
    /// raw variable references without a `.`, a [`FieldReference`] will be
    /// produced.
    ///
    /// The [`IdentifierIndex`] is globally unique per name and can be compared
    /// for equality against another `IdentifierIndex`.
    ///
    RawIdentifier(IdentifierIndex),
    ///
    /// An unparseable set of text.
    ///
    /// Used for things we don't understand that are nonetheless valid
    /// UTF-8, such as `123abc`.
    ///
    /// The [`LiteralIndex`] here can be used to look up the actual string that
    /// caused the error from [`AstData::literals`].
    ///
    ErrorTerm(ErrorTermError, LiteralIndex),
    ///
    /// An unparseable set of non-UTF-8 text.
    ///
    /// Berg only supports UTF-8, so when an invalid UTF-8 byte sequence is found,
    /// this is used to record the error instead of `ErrorTerm`.
    ///
    /// The [`RawLiteralIndex`] here can be used to look up the actual string that
    /// caused the error from [`AstData::raw_literals`].
    ///
    RawErrorTerm(RawErrorTermError, RawLiteralIndex),
    ///
    /// Used when the source has an operator with no operand.
    ///
    /// For example, `(a + )` has a `MissingExpression` just in front of the `)`.
    ///
    MissingExpression,
}

///
/// Inline block level.
///
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum InlineBlockLevel {
    One,
    Two,
}

///
/// Indicates an error making us uncertain of the block's contents.
///
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ExpressionBoundaryError {
    ///
    /// Indicates a close operator was found (such as `)`) but that no corresponding
    /// open operator.
    ///
    /// For example: `1 + 2)`
    ///
    CloseWithoutOpen,
    ///
    /// Indicates an open operator was found (such as `(`) but that no corresponding
    /// close operator.
    ///
    /// For example, `(1 + 2`.
    ///
    OpenWithoutClose,
}

///
/// The type of an open/close pair.
///
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExpressionBoundary {
    /// Used when the thing on the right binds tighter: x + y * z has a group for y * z
    PrecedenceGroup,
    /// For a group of things with no spaces: a+b * c+d has groups for a+b and c+d
    CompoundTerm,
    /// (b + c)
    Parentheses,
    /// After a colon: a: b+c has an auto-block for b+c
    AutoBlock,
    /// { b + c }
    CurlyBraces,
    /// a + b +
    ///   c + d
    IndentedExpression,
    /// if a == b
    ///   c = d
    IndentedBlock,
    /// The top-level group for the source file, the scope in which top-level names are defined
    Source,
    /// The top-level group, used to give the evaluator a place to attach external keywords and variables
    Root,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Fixity {
    Term,
    Infix,
    Prefix,
    Postfix,
    Open,
    Close,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ErrorTermError {
    IdentifierStartsWithNumber,
    UnsupportedCharacters,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum RawErrorTermError {
    InvalidUtf8,
}

impl Token {
    pub fn fixity(self) -> Fixity {
        use Token::*;
        match self {
            Expression(token) => token.fixity(),
            Operator(token) => token.fixity(),
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
    pub fn to_string(self, ast: &Ast) -> Cow<str> {
        use Token::*;
        match self {
            Expression(token) => token.to_string(ast),
            Operator(token) => token.to_string(ast),
        }
    }
    pub fn to_visible_string(self, ast: &Ast) -> Cow<str> {
        use Token::*;
        match self {
            Expression(token) => token.to_visible_string(ast),
            Operator(token) => token.to_visible_string(ast),
        }
    }
    pub fn takes_right_child(self, right: impl Into<Token>) -> bool {
        use Token::*;
        match self {
            Operator(token) => token.takes_right_child(right),
            Expression(token) => token.takes_right_child(right),
        }
    }
    pub fn original_bytes(self, ast: &Ast) -> Cow<[u8]> {
        use Token::*;
        match self {
            Expression(token) => token.original_bytes(ast),
            Operator(token) => token.original_bytes(ast),
        }
    }
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Token::*;
        match self {
            Expression(token) => write!(f, "{:?}", token),
            Operator(token) => write!(f, "{:?}", token),
        }
    }
}

impl ExpressionToken {
    pub fn fixity(self) -> Fixity {
        use ExpressionToken::*;
        match self {
            Term(_) => Fixity::Term,
            PrefixOperator(_) => Fixity::Prefix,
            Open(..) => Fixity::Open,
        }
    }
    pub fn num_operands(self) -> u8 {
        self.fixity().num_operands()
    }
    pub fn has_right_operand(self) -> bool {
        self.fixity().has_right_operand()
    }
    pub fn to_string(self, ast: &Ast) -> Cow<str> {
        use ExpressionBoundaryError::*;
        use ExpressionToken::*;
        match self {
            Term(token) => token.to_string(ast),
            PrefixOperator(identifier) => ast.identifier_string(identifier).into(),
            Open(Some(CloseWithoutOpen), boundary, _) => {
                format!("missing {}", boundary.open_string()).into()
            }
            Open(Some(OpenWithoutClose), boundary, _) => {
                format!("unclosed {}", boundary.open_string()).into()
            }
            Open(None, boundary, _) => boundary.open_string().into(),
        }
    }
    pub fn to_visible_string(self, ast: &Ast) -> Cow<str> {
        use ExpressionBoundaryError::*;
        use ExpressionToken::*;
        match self {
            Term(token) => token.to_visible_string(ast),
            Open(Some(CloseWithoutOpen), boundary, _) => {
                format!("missing {}", boundary.visible_open_string()).into()
            }
            Open(Some(OpenWithoutClose), boundary, _) => {
                format!("unclosed {}", boundary.visible_open_string()).into()
            }
            Open(None, boundary, _) => boundary.visible_open_string().into(),
            _ => self.to_string(ast),
        }
    }
    pub fn takes_right_child(self, right: impl Into<Token>) -> bool {
        self.fixity().takes_right_child(right.into().fixity())
    }
    pub fn original_bytes(self, ast: &Ast) -> Cow<[u8]> {
        use ExpressionToken::*;
        match self {
            Term(token) => token.original_bytes(ast),
            PrefixOperator(identifier) => ast.identifier_string(identifier).as_bytes().into(),
            Open(_, boundary, _) => boundary.open_string().as_bytes().into(),
        }
    }
}

impl OperatorToken {
    pub fn fixity(self) -> Fixity {
        use OperatorToken::*;
        match self {
            InfixOperator(_) | InfixAssignment(_) | InlineBlockDelimiter(..) => Fixity::Infix,
            PostfixOperator(_) => Fixity::Postfix,
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
    pub fn starts_auto_block(self) -> bool {
        use OperatorToken::*;
        matches!(self, InfixOperator(COLON) | InlineBlockDelimiter(..))
    }
    pub fn to_string(self, ast: &Ast) -> Cow<str> {
        use OperatorToken::*;
        match self {
            InfixOperator(NEWLINE_SEQUENCE)
            | InfixOperator(FOLLOWED_BY)
            | InfixOperator(IMMEDIATELY_FOLLOWED_BY) => "".into(),

            InfixOperator(identifier) | PostfixOperator(identifier) => {
                ast.identifier_string(identifier).into()
            }

            InlineBlockDelimiter(level, _) => level.to_string().into(),

            InfixAssignment(identifier) => format!("{}=", ast.identifier_string(identifier)).into(),
            Close(_, boundary) | CloseBlock(_, boundary) => boundary.close_string().into(),
        }
    }
    pub fn to_visible_string(self, ast: &Ast) -> Cow<str> {
        use OperatorToken::*;
        match self {
            InfixOperator(NEWLINE_SEQUENCE) => "<\\n>".into(),
            InfixOperator(FOLLOWED_BY) => "< >".into(),
            InfixOperator(IMMEDIATELY_FOLLOWED_BY) => "<>".into(),
            Close(_, boundary) | CloseBlock(_, boundary) => boundary.visible_close_string().into(),
            _ => self.to_string(ast),
        }
    }
    pub fn takes_right_child(self, right: impl Into<Token>) -> bool {
        use Fixity::*;
        match (self.fixity(), right.into()) {
            (Infix, Token::Operator(right)) if right.fixity() == Infix => {
                Precedence::from(self).takes_right_child(Precedence::from(right))
            }
            (left, right) => left.takes_right_child(right.fixity()),
        }
    }

    pub fn original_bytes(self, ast: &Ast) -> Cow<[u8]> {
        use OperatorToken::*;
        match self {
            InfixOperator(NEWLINE_SEQUENCE)
            | InfixOperator(FOLLOWED_BY)
            | InfixOperator(IMMEDIATELY_FOLLOWED_BY) => Cow::Borrowed(b""),

            InlineBlockDelimiter(level, repeat) => level.original_string(repeat).into(),

            InfixOperator(identifier) | PostfixOperator(identifier) => {
                ast.identifier_string(identifier).as_bytes().into()
            }

            InfixAssignment(identifier) => {
                // Because of how InfixAssignment works, we store the str for the "+" and assume the "="
                let bytes = ast.identifier_string(identifier).as_bytes();
                let mut vec = Vec::with_capacity(bytes.len() + 1);
                vec.extend_from_slice(bytes);
                vec.push(b'=');
                vec.into()
            }

            Close(_, boundary) | CloseBlock(_, boundary) => {
                boundary.close_string().as_bytes().into()
            }
        }
    }
}

impl TermToken {
    pub fn to_string(self, ast: &Ast) -> Cow<str> {
        use TermToken::*;
        match self {
            IntegerLiteral(literal) => ast.literal_string(literal).into(),
            ErrorTerm(code, ..) => format!("error({:?})", code).into(),
            RawErrorTerm(code, ..) => format!("error({:?})", code).into(),
            FieldReference(field) => ast.identifier_string(ast.fields[field].name).into(),
            RawIdentifier(identifier) => ast.identifier_string(identifier).into(),
            MissingExpression => "".into(),
        }
    }
    pub fn to_visible_string(self, ast: &Ast) -> Cow<str> {
        use TermToken::*;
        match self {
            MissingExpression => "<missing>".into(),
            _ => self.to_string(ast),
        }
    }
    pub fn original_bytes(self, ast: &Ast) -> Cow<[u8]> {
        use TermToken::*;
        match self {
            IntegerLiteral(literal) | ErrorTerm(.., literal) => {
                ast.literal_string(literal).as_bytes()
            }
            RawErrorTerm(.., raw_literal) => &ast.raw_literals[raw_literal],

            FieldReference(field) => ast.identifier_string(ast.fields[field].name).as_bytes(),

            RawIdentifier(identifier) => ast.identifier_string(identifier).as_bytes(),
            MissingExpression => unreachable!(),
        }
        .into()
    }
}

impl InlineBlockLevel {
    pub fn to_string(self) -> &'static str {
        match self {
            InlineBlockLevel::One => "===",
            InlineBlockLevel::Two => "---",
        }
    }
    pub fn original_string(self, repeat: Delta<ByteIndex>) -> Vec<u8> {
        match self {
            InlineBlockLevel::One => b"=".repeat(repeat.into()),
            InlineBlockLevel::Two => b"-".repeat(repeat.into()),
        }
    }
    pub fn identifier(self) -> IdentifierIndex {
        match self {
            InlineBlockLevel::One => INLINE_BLOCK_LEVEL_ONE,
            InlineBlockLevel::Two => INLINE_BLOCK_LEVEL_TWO,
        }
    }
}

impl ExpressionBoundary {
    /// Tells whether this expression boundary represents a block.
    pub fn is_block(self) -> bool {
        match self {
            CurlyBraces | Source | Root | AutoBlock | IndentedBlock => true,
            Parentheses | PrecedenceGroup | CompoundTerm | IndentedExpression => false,
        }
    }
    /// Tells whether this boundary type MUST be in the expression tree (because
    /// it represents actual user syntax, or opens a scope).
    pub fn is_required(self) -> bool {
        match self {
            Root | Source | CurlyBraces | Parentheses | AutoBlock | IndentedBlock
            | IndentedExpression => true,
            PrecedenceGroup | CompoundTerm => false,
        }
    }
    /// Tells whether we expect a close token for this boundary or if it's handled
    /// by the grouper automatically.
    pub fn is_closed_automatically(self) -> bool {
        match self {
            PrecedenceGroup | CompoundTerm | AutoBlock | IndentedBlock | IndentedExpression => true,
            Root | Source | CurlyBraces | Parentheses => false,
        }
    }
    pub fn placeholder_open_token(self, error: Option<ExpressionBoundaryError>) -> ExpressionToken {
        ExpressionToken::Open(error, self, Default::default())
    }
    pub fn placeholder_close_token(self) -> OperatorToken {
        OperatorToken::Close(Default::default(), self)
    }
    pub fn open_string(self) -> &'static str {
        match self {
            CurlyBraces => OPEN_CURLY.well_known_str(),
            Parentheses => OPEN_PAREN.well_known_str(),
            PrecedenceGroup | AutoBlock | IndentedBlock | IndentedExpression | CompoundTerm
            | Source | Root => "",
        }
    }
    pub fn visible_open_string(self) -> &'static str {
        match self {
            CurlyBraces => OPEN_CURLY.well_known_str(),
            Parentheses => OPEN_PAREN.well_known_str(),
            PrecedenceGroup => "precedence (",
            AutoBlock => "auto {",
            IndentedBlock => "indent {",
            IndentedExpression => "indent (",
            CompoundTerm => "term (",
            Source => "source {",
            Root => "root {",
        }
    }
    pub fn close_string(self) -> &'static str {
        match self {
            CurlyBraces => CLOSE_CURLY.well_known_str(),
            Parentheses => CLOSE_PAREN.well_known_str(),
            PrecedenceGroup | AutoBlock | IndentedBlock | IndentedExpression | CompoundTerm
            | Source | Root => "",
        }
    }
    pub fn visible_close_string(self) -> &'static str {
        match self {
            CurlyBraces => CLOSE_CURLY.well_known_str(),
            Parentheses => CLOSE_PAREN.well_known_str(),
            PrecedenceGroup => ") precedence",
            AutoBlock => "} auto",
            IndentedBlock => "} indent",
            IndentedExpression => ") indent",
            CompoundTerm => ") term",
            Source => "} source",
            Root => "} root",
        }
    }
}

impl From<ExpressionToken> for Token {
    fn from(from: ExpressionToken) -> Token {
        Token::Expression(from)
    }
}

impl From<TermToken> for ExpressionToken {
    fn from(from: TermToken) -> ExpressionToken {
        ExpressionToken::Term(from)
    }
}

impl From<TermToken> for Token {
    fn from(from: TermToken) -> Token {
        ExpressionToken::from(from).into()
    }
}

impl From<OperatorToken> for Token {
    fn from(from: OperatorToken) -> Token {
        Token::Operator(from)
    }
}

impl Fixity {
    pub fn num_operands(self) -> u8 {
        use Fixity::*;
        match self {
            Term => 0,
            Prefix | Postfix | Open | Close => 1,
            Infix => 2,
        }
    }
    pub fn has_left_operand(self) -> bool {
        use Fixity::*;
        match self {
            Infix | Postfix | Close => true,
            Term | Prefix | Open => false,
        }
    }
    pub fn has_right_operand(self) -> bool {
        use Fixity::*;
        match self {
            Infix | Prefix | Open => true,
            Term | Postfix | Close => false,
        }
    }
    pub fn takes_right_child(self, right: Fixity) -> bool {
        use Fixity::*;
        match (self, right) {
            // Terms are always OK as a right child
            (_, Term) | (_, Prefix) | (_, Open) => true,
            // Term, postfix and close don't take right children at all.
            (Term, _) | (Postfix, _) | (Close, _) => false,
            // Prefix doesn't take any operators as right child
            (Prefix, Postfix) | (Prefix, Infix) | (Prefix, Close) => false,
            // Open takes all operators as right child
            (Open, Postfix) | (Open, Infix) | (Open, Close) => true,
            // Infix takes postfix operators, but not infix or close.
            (Infix, Postfix) => true,
            (Infix, Infix) | (Infix, Close) => false,
        }
    }
}

impl fmt::Display for Fixity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Fixity::*;
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

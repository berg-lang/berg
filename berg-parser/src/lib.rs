extern crate derive_more;

pub(crate) mod bytes;
mod parser;
pub(crate) mod syntax;

pub use bytes::{line_column::LineColumnRange, ByteIndex, ByteRange, ByteSlice};
pub use parser::parse;
pub use syntax::{
    ast::{Ast, AstIndex, LiteralIndex, RawLiteralIndex},
    block::{BlockIndex, FieldError, FieldIndex},
    expression_tree::{AstExpressionTree, ExpressionPosition, ExpressionTreeWalker},
    identifiers::{self, IdentifierIndex},
    token::{
        ErrorTermError, ExpressionBoundary, ExpressionBoundaryError, ExpressionToken, Fixity,
        OperatorToken, RawErrorTermError, TermToken, Token,
    },
};

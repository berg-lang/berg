extern crate derive_more;

mod parser;
pub(crate) mod syntax;

pub use parser::parse;
pub use syntax::{
    ast::{Ast, AstIndex, LiteralIndex, RawLiteralIndex},
    block::{BlockIndex, FieldError, FieldIndex},
    bytes::{ByteIndex, ByteRange, ByteSlice},
    line_column::LineColumnRange,
    expression_tree::{AstExpressionTree, ExpressionPosition, ExpressionTreeWalker},
    identifiers::{self, IdentifierIndex},
    token::{
        ErrorTermError, ExpressionBoundary, ExpressionBoundaryError, ExpressionToken, Fixity,
        OperatorToken, RawErrorTermError, TermToken, Token,
    },
};

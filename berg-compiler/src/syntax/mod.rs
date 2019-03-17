pub use self::char_data::LineColumnRange;
pub use self::expression_tree::{AstExpressionTree, ExpressionTreeWalker, ExpressionRef};
pub use self::expression_visitor::{ExpressionVisitor, Expression, VisitResult};
pub use self::expression_formatter::{ExpressionFormatter, ExpressionTreeFormatter};
pub mod identifiers;
pub use self::identifiers::IdentifierIndex;
pub use self::token::ExpressionBoundary;

pub(crate) use self::ast::{
    Ast, AstDelta, AstIndex, AstRef, LiteralIndex, OperandPosition, RawLiteralIndex,
};
pub(crate) use self::block::{AstBlock, BlockIndex, Field, FieldError, FieldIndex};
pub(crate) use self::source::{
    ByteIndex, ByteRange, ByteSlice, SourceBuffer, SourceOpenError, SourceRef,
};
pub(crate) use self::source_reconstruction::{SourceReconstruction, SourceReconstructionReader};
pub(crate) use self::token::{ErrorTermError, ExpressionBoundaryError, ExpressionToken, Fixity, OperatorToken, RawErrorTermError, TermToken, Token};

mod ast;
mod ast_expression;
mod block;
mod char_data;
mod expression_formatter;
mod expression_tree;
mod expression_visitor;
mod precedence;
mod source;
mod source_reconstruction;
mod token;

///
/// Use this to make a series of constant fields starting at a particular index.
/// Used for keyword fields on [`RootData`], but could be used for anything with
/// a known starting index.
/// 
macro_rules! fields {
    { starting at $start:tt { $($name:ident,)* } } => {
        pub const FIELD_NAMES: [crate::syntax::IdentifierIndex; FieldDeltas::COUNT as usize] = [
            $(crate::syntax::identifiers::$name,)*
        ];
        #[allow(dead_code)]
        enum FieldDeltas {
            $($name),*,
            COUNT
        }
        #[allow(dead_code)]
        fn field_name(field: crate::syntax::FieldIndex) -> crate::syntax::IdentifierIndex {
            FIELD_NAMES[usize::from(field) - $start]
        }
        $(
            #[allow(dead_code)]
            pub const $name: crate::syntax::FieldIndex = crate::syntax::FieldIndex($start + FieldDeltas::$name as u32);
        )*
    };
    { $($name:ident,)* } => { fields! { starting at 0 { $($name,)* } } }
}


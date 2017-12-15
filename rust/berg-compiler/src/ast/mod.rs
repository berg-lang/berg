pub(crate) mod expression;
pub(crate) mod identifiers;
pub(crate) mod intern_pool;
pub(crate) mod precedence;
pub(crate) mod token;


use ast::token::Token;
use compiler::source_data::ByteRange;
use indexed_vec::IndexedVec;
use indexed_vec;
use std::u32;

index_type! {
    pub struct AstIndex(pub u32) <= u32::MAX;
    pub struct IdentifierIndex(pub u32) <= u32::MAX;
    pub struct LiteralIndex(pub u32) <= u32::MAX;
}

pub(crate) type Tokens = IndexedVec<Token,AstIndex>;
pub(crate) type TokenRanges = IndexedVec<ByteRange,AstIndex>;

// So we can signify that something is meant to be a *difference* between indices.
pub type AstDelta = indexed_vec::Delta<AstIndex>;

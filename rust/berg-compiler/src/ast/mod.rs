pub(crate) mod ast_walker;
pub(crate) mod operators;
pub(crate) mod intern_pool;
pub(crate) mod token;

use indexed_vec;
use std::u32;

index_type! {
    pub struct IdentifierIndex(pub u32) <= u32::MAX;
    pub struct LiteralIndex(pub u32) <= u32::MAX;
    pub struct AstIndex(pub u32) <= u32::MAX;
}

// So we can signify that something is meant to be a *difference* between indices.
pub type AstDelta = indexed_vec::Delta<AstIndex>;

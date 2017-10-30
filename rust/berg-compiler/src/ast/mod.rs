pub(crate) mod ast_walker;
pub(crate) mod operators;
pub(crate) mod intern_pool;
pub(crate) mod token;

use std::u32;

index_type! {
    pub struct IdentifierIndex(pub u32) <= u32::MAX;
    pub struct LiteralIndex(pub u32) <= u32::MAX;
    pub struct AstIndex(pub u32) <= u32::MAX;
}

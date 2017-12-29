pub(crate) mod expression;
pub(crate) mod identifiers;
pub(crate) mod intern_pool;
pub(crate) mod precedence;
pub(crate) mod token;

use ast::identifiers::*;
use ast::token::Token;
use interpreter::value::Value;
use source::parse_result::ByteRange;
use std::u32;
use util::indexed_vec::IndexedVec;

index_type! {
    pub struct AstIndex(pub u32) <= u32::MAX;
    pub struct IdentifierIndex(pub u32) <= u32::MAX;
    pub struct LiteralIndex(pub u32) <= u32::MAX;
    pub struct VariableIndex(pub u32) <= u32::MAX;
}

pub(crate) type Tokens = IndexedVec<Token,AstIndex>;
pub(crate) type TokenRanges = IndexedVec<ByteRange,AstIndex>;

// So we can signify that something is meant to be a *difference* between indices.
pub type AstDelta = Delta<AstIndex>;

#[derive(Debug)]
pub struct Variable {
    pub(crate) name: IdentifierIndex,
}

pub(crate) fn root_variables() -> IndexedVec<(IdentifierIndex,Value),VariableIndex> {
    vec![
        (TRUE, true.into()),
        (FALSE, false.into()),
    ].into()
}

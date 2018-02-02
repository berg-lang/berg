use eval::BergEval;
use syntax::{AstRef, BlockIndex, FieldIndex};
use eval::BlockRef;
use std::fmt;
use value::BergResult;

#[derive(Clone)]
pub enum ScopeRef<'a> {
    BlockRef(BlockRef<'a>),
    AstRef(AstRef<'a>),
}

impl<'a> ScopeRef<'a> {
    pub fn create_child_block(&mut self, index: BlockIndex) -> Self {
        match *self {
            ScopeRef::BlockRef(ref block) => ScopeRef::BlockRef(block.create_child_block(index)),
            ScopeRef::AstRef(_) => ScopeRef::BlockRef(BlockRef::new(index, self.clone())),
        }
    }
    pub fn field(&self, index: FieldIndex, ast: &AstRef) -> BergResult<'a, BergResult<'a, BergEval<'a>>> {
        match *self {
            ScopeRef::BlockRef(ref block) => block.field(index, ast),
            ScopeRef::AstRef(ref ast) => Ok(ast.source().root().field(index)),
        }
    }
    pub fn declare_field(&mut self, index: FieldIndex, ast: &AstRef) -> BergResult<'a, ()> {
        match *self {
            ScopeRef::BlockRef(ref mut block) => block.declare_field(index, ast),
            ScopeRef::AstRef(_) => ast.source().root().declare_field(index),
        }
    }
    pub fn set_field(
        &mut self,
        index: FieldIndex,
        value: BergResult<'a, BergEval<'a>>,
        ast: &AstRef,
    ) -> BergResult<'a, ()> {
        match *self {
            ScopeRef::BlockRef(ref mut block) => block.set_field(index, value, ast),
            ScopeRef::AstRef(ref ast) => ast.source().root().set_field(index, value),
        }
    }
    pub fn ast(&self) -> AstRef<'a> {
        match *self {
            ScopeRef::BlockRef(ref block) => block.ast(),
            ScopeRef::AstRef(ref ast) => ast.clone(),
        }
    }
}

impl<'a> fmt::Debug for ScopeRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ScopeRef::BlockRef(ref block) => block.fmt(f),
            ScopeRef::AstRef(ref ast) => f.debug_struct("AstRef")
                .field("fields", &ast.root().field_names())
                .finish(),
        }
    }
}

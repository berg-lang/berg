use crate::eval::BlockRef;
use crate::syntax::{Ast, AstIndex, AstRef, BlockIndex, FieldIndex, IdentifierIndex};
use crate::value::{BergResult, BergVal, BergValue};
use std::fmt;

#[derive(Clone)]
pub enum ScopeRef<'a> {
    BlockRef(BlockRef<'a>),
    AstRef(AstRef<'a>),
}

impl<'a> ScopeRef<'a> {
    pub fn create_child_block(&self, expression: AstIndex, index: BlockIndex) -> BlockRef<'a> {
        match self {
            ScopeRef::BlockRef(ref block) => block.create_child_block(expression, index),
            ScopeRef::AstRef(_) => BlockRef::new(expression, index, self.clone(), Ok(BergVal::empty_tuple())),
        }
    }
    pub fn local_field(&self, index: FieldIndex, ast: &Ast) -> BergResult<'a, BergResult<'a>> {
        match self {
            ScopeRef::BlockRef(ref block) => block.local_field(index, ast),
            ScopeRef::AstRef(ref ast) => Ok(ast.source.root().local_field(index)),
        }
    }
    pub fn field(self, name: IdentifierIndex) -> BergResult<'a, BergResult<'a>> {
        match self {
            ScopeRef::BlockRef(block) => block.field(name),
            ScopeRef::AstRef(ast) => ast.source.root().field(name),
        }
    }
    pub fn declare_field(&self, index: FieldIndex, ast: &Ast) -> BergResult<'a, ()> {
        match self {
            ScopeRef::BlockRef(block) => block.declare_field(index, ast),
            ScopeRef::AstRef(_) => ast.source.root().declare_field(index),
        }
    }
    pub fn set_local_field(
        &self,
        index: FieldIndex,
        value: BergResult<'a>,
        ast: &Ast,
    ) -> BergResult<'a, ()> {
        match self {
            ScopeRef::BlockRef(block) => block.set_local_field(index, value, ast),
            ScopeRef::AstRef(ast) => ast.source.root().set_local_field(index, value),
        }
    }
    pub fn ast(&self) -> AstRef<'a> {
        match self {
            ScopeRef::BlockRef(block) => block.ast(),
            ScopeRef::AstRef(ast) => ast.clone(),
        }
    }
}

impl<'a> fmt::Debug for ScopeRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ScopeRef::BlockRef(ref block) => block.fmt(f),
            ScopeRef::AstRef(ref ast) => f
                .debug_struct("AstRef")
                .field("fields", &ast.root().field_names())
                .finish(),
        }
    }
}

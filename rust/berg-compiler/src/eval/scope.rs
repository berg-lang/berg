use error::{BergResult, EvalResult};
use eval::{BlockRef, Expression};
use std::fmt;
use syntax::{AstRef, BlockIndex, FieldIndex, IdentifierIndex};

#[derive(Clone)]
pub enum ScopeRef<'a> {
    BlockRef(BlockRef<'a>),
    AstRef(AstRef<'a>),
}

impl<'a> ScopeRef<'a> {
    pub fn create_child_block(&mut self, expression: Expression, index: BlockIndex) -> BlockRef<'a> {
        match *self {
            ScopeRef::BlockRef(ref block) => block.create_child_block(expression, index),
            ScopeRef::AstRef(_) => BlockRef::new(expression, index, self.clone()),
        }
    }
    pub fn field(&self, index: FieldIndex, ast: &AstRef) -> EvalResult<'a> {
        match *self {
            ScopeRef::BlockRef(ref block) => block.field(index, ast),
            ScopeRef::AstRef(ref ast) => ast.source().root().field(index),
        }
    }
    pub fn public_field_by_name(&self, name: IdentifierIndex) -> EvalResult<'a> {
        match *self {
            ScopeRef::BlockRef(ref block) => block.public_field_by_name(name),
            ScopeRef::AstRef(ref ast) => ast.source().root().public_field_by_name(name),
        }
    }
    pub fn declare_field(&mut self, index: FieldIndex, ast: &AstRef) -> EvalResult<'a, ()> {
        match *self {
            ScopeRef::BlockRef(ref mut block) => block.declare_field(index, ast),
            ScopeRef::AstRef(_) => ast.source().root().declare_field(index),
        }
    }
    pub fn set_field(
        &mut self,
        index: FieldIndex,
        value: BergResult<'a>,
        ast: &AstRef,
    ) -> EvalResult<'a, ()> {
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

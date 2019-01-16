use crate::error::BergError;
use crate::eval::BlockRef;
use std::u32;
use crate::syntax::{AstRef, ExpressionBoundary, IdentifierIndex};

index_type! {
    pub struct BlockIndex(pub u32) with Display,Debug <= u32::MAX;
    pub struct FieldIndex(pub u32) with Display,Debug <= u32::MAX;
}

#[derive(Debug)]
pub struct AstBlock {
    pub boundary: ExpressionBoundary,
    pub parent: Delta<BlockIndex>,
    pub scope_start: FieldIndex,
    pub scope_count: Delta<FieldIndex>,
}

#[derive(Clone, Debug)]
pub struct Field {
    pub name: IdentifierIndex,
    pub is_public: bool,
}

#[derive(Copy, Clone, Debug)]
pub enum FieldError {
    PrivateField(IdentifierIndex),
    NoSuchPublicField(IdentifierIndex),
}

impl AstBlock {
    pub fn public_field_index(
        &self,
        index: BlockIndex,
        name: IdentifierIndex,
        ast: &AstRef,
    ) -> Result<FieldIndex, FieldError> {
        let mut child_index = index + 1;
        let mut field_index = self.scope_start;
        let scope_end = self.scope_start + self.scope_count;
        while field_index < scope_end {
            // Bypass any indices that are owned by child blocks.
            if let Some(child) = ast.blocks().get(child_index) {
                if field_index >= child.scope_start {
                    field_index = child.scope_start + child.scope_count;
                    child_index += 1;
                    continue;
                }
            }

            let field = &ast.fields()[field_index];
            if field.name == name {
                if field.is_public {
                    return Ok(field_index);
                } else {
                    return Err(FieldError::PrivateField(name));
                };
            }
            field_index += 1;
        }

        Err(FieldError::NoSuchPublicField(name))
    }
}

impl FieldError {
    pub fn in_block<'a>(self, block: &BlockRef<'a>) -> BergError<'a> {
        match self {
            FieldError::NoSuchPublicField(index) => {
                BergError::NoSuchPublicField(block.clone(), index)
            }
            FieldError::PrivateField(index) => BergError::PrivateField(block.clone(), index),
        }
    }
}

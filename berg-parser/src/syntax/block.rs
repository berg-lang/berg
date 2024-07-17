use berg_util::index_type;

use super::{
    ast::{Ast, AstDelta},
    identifiers::IdentifierIndex,
    token::ExpressionBoundary,
};

index_type! {
    pub struct BlockIndex(pub u32) with Display,Debug <= u32::MAX;
    pub struct FieldIndex(pub u32) with Display,Debug <= u32::MAX;
}

#[derive(Debug)]
pub struct AstBlock {
    pub boundary: ExpressionBoundary,
    pub parent: Delta<BlockIndex>,
    pub delta: AstDelta,
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
    PrivateField,
    NoSuchPublicField,
}

///
/// Use this to make a series of constant fields starting at a particular index.
/// Used for keyword fields on [`RootData`], but could be used for anything with
/// a known starting index.
///
#[macro_export]
macro_rules! fields {
    { starting at $start:tt { $($name:ident,)* } } => {
        pub const FIELD_NAMES: [$crate::IdentifierIndex; FieldDeltas::COUNT as usize] = [
            $($crate::identifiers::$name,)*
        ];
        #[allow(dead_code)]
        enum FieldDeltas {
            $($name),*,
            COUNT
        }
        #[allow(dead_code)]
        fn field_name(field: $crate::FieldIndex) -> $crate::IdentifierIndex {
            FIELD_NAMES[usize::from(field) - $start]
        }
        $(
            #[allow(dead_code)]
            pub const $name: $crate::FieldIndex = $crate::FieldIndex($start + FieldDeltas::$name as u32);
        )*
    };
    { $($name:ident,)* } => { fields! { starting at 0 { $($name,)* } } }
}

impl AstBlock {
    pub fn public_field_index(
        &self,
        index: BlockIndex,
        name: IdentifierIndex,
        ast: &Ast,
    ) -> Result<FieldIndex, FieldError> {
        let mut child_index = index + 1;
        let mut field_index = self.scope_start;
        let scope_end = self.scope_start + self.scope_count;
        while field_index < scope_end {
            // Bypass any indices that are owned by child blocks.
            if let Some(child) = ast.blocks.get(child_index) {
                if field_index >= child.scope_start {
                    field_index = child.scope_start + child.scope_count;
                    child_index += 1;
                    continue;
                }
            }

            let field = &ast.fields[field_index];
            if field.name == name {
                if field.is_public {
                    return Ok(field_index);
                } else {
                    return Err(FieldError::PrivateField);
                };
            }
            field_index += 1;
        }

        Err(FieldError::NoSuchPublicField)
    }
}

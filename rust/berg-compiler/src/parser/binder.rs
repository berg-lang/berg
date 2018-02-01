use parser::source::SourceRef;
use syntax::{AstBlock, AstData, AstDelta, AstIndex, BlockIndex, ExpressionBoundary,
             ExpressionBoundaryError, Field, FieldIndex, IdentifierIndex};
use parser::ByteRange;
use syntax::Token;
use syntax::Token::*;
use syntax::identifiers::*;
use util::indexed_vec::Delta;

// Handles nesting and precedence: balances (), {}, and compound terms, and
// inserts "precedence groups," and removes compound terms and precedence
// groups where it can.
#[derive(Debug)]
pub struct Binder<'a> {
    pub ast: AstData<'a>,
    open_scopes: Vec<OpenScope>,
    scope: Vec<FieldIndex>,
}

#[derive(Debug)]
pub struct OpenScope {
    open_index: AstIndex,
    index: BlockIndex,
    scope_start: usize,
}

impl<'a> Binder<'a> {
    pub fn new(source: SourceRef<'a>) -> Self {
        // Grab the root field names
        let scope = (0..source.root().field_names().len())
            .map(|i| i.into())
            .collect();
        let mut result = Binder {
            ast: AstData::new(source),
            open_scopes: Default::default(),
            scope,
        };
        result.push_open_scope(ExpressionBoundary::Root, None);

        // Ensure the scope and the ast's fields (taken from root names) match up
        assert_eq!(
            result.scope.len(),
            result.ast.source.root().field_names().len()
        );
        for (root_field_name, scope_field) in result
            .ast
            .source
            .root()
            .field_names()
            .zip(result.scope.iter())
        {
            assert_eq!(*root_field_name, result.ast.fields[*scope_field].name);
        }
        result
    }

    pub fn on_source_end(self) -> AstData<'a> {
        self.ast
    }

    pub fn push_token(&mut self, token: Token, range: ByteRange) -> AstIndex {
        match token {
            RawIdentifier(name) => self.push_field_reference(name, range),
            Open {
                boundary,
                error,
                delta,
            } if boundary.is_scope() =>
            {
                let open_block_index = self.open_block_index();
                let index = self.push_open_scope(boundary, Some(open_block_index));

                // Push the token.
                let token = Token::OpenBlock {
                    index,
                    delta,
                    error,
                };
                self.ast.push_token(token, range)
            }
            Close {
                boundary,
                error,
                delta,
            } if boundary.is_scope() =>
            {
                let index = self.push_close_scope();

                // Push the token.
                let token = Token::CloseBlock {
                    index,
                    delta,
                    error,
                };
                self.ast.push_token(token, range)
            }
            FieldReference(_) | OpenBlock { .. } | CloseBlock { .. } => unreachable!(),
            _ => self.ast.push_token(token, range),
        }
    }

    pub fn insert_token(&mut self, index: AstIndex, token: Token, range: ByteRange) {
        match token {
            Open {
                boundary,
                error,
                delta,
            } if boundary.is_scope() =>
            {
                self.insert_open_scope(index, boundary, error, delta, range)
            }
            Open { .. } => self.ast.insert_token(index, token, range),
            _ => unreachable!(),
        }
    }

    fn push_field_reference(&mut self, name: IdentifierIndex, range: ByteRange) -> AstIndex {
        let is_local = match self.ast.tokens.last() {
            Some(&PrefixOperator(COLON)) => true,
            _ => false,
        };
        let field = self.find_field(name, is_local)
            .unwrap_or_else(|| self.create_field(name));
        self.ast.push_token(FieldReference(field), range)
    }

    fn find_field(&mut self, name: IdentifierIndex, is_local: bool) -> Option<FieldIndex> {
        let mut scope = if is_local {
            self.scope[self.open_scope().scope_start..].iter().rev()
        } else {
            self.scope[0..].iter().rev()
        };
        scope.find(|v| self.ast.fields[**v].name == name).cloned()
    }

    fn create_field(&mut self, name: IdentifierIndex) -> FieldIndex {
        // We couldn't find it (or we exposed a new field). Declare it in local scope.
        let index = self.ast.fields.push(Field { name });
        self.scope.push(index);
        index
    }

    fn insert_open_scope(
        &mut self,
        open_index: AstIndex,
        boundary: ExpressionBoundary,
        error: ExpressionBoundaryError,
        delta: AstDelta,
        range: ByteRange,
    ) {
        let (index, ast_block) = {
            // As long as scope openers are the highest precedence, scope openers will only ever be
            // inserted right after the previously opened scope. Test that assumption here.
            let open_scope = self.open_scope();
            assert_eq!(open_scope.open_index, open_index - 1);

            // Insert the block right after the open block.
            let index = open_scope.index + 1;
            let ast_block = AstBlock {
                parent: open_scope.index - index,
                scope_start: self.ast.fields.next_index(),
                boundary,
            };
            (index, ast_block)
        };

        self.ast.blocks.insert(index, ast_block);

        // Insert the scope. This block will take all of the previous block's children.
        let scope_start = self.scope.len();
        self.open_scopes.push(OpenScope {
            open_index,
            index,
            scope_start,
        });

        // Fix all parent indices after the block. They are guaranteed to be our children since this
        // will only happen after closing any children.
        for (i, b) in self.ast.blocks[(index + 1)..].iter_mut().enumerate() {
            let i = (index + 1) + i;
            assert!(i - b.parent >= index);
            b.parent += 1;
        }

        // Insert the token. No token adjustment necessary since everything does deltas.
        let token = Token::OpenBlock {
            index,
            delta,
            error,
        };
        self.ast.insert_token(open_index, token, range);
    }

    fn push_open_scope(
        &mut self,
        boundary: ExpressionBoundary,
        parent: Option<BlockIndex>,
    ) -> BlockIndex {
        // Create the block.
        let parent = parent
            .and_then(|parent| Some(self.ast.blocks.next_index() - parent))
            .unwrap_or(Delta(BlockIndex(0)));
        let index = self.ast.blocks.push(AstBlock {
            boundary,
            parent,
            scope_start: self.ast.fields.next_index(),
        });

        // Push the scope.
        let open_index = self.ast.next_index();
        let scope_start = self.scope.len();
        self.open_scopes.push(OpenScope {
            open_index,
            index,
            scope_start,
        });

        index
    }

    fn push_close_scope(&mut self) -> BlockIndex {
        // Pop the scope.
        let open_scope = self.open_scopes.pop().unwrap();
        self.scope.truncate(open_scope.scope_start);
        open_scope.index
    }

    fn open_block_index(&self) -> BlockIndex {
        self.open_scope().index
    }

    fn open_scope(&self) -> &OpenScope {
        self.open_scopes.last().unwrap()
    }
}

use crate::syntax::identifiers::*;
use crate::syntax::{
    Ast, AstBlock, AstDelta, AstIndex, BlockIndex, ByteRange, ExpressionBoundary,
    ExpressionBoundaryError, Field, FieldIndex, IdentifierIndex, Token, ExpressionToken,
    OperatorToken, TermToken
};
use crate::util::indexed_vec::Delta;

// Handles nesting and precedence: balances (), {}, and compound terms, and
// inserts "precedence groups," and removes compound terms and precedence
// groups where it can.
#[derive(Debug)]
pub struct Binder<'a> {
    pub ast: Ast<'a>,
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
    pub fn new(ast: Ast<'a>) -> Self {
        // Grab the root field names
        let scope = (0..ast.source.root().field_names().len())
            .map(|i| i.into())
            .collect();
        let mut result = Binder {
            ast,
            open_scopes: Default::default(),
            scope,
        };
        result.push_open_scope(ExpressionBoundary::Root, None);

        // Ensure the scope and the ast's fields (taken from root names) match up
        // since the root object is going to assume that. The code above should
        // ensure this; we're just making sure.
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

    pub fn on_source_end(self) -> Ast<'a> {
        self.ast
    }

    pub fn push_expression_token(&mut self, token: ExpressionToken, range: ByteRange) -> AstIndex {
        use ExpressionToken::*;
        use TermToken::*;
        match token {
            Term(token) => match token {
                // Unless it's preceded by a ., raw identifier is always a local field access or declaration, so bind it and translate it.
                RawIdentifier(name)
                    if match self.ast.tokens.last() {
                        Some(&Token::Operator(OperatorToken::InfixOperator(DOT))) => false,
                        _ => true,
                    } =>
                {
                    self.push_field_reference(name, range)
                }
                IntegerLiteral(_) | RawIdentifier(_) | ErrorTerm(..) | RawErrorTerm(..) | MissingExpression => self.ast.push_token(token, range),
                // The binder generates these tokens, so should not receive them as input.
                FieldReference(_) => unreachable!(),
            }
            Open(_, boundary, _) if boundary.is_block() => {
                let open_block_index = self.open_block_index();
                self.push_open_scope(boundary, Some(open_block_index));
                self.ast.push_token(token, range)
            }
            PrefixOperator(_) | Open(..) => self.ast.push_token(token, range),
        }
    }

    pub fn push_operator_token(&mut self, token: OperatorToken, range: ByteRange) -> AstIndex {
        use OperatorToken::*;
        match token {
            Close(delta, boundary) if boundary.is_block() => {
                let index = self.push_close_scope(delta);
                self.ast.push_token(CloseBlock(index, boundary), range)
            }
            // We are the one who generates CloseBlock; no one before us should be doing so.
            CloseBlock(..) => unreachable!(),
            InfixOperator(COLON) => self.push_declaration_with_default(token, range),
            _ => self.ast.push_token(token, range),
        }
    }

    pub fn insert_open_token(&mut self, index: AstIndex, error: Option<ExpressionBoundaryError>, boundary: ExpressionBoundary, delta: AstDelta, range: ByteRange) {
        if boundary.is_block() {
            self.insert_open_scope(index, boundary, error, delta, range)
        } else {
            self.ast.insert_token(index, ExpressionToken::Open(error, boundary, delta), range)
        }
    }

    fn push_field_reference(&mut self, name: IdentifierIndex, range: ByteRange) -> AstIndex {
        use Token::*;
        use ExpressionToken::*;
        use TermToken::*;
        let is_declaration = match self.ast.tokens.last() {
            Some(&Expression(PrefixOperator(COLON))) => true,
            _ => false,
        };
        let field = self
            .find_field(name, is_declaration)
            .unwrap_or_else(|| self.create_field(name, is_declaration));
        if is_declaration {
            self.ast.fields[field].is_public = true;
        }
        self.ast.push_token(FieldReference(field), range)
    }

    fn push_declaration_with_default(&mut self, token: OperatorToken, range: ByteRange) -> AstIndex {
        use Token::*;
        use ExpressionToken::*;
        use TermToken::*;
        let prev_token_index = self.ast.tokens.last_index();
        let prev_token = self.ast.tokens[prev_token_index];
        // Flip the field public now that we know it's a declaration.
        if let Expression(Term(FieldReference(field))) = prev_token {
            // If the field is from a parent block, we have misidentified this, because
            // a: b always refers to a local variable. Fix that up.
            // NOTE: This misidentification has no repercussions in the current code,
            // but that doesn't mean it won't become a problem in the future (for
            // example, if we start making a table of which fields are referenced by
            // other blocks). Watch out for that! We'll either need to delay identification
            // until we know the next token, or else have to go fix it up.
            if field < self.open_scope().scope_start {
                let name = self.ast.fields[field].name;
                let new_field = self.create_field(name, true);
                self.ast.tokens[prev_token_index] = FieldReference(new_field).into();
            } else {
                self.ast.fields[field].is_public = true;
            }
        }
        self.ast.push_token(token, range)
    }

    fn find_field(&mut self, name: IdentifierIndex, is_declaration: bool) -> Option<FieldIndex> {
        let mut scope = if is_declaration {
            self.scope[self.open_scope().scope_start..].iter().rev()
        } else {
            self.scope[0..].iter().rev()
        };
        scope.find(|v| self.ast.fields[**v].name == name).cloned()
    }

    fn create_field(&mut self, name: IdentifierIndex, is_public: bool) -> FieldIndex {
        // We couldn't find it (or we exposed a new field). Declare it in local scope.
        let index = self.ast.fields.push(Field { name, is_public });
        self.scope.push(index);
        index
    }

    fn insert_open_scope(
        &mut self,
        open_index: AstIndex,
        boundary: ExpressionBoundary,
        error: Option<ExpressionBoundaryError>,
        delta: AstDelta,
        range: ByteRange,
    ) {
        let (index, ast_block) = {
            // As long as scope openers are the highest precedence, scope openers will only ever be
            // inserted right after the previously opened scope. Test that assumption here.
            let open_scope = self.open_scope();
            assert_eq!(open_scope.open_index, open_index - 1);

            let open_block = &self.ast.blocks[open_scope.index];

            // This block includes all fields up to this point from the parent block, so its start is the same as the parent block's.
            let index = open_scope.index + 1;
            let ast_block = AstBlock {
                parent: index - open_scope.index,
                scope_start: open_block.scope_start,
                scope_count: Delta(FieldIndex(0)),
                delta,
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
        let token = ExpressionToken::Open(error, boundary, delta);
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
            scope_count: Delta(FieldIndex(0)),
            delta: Default::default(),
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

    fn push_close_scope(&mut self, delta: AstDelta) -> BlockIndex {
        // Pop the scope.
        let open_scope = self.open_scopes.pop().unwrap();
        // Set the range of fields in scope for this block and its children.
        {
            let block = &mut self.ast.blocks[open_scope.index];
            block.scope_count = FieldIndex(self.ast.fields.len() as u32) - block.scope_start;
            block.delta = delta;
        }
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

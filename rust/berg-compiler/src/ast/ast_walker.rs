use ast::{AstIndex,IdentifierIndex};
use ast::operators::Operators;
use ast::operators::Operators::*;
use ast::token::Token::*;
use ast::token::PrefixToken::*;
use ast::token::PostfixToken::*;
use public::*;

pub trait AstVisitorMut<T> {
    fn visit_term(&mut self, token: TermToken, index: AstIndex, source_data: &SourceData) -> T;
    fn visit_postfix(&mut self, postfix: IdentifierIndex, operand: T, index: AstIndex, source_data: &SourceData) -> T;
    fn visit_prefix(&mut self, prefix: IdentifierIndex, operand: T, index: AstIndex, source_data: &SourceData) -> T;
    fn visit_infix(&mut self, token: InfixToken, left: T, right: T, index: AstIndex, source_data: &SourceData) -> T;
    fn open_without_close(&mut self, _open: Operators, _open_index: AstIndex, _missing_close_index: AstIndex, _source_data: &SourceData) {}
    fn close_without_open(&mut self, _close: Operators, _close_index: AstIndex, _missing_open_index: AstIndex, _source_data: &SourceData) {}
}

#[derive(Copy,Clone,Default)]
pub struct AstWalkerMut {
    index: AstIndex
}

impl AstWalkerMut {
    pub fn walk<T, V: AstVisitorMut<T>>(visitor: &mut V, source_data: &SourceData) -> T {
        let mut walker = AstWalkerMut { index: AstIndex(0) };
        let mut value = walker.walk_expression(visitor, source_data);
        while let Some(Postfix(Close(close))) = walker.token(source_data) {
            visitor.close_without_open(Operators::from(close), walker.index, AstIndex(0), source_data);
            walker.index += 1;
            value = walker.walk_postfixes(visitor, value, source_data);
            value = walker.walk_infix_while(visitor, value, source_data, |_| true)
        }
        assert!(walker.token(source_data).is_none());
        value
    }

    fn walk_expression<T, V: AstVisitorMut<T>>(&mut self, visitor: &mut V, source_data: &SourceData) -> T {
        let left = self.walk_infix_operand(visitor, source_data);
        self.walk_infix_while(visitor, left, source_data, |_| true)
    }

    fn walk_infix_while<T, V: AstVisitorMut<T>, F: Fn(InfixToken)->bool>(
        &mut self,
        visitor: &mut V,
        mut left: T,
        source_data: &SourceData,
        walk_if: F
    ) -> T {
        while let Some(Infix(infix)) = self.token(source_data) {
            if !walk_if(infix) {
                break;
            }
            let infix_index = self.index;
            self.index += 1;

            // Get the right operand.
            let mut right = self.walk_infix_operand(visitor, source_data);

            // Handle precedence: if we see + or -, grab all the * and / first.
            match infix.operator() {
                Operators::Plus|Operators::Dash =>
                    right = self.walk_infix_while(visitor, right, source_data, Self::multiply_or_divide),
                _ => {}
            }

            // Apply the operator now that we've grabbed anything we needed from the right!
            left = visitor.visit_infix(infix, left, right, infix_index, source_data);
        }
        left
    }

    fn multiply_or_divide(infix: InfixToken) -> bool {
        match infix.operator() {
            Star|Slash => true,
            _ => false,
        }
    }

    fn walk_infix_operand<T, V: AstVisitorMut<T>>(&mut self, visitor: &mut V, source_data: &SourceData) -> T {
        // Skip any prefixes (we'll apply them after we calculate the term)
        let start = self.index;
        while let Some(Prefix(prefix)) = self.token(source_data) {
            if prefix.operator() == OpenParen {
                break;
            }
        }

        // Handle the term
        let mut term_index = self.index;
        let token = self.token(source_data);
        self.index += 1;
        let mut value = match token {
            // Handle terms (i.e. literals and properties)
            Some(Term(term)) => visitor.visit_term(term, term_index, source_data),

            // Handle parentheses
            Some(Prefix(Open(prefix))) => {
                // Walk the expression inside
                let value = self.walk_expression(visitor, source_data);

                // Check for close parentheses
                match self.token(source_data) {
                    Some(Postfix(Close(postfix))) if Operators::from(postfix) == CloseParen => self.index += 1,
                    _ => visitor.open_without_close(Operators::from(prefix), term_index, self.index, source_data),
                }

                value
            },

            _ => { println!("Huh {:?} at {:?}", token, self.index-1); unreachable!() },
        };

        // Go backwards applying all the prefixes
        while term_index > start {
            term_index -= 1;
            value = match self.token(source_data) {
                Some(Prefix(PrefixOperator(prefix))) => visitor.visit_prefix(prefix, value, term_index, source_data),
                _ => unreachable!(),
            };
        }

        // Now a apply any postfixes
        self.walk_postfixes(visitor, value, source_data)
    }

    fn walk_postfixes<T, V: AstVisitorMut<T>>(&mut self, visitor: &mut V, mut value: T, source_data: &SourceData) -> T {
        while let Some(Postfix(PostfixOperator(postfix))) = self.token(source_data) {
            value = visitor.visit_postfix(postfix, value, self.index, source_data);
            self.index += 1;
        }

        value
    }

    fn token(&self, source_data: &SourceData) -> Option<Token> {
        if self.index < source_data.num_tokens() {
            Some(*source_data.token(self.index))
        } else {
            None
        }
    }
}

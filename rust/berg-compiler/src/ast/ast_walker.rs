use std::fmt::Debug;
use ast::{AstIndex,IdentifierIndex};
use ast::ast_walker::Advance::*;
use ast::operators::Operators;
use ast::operators::Operators::*;
use ast::token::Token::*;
use ast::token::Fixity::*;
use public::*;

pub trait AstVisitorMut<T> {
    fn visit_term(&mut self, token: TermToken, index: AstIndex, source_data: &SourceData) -> T;
    fn visit_postfix(&mut self, postfix: IdentifierIndex, operand: T, index: AstIndex, source_data: &SourceData) -> T;
    fn visit_prefix(&mut self, prefix: IdentifierIndex, operand: T, index: AstIndex, source_data: &SourceData) -> T;
    fn visit_infix(&mut self, token: InfixToken, left: T, right: T, index: AstIndex, source_data: &SourceData) -> T;
    fn open_without_close(&mut self, _open: Operators, _open_index: AstIndex, _missing_close_index: AstIndex, _source_data: &SourceData) {}
    fn close_without_open(&mut self, _close: Operators, _close_index: AstIndex, _missing_open_index: AstIndex, _source_data: &SourceData) {}
}

#[derive(Debug)]
enum Advance<T> {
    NextToken(T, AstIndex),
    NoMatch,
    Eof,
}

#[derive(Debug,Copy,Clone,Default)]
pub struct AstWalkerMut {
    index: AstIndex
}

impl AstWalkerMut {
    pub fn walk<T: Debug, V: AstVisitorMut<T>>(visitor: &mut V, source_data: &SourceData) -> T {
        let mut walker = AstWalkerMut { index: AstIndex(0) };
        let mut value = walker.walk_expression(visitor, source_data);

        // If there are extra close operators, report them.
        while let NextToken(close, close_index) = walker.advance_if(source_data,
            |token| match token { Close(close) => Some(close), _ => None }) {
            let close = Operators::from(close);
            visitor.close_without_open(close, close_index, AstIndex(0), source_data);
            // Walk any remaining postfixes.
            value = walker.walk_postfixes(visitor, value, source_data);
            // Read any remaining infixes as well.
            value = walker.walk_infix_while(visitor, value, source_data, |_| true)
        }
        assert!(walker.is_at_eof(source_data));
        value
    }

    fn walk_expression<T: Debug, V: AstVisitorMut<T>>(&mut self, visitor: &mut V, source_data: &SourceData) -> T {
        let left = self.walk_infix_operand(visitor, source_data);
        self.walk_infix(visitor, left, source_data)
    }

    fn walk_infix<T: Debug, V: AstVisitorMut<T>>(&mut self, visitor: &mut V, left: T, source_data: &SourceData) -> T {
        self.walk_infix_while(visitor, left, source_data, |_| true)
    }

    fn walk_infix_while<T: Debug, V: AstVisitorMut<T>, F: Fn(InfixToken)->bool>(
        &mut self,
        visitor: &mut V,
        mut left: T,
        source_data: &SourceData,
        walk_if: F
    ) -> T {
        while let NextToken(infix, infix_index) = self.advance_if(source_data,
            |token| match InfixToken::try_from(token) { Some(infix) if walk_if(infix) => Some(infix), _ => None })
        {
            // Get the right operand.
            let mut right = self.walk_infix_operand(visitor, source_data);

            // Handle precedence: if we see + or -, grab all the * and / first.
            match infix.operator() {
                Operators::Plus|Operators::Dash => {
                    right = self.walk_infix_while(visitor, right, source_data, Self::multiply_or_divide);
                },
                _ => {
}
            }

            // Apply the operator now that we've grabbed anything we needed from the right!
            println!("Visit infix {:?} {:?}, {:?}", infix.operator(), left, right);
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

    fn walk_infix_operand<T: Debug, V: AstVisitorMut<T>>(&mut self, visitor: &mut V, source_data: &SourceData) -> T {
        // Skip any prefixes (we'll apply them after we calculate the term)
        let first_prefix = self.index;
        while let NextToken(..) = self.advance_if(source_data, |token|
            match token.fixity() { Prefix => Some(()), _ => None }
        ) {
            // Do-nothing loop, we're just skipping them to be dealt with later
        }

        // Handle the term
        let (term_token, term_index) = self.advance(source_data);
        let term = term_token.to_term().unwrap();
        println!("Visit term {:?}", term_index);
        let mut value = visitor.visit_term(term, term_index, source_data);

        // Handle prefixes (in reverse order)
        let mut prefix_index = term_index;
        while prefix_index > first_prefix {
            prefix_index -= 1;
            let prefix_token = (*source_data.token(prefix_index)).to_prefix().unwrap();
            match prefix_token {
                PrefixToken::PrefixOperator(prefix) => {
                    println!("Visit prefix {:?} {:?}", Operators::from(prefix), value);
                    value = visitor.visit_prefix(prefix, value, term_index, source_data);
                },
                // Handle parentheses
                PrefixToken::Open(prefix) => {
                    // Walk the remainder of the expression in the parens (we already got the term)
                    value = self.walk_infix(visitor, value, source_data);

                    // Check for close and skip if it's the right one
                    let close = Operators::from(prefix).corresponding_close().identifier();
                    self.advance_if(source_data,
                        |token| match token { Close(postfix) if postfix == close => Some(()), _ => None });
                },
            };
        };

        // Now a apply any postfixes. Stop when we see a close operator.
        self.walk_postfixes(visitor, value, source_data)
    }

    fn walk_postfixes<T: Debug, V: AstVisitorMut<T>>(&mut self, visitor: &mut V, mut value: T, source_data: &SourceData) -> T {
        while let NextToken(postfix, postfix_index) = self.advance_if(source_data,
            |token| match token { PostfixOperator(postfix) => Some(postfix), _ => None })
        {
            println!("Visit postfix {:?} {:?}", Operators::from(postfix), value);
            value = visitor.visit_postfix(postfix, value, postfix_index, source_data)
        }
        value
    }

    fn advance(&mut self, source_data: &SourceData) -> (Token, AstIndex) {
        let advanced = self.advance_if(source_data, Some);
        if let NextToken(result, index) = advanced {
            (result, index)
        } else {
            panic!("Internal Compiler Error: Walker algorithm is wrong or parser built a bad tree. Advanced: {:?}. Walker: {:?}. SourceData: {:?}", advanced, self, source_data);
        }
    }

    fn advance_if<T, F: Fn(Token)->Option<T>>(
        &mut self,
        source_data: &SourceData,
        match_token: F
    ) -> Advance<T> {
        if self.is_at_eof(source_data) {
            return Eof;
        }

        let token = *source_data.token(self.index);
        if let Some(result) = match_token(token) {
            let index = self.index;
            self.index += 1;
            NextToken(result, index)
        } else {
            NoMatch
        }
    }

    fn is_at_eof(&self, source_data: &SourceData) -> bool {
        self.index >= source_data.num_tokens()
    }
}

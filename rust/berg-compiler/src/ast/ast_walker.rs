use std::fmt::Debug;
use ast::{AstIndex,IdentifierIndex};
use ast::ast_walker::Advance::*;
use ast::token::{Token,TermToken,InfixToken};
use ast::token::Token::*;
use ast::token::ExpressionBoundary::*;
use ast::token::Fixity::*;
use compiler::source_data::ParseData;

pub(crate) trait AstVisitorMut<T> {
    fn visit_term(&mut self, token: TermToken, index: AstIndex, parse_data: &ParseData) -> T;
    fn visit_postfix(&mut self, postfix: IdentifierIndex, operand: T, index: AstIndex, parse_data: &ParseData) -> T;
    fn visit_prefix(&mut self, prefix: IdentifierIndex, operand: T, index: AstIndex, parse_data: &ParseData) -> T;
    fn visit_infix(&mut self, token: InfixToken, left: T, right: T, index: AstIndex, parse_data: &ParseData) -> T;
    fn visit_parentheses(&mut self, value: T, open_index: AstIndex, close_index: AstIndex, parse_data: &ParseData) -> T;
}

#[derive(Debug)]
enum Advance<T> {
    NextToken(T, AstIndex),
    NoMatch,
    Eof,
}

#[derive(Debug,Copy,Clone,Default)]
pub(crate) struct AstWalkerMut {
    index: AstIndex
}

impl AstWalkerMut {
    pub(crate) fn walk<T: Debug, V: AstVisitorMut<T>>(visitor: &mut V, parse_data: &ParseData) -> T {
        let mut walker = AstWalkerMut { index: AstIndex(0) };
        let mut value = walker.walk_expression(visitor, parse_data);

        // If there are extra close operators, report them.
        while let NextToken(..) = walker.advance_if(parse_data,
            |token| match token { Close(..) => Some(()), _ => None }) {
            // Walk any remaining postfixes.
            value = walker.walk_postfixes(visitor, value, parse_data);
            // Read any remaining infixes as well.
            value = walker.walk_infix(visitor, value, parse_data)
        }
        assert!(walker.is_at_eof(parse_data));
        value
    }

    fn walk_expression<T: Debug, V: AstVisitorMut<T>>(&mut self, visitor: &mut V, parse_data: &ParseData) -> T {
        let left = self.walk_infix_operand(visitor, parse_data);
        self.walk_infix(visitor, left, parse_data)
    }

    fn walk_infix<T: Debug, V: AstVisitorMut<T>>(&mut self, visitor: &mut V, left: T, parse_data: &ParseData) -> T {
        self.walk_infix_while(visitor, left, parse_data, |_| true)
    }

    fn walk_infix_while<T: Debug, V: AstVisitorMut<T>, F: Fn(InfixToken)->bool>(
        &mut self,
        visitor: &mut V,
        mut left: T,
        parse_data: &ParseData,
        walk_if: F
    ) -> T {
        while let NextToken(infix, infix_index) = self.advance_if(parse_data,
            |token| match InfixToken::try_from(token) { Some(infix) if walk_if(infix) => Some(infix), _ => None })
        {
            // Get the right operand.
            let mut right = self.walk_infix_operand(visitor, parse_data);

            // Apply the operator now that we've grabbed anything we needed from the right!
            println!("Visit infix {:?} {:?}, {:?}", infix, left, right);
            left = visitor.visit_infix(infix, left, right, infix_index, parse_data);
        }
        left
    }

    fn walk_infix_operand<T: Debug, V: AstVisitorMut<T>>(&mut self, visitor: &mut V, parse_data: &ParseData) -> T {
        // Skip any prefixes (we'll apply them after we calculate the term)
        let first_prefix = self.index;
        while let NextToken(..) = self.advance_if(parse_data, |token|
            match token.fixity() { Prefix => Some(()), _ => None }
        ) {
            // Do-nothing loop, we're just skipping them to be dealt with later
        }

        // Handle the term
        let (term_token, term_index) = self.advance(parse_data);
        let term = term_token.to_term().unwrap();
        let mut value = visitor.visit_term(term, term_index, parse_data);

        // Handle prefixes (in reverse order)
        let mut prefix_index = term_index;
        while prefix_index > first_prefix {
            prefix_index -= 1;
            let prefix_token = (*parse_data.token(prefix_index)).to_prefix().unwrap();
            use ast::token::PrefixToken::*;
            match prefix_token {
                PrefixOperator(prefix) => {
                    println!("Visit prefix {:?} {:?}", &parse_data.identifiers[prefix], value);
                    value = visitor.visit_prefix(prefix, value, term_index, parse_data);
                },
                // Handle parentheses
                Open(open_boundary,delta) => {
                    // Walk the remainder of the expression in the parens (we already got the term)
                    value = self.walk_postfixes(visitor, value, parse_data);
                    value = self.walk_infix(visitor, value, parse_data);
                    assert_eq!(delta, self.index-prefix_index);

                    // Skip the close token
                    if let NextToken(close_delta, close_index) = self.advance_if(parse_data, |token| {
                        match token {
                            Close(close_boundary, delta) if close_boundary == open_boundary => Some(delta),
                            _ => None
                        }
                    }) {
                        assert_eq!(close_delta, delta);
                        assert_eq!(delta, close_index-prefix_index);
                        if open_boundary == Parentheses {
                            value = visitor.visit_parentheses(value, prefix_index, close_index, parse_data);
                        }
                    } else {
                        unreachable!();
                    }
                },
            };
        };

        // Now a apply any postfixes. Stop when we see a close operator.
        self.walk_postfixes(visitor, value, parse_data)
    }

    fn walk_postfixes<T: Debug, V: AstVisitorMut<T>>(&mut self, visitor: &mut V, mut value: T, parse_data: &ParseData) -> T {
        while let NextToken(postfix, postfix_index) = self.advance_if(parse_data,
            |token| match token { PostfixOperator(postfix) => Some(postfix), _ => None })
        {
            println!("Visit postfix {:?} {:?}", &parse_data.identifiers[postfix], value);
            value = visitor.visit_postfix(postfix, value, postfix_index, parse_data)
        }
        value
    }

    fn advance(&mut self, parse_data: &ParseData) -> (Token, AstIndex) {
        let advanced = self.advance_if(parse_data, Some);
        if let NextToken(result, index) = advanced {
            (result, index)
        } else {
            panic!("Internal Compiler Error: Walker algorithm is wrong or parser built a bad tree. Advanced: {:?}. Walker: {:?}. ParseData: {:?}", advanced, self, parse_data);
        }
    }

    fn advance_if<T, F: Fn(Token)->Option<T>>(
        &mut self,
        parse_data: &ParseData,
        match_token: F
    ) -> Advance<T> {
        if self.is_at_eof(parse_data) {
            return Eof;
        }

        let token = *parse_data.token(self.index);
        if let Some(result) = match_token(token) {
            let index = self.index;
            self.index += 1;
            NextToken(result, index)
        } else {
            NoMatch
        }
    }

    fn is_at_eof(&self, parse_data: &ParseData) -> bool {
        self.index >= parse_data.num_tokens()
    }
}

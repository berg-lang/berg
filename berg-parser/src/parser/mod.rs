mod binder;
mod grouper;
mod scanner;
mod sequencer;
mod tokenizer;

use std::borrow::Cow;

use sequencer::Sequencer;

use crate::syntax::{ast::Ast, bytes::ByteSlice};

///
/// Opens and parses the source into an AST.
///
/// # Errors
///
/// This *always* returns an AST, even if the source cannot be opened or
/// is unparseable. In these error cases there will be error terms that,
/// when executed, produce the appropriate error.
///
/// Errors placed in the AST include any parse error or open error.
///
pub fn parse(buffer: Cow<'static, ByteSlice>) -> Ast {
    let ast = Sequencer::new(buffer).parse();
    println!();
    println!("Parsed:");
    let mut level = 0;
    for i in 0..ast.tokens.len() {
        use crate::syntax::token::{ExpressionToken, OperatorToken, Token};
        let token = ast.token(i.into());
        let token_range = ast.token_range(i.into());
        if let Token::Operator(OperatorToken::Close(..)) = token {
            level -= 1
        }
        if let Token::Operator(OperatorToken::CloseBlock(..)) = token {
            level -= 1
        }
        println!(
            "{:>3} {:<indent$}{:<16} | {:?}  at {}..{}",
            i,
            "",
            ast.visible_token_string(i.into()),
            token,
            token_range.start,
            token_range.end,
            indent = level * 4
        );
        if let Token::Expression(ExpressionToken::Open(..)) = token {
            level += 1
        }
    }
    println!();
    ast
}

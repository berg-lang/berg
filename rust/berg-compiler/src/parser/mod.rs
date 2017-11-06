mod scanner;
mod tokenizer;

use ast::AstIndex;
use ast::token::Token;
use ast::token::Token::*;
use compiler::Compiler;
use compiler::compile_errors;
use compiler::source_data::{ByteIndex,ByteRange,ParseData,SourceIndex};
use indexed_vec::{Delta,IndexedVec};

pub(super) fn parse(compiler: &Compiler, source: SourceIndex) -> ParseData
{
    // Actual list of tokens
    let mut tokens: IndexedVec<Token,AstIndex> = Default::default();
    let mut token_ranges: IndexedVec<ByteRange,AstIndex> = Default::default();
    // Keeps track of open parentheses so we can close them appropriately
    let mut open_parens: Vec<AstIndex> = Default::default();

    // Loop through tokens, inserting term, then operator, then term, then operator ...
    let mut need_operand = true;
    let (identifiers, literals) = tokenizer::tokenize(compiler, source, |mut token, range| {
        // Put a MissingExpression or MissingInfix in between if we're missing something.
        match (need_operand, token.has_left_operand()) {
            (true, true) => {
                tokens.push(MissingExpression);
                token_ranges.push(range.start..range.start);
            },
            (false, false) => {
                tokens.push(MissingInfix);
                token_ranges.push(range.start..range.start);
            },
            (true,false)|(false,true) => {}
        }
        need_operand = token.has_right_operand();

        // Handle open/close parens
        match token {
            OpenParen(_) => open_parens.push(tokens.len()),
            CloseParen(ref mut close_delta) => {
                *close_delta = match open_parens.pop() {
                    Some(open_index) => {
                        let close_index = tokens.len();
                        match tokens[open_index] {
                            OpenParen(ref mut open_delta) => { *open_delta = close_index-open_index; *open_delta }
                            _ => unreachable!()
                        }
                    }
                    None => handle_close_without_open(&range, &mut tokens, &mut token_ranges, compiler, source),
                }
            },
            _ => {}
        }

        // Insert the tokens
        tokens.push(token);
        token_ranges.push(range);
    });

    // Add a "missing operand" if we still need an operand
    if need_operand {
        let end = match token_ranges.last() { Some(range) => range.end, None => ByteIndex(0) };
        tokens.push(MissingExpression);
        token_ranges.push(end..end);
    }
    // Close off any remaining open parens
    while let Some(open_index) = open_parens.pop() {
        handle_open_without_close(open_index, &mut tokens, &mut token_ranges, compiler, source);
    }

    let char_data = Default::default();
    ParseData { char_data, identifiers, literals, tokens, token_ranges }
}

fn handle_close_without_open(
    range: &ByteRange,
    tokens: &mut IndexedVec<Token,AstIndex>,
    token_ranges: &mut IndexedVec<ByteRange,AstIndex>,
    compiler: &Compiler,
    source: SourceIndex
) -> Delta<AstIndex> {
    let open_index = AstIndex(0);
    let open_loc = ByteIndex(0);
    // We're about to shift the close_index over, so we need to set the delta correctly.
    let close_index = tokens.len()+1;
    let delta = close_index-open_index;
    println!("  open/close: {}/{}", open_index, close_index);
    tokens.insert(open_index, OpenParen(delta));
    token_ranges.insert(open_index, open_loc..open_loc);
    compiler.report(compile_errors::CloseWithoutOpen { source, close: range.clone(), open: String::from("(") });
    delta
}

fn handle_open_without_close(
    open_index: AstIndex,
    tokens: &mut IndexedVec<Token,AstIndex>,
    token_ranges: &mut IndexedVec<ByteRange,AstIndex>,
    compiler: &Compiler,
    source: SourceIndex
) {
    let close_index = tokens.len();
    let close_loc = token_ranges[close_index-1].end;
    let delta = close_index-open_index;
    tokens.push(CloseParen(delta));
    token_ranges.push(close_loc..close_loc);
    if let OpenParen(ref mut open_delta) = tokens[open_index] {
        *open_delta = delta;
    } else {
        unreachable!()
    }

    compiler.report(compile_errors::OpenWithoutClose { source, open: token_ranges[open_index].clone(), close: String::from(")") });
}

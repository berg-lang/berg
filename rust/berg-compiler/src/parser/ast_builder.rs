use ast::AstIndex;
use ast::token::Token;
use ast::token::Token::*;
use compiler::compile_errors::{SourceCompileErrors,CompileErrorType};
use compiler::source_data::{ByteIndex,ByteRange,ByteSlice,ParseData};
use indexed_vec::{Delta,IndexedVec};
use parser::tokenizer;
use std::str;

#[derive(Debug,Default)]
pub(super) struct AstBuilder {
    pub tokens: IndexedVec<Token,AstIndex>,
    pub token_ranges: IndexedVec<ByteRange,AstIndex>,
    open_parens: Vec<AstIndex>,
}

pub(super) fn build_ast(buffer: &ByteSlice, errors: &mut SourceCompileErrors) -> ParseData
{
    let mut tokens: IndexedVec<Token,AstIndex> = Default::default();
    let mut token_ranges: IndexedVec<ByteRange,AstIndex> = Default::default();
    let mut open_parens: Vec<AstIndex> = Default::default();

    // Loop through tokens, inserting term, then operator, then term, then operator ...
    let (identifiers, literals) = tokenizer::tokenize(buffer, errors, |mut token, range, errors| {
        println!("TOKEN: {:?}", token);
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
                    None => handle_close_without_open(buffer, &range, &mut tokens, &mut token_ranges, errors),
                }
            },
            _ => {}
        }

        // Insert the tokens
        tokens.push(token);
        token_ranges.push(range);
    });

    while let Some(open_index) = open_parens.pop() {
        handle_open_without_close(buffer, open_index, &mut tokens, &mut token_ranges, errors);
    }
    let char_data = Default::default();
    ParseData { char_data, identifiers, literals, tokens, token_ranges }
}

fn handle_close_without_open(
    buffer: &ByteSlice,
    range: &ByteRange,
    tokens: &mut IndexedVec<Token,AstIndex>,
    token_ranges: &mut IndexedVec<ByteRange,AstIndex>,
    errors: &mut SourceCompileErrors
) -> Delta<AstIndex> {
    let open_index = AstIndex(0);
    let open_loc = ByteIndex(0);
    // We're about to shift the close_index over, so we need to set the delta correctly.
    let close_index = tokens.len()+1;
    let delta = close_index-open_index;
    println!("  open/close: {}/{}", open_index, close_index);
    tokens.insert(open_index, OpenParen(delta));
    token_ranges.insert(open_index, open_loc..open_loc);
    let string = unsafe { str::from_utf8_unchecked(&buffer[range]) };
    errors.report_at(CompileErrorType::CloseWithoutOpen, range.clone(), string);
    delta
}

fn handle_open_without_close(
    buffer: &ByteSlice,
    open_index: AstIndex,
    tokens: &mut IndexedVec<Token,AstIndex>,
    token_ranges: &mut IndexedVec<ByteRange,AstIndex>,
    errors: &mut SourceCompileErrors
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

    let string = unsafe { str::from_utf8_unchecked(&buffer[&token_ranges[open_index]]) };
    errors.report_at(CompileErrorType::OpenWithoutClose, token_ranges[open_index].clone(), string);
}

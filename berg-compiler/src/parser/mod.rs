mod binder;
mod grouper;
mod sequencer;
mod tokenizer;

use crate::syntax::{Ast, AstRef, SourceBuffer, SourceRef};
use binder::Binder;
use grouper::Grouper;
use sequencer::Sequencer;
use tokenizer::Tokenizer;

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
pub fn parse(source: SourceRef) -> AstRef {
    let SourceBuffer {
        buffer,
        source_open_error,
    } = source.open();
    let sequencer = Sequencer::new(Ast::new(source, source_open_error));
    let ast = AstRef::new(sequencer.parse_buffer(&buffer));
    println!();
    println!("Parsed:");
    print!("{}", ast.expression().format_tree());
    for i in 0..ast.tokens.len() {
        use crate::syntax::AstIndex;
        println!("{:?} = {:?}", ast.token_ranges[AstIndex::from(i)], ast.tokens[AstIndex::from(i)])
    }
    ast
}

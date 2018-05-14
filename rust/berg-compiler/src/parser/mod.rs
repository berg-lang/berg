mod binder;
mod grouper;
mod sequencer;
mod tokenizer;

use syntax::{AstRef, AstData, SourceRef, SourceBuffer};
use parser::sequencer::Sequencer;

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
    let SourceBuffer { buffer, source_open_error } = source.open();
    let ast = AstData::new(source, source_open_error);
    let sequencer = Sequencer::new(ast);
    AstRef::new(sequencer.parse_buffer(&buffer))
}

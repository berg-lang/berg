mod internals;
mod source_reader;

use compiler::parser::internals::*;

/// Shared parsing state
pub struct Parser {
    /// The stream we are parsing.
    reader: SourceReader,
}

impl Parser {
    pub fn parse(source: &Source) -> Parser {
        let errors = CompileErrorReporter::new();
        let reader = SourceReader::start(source, errors);
        Parser { reader }
    }
}

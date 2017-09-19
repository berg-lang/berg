mod internals;
mod source_reader;
mod syntax_expression;
mod tokenizer;

pub use parser::syntax_expression::SyntaxExpression;
pub use parser::syntax_expression::SyntaxExpressionType;

use parser::internals::*;

/// Shared parsing state
pub struct Parser {
    tokenizer: Tokenizer,
}

pub struct ParseResult {
    metadata: SourceMetadata,
    expressions: Vec<SyntaxExpression>,
    errors: CompileErrors,
}

impl Parser {
    pub fn parse(source: &Source) -> ParseResult {
        let mut parser = Self::new(source);
        parser.step();
        parser.close()
    }

    fn new(source: &Source) -> Parser {
        let errors = CompileErrors::new();
        let reader = SourceReader::start(source, errors);
        let tokenizer = Tokenizer::new(reader);
        Parser { tokenizer }
    }

    fn step(&mut self) {
        self.tokenizer.next();
    }

    fn close(self) -> ParseResult {
        let (metadata, expressions, errors) = self.tokenizer.close();
        ParseResult { metadata, expressions, errors }
    }
}

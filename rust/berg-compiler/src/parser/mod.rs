mod internals;
mod parse_result;
mod source_reader;
mod syntax_expression;
mod tokenizer;

pub use parser::parse_result::ParseResult;
pub use parser::syntax_expression::SyntaxExpression;
pub use parser::syntax_expression::SyntaxExpressionType;

use parser::internals::*;

/// Shared parsing state
pub struct Parser {
    tokenizer: Tokenizer,
}

impl Parser {
    pub fn parse<'s>(source: &'s Source) -> ParseResult<'s> {
        let mut parser = Self::new(source);
        parser.step();
        parser.close(source)
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

    fn close<'s>(self, source: &'s Source) -> ParseResult<'s> {
        let (metadata, expressions, errors) = self.tokenizer.close();
        ParseResult { source, metadata, expressions, errors }
    }
}

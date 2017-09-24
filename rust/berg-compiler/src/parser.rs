use berg::*;
use source_reader::*;
use tokenizer::*;

/// Shared parsing state
pub struct Parser<'a, R: SourceReader + 'a> {
    tokenizer: Tokenizer<'a, R>,
}

impl<'a, R: SourceReader + 'a> Parser<'a, R> {
    pub fn parse(mut reader: R, berg: &Berg) -> ParseResult {
        let expressions = {
            let mut parser = Self::new(&mut reader);
            if parser.open(berg) {
                while parser.step() {};
            }
            parser.close()
        };
        let (metadata, errors) = reader.close();
        ParseResult { metadata, expressions, errors }
    }
    fn new(reader: &'a mut R) -> Parser<'a, R> {
        let tokenizer = Tokenizer::new(reader);
        Parser { tokenizer }
    }
    fn open(&mut self, berg: &Berg) -> bool {
        self.tokenizer.open(berg)
    }

    fn step(&mut self) -> bool {
        if let Some(_) = self.tokenizer.next() {
            true
        } else {
            false
        }
    }

    fn close(self) -> Vec<SyntaxExpression> {
        self.tokenizer.close()
    }
}

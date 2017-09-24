use berg::*;
use source_reader::*;
use tokenizer::*;

/// Shared parsing state
pub struct Parser<'a, R: SourceReader + 'a> {
    tokenizer: Tokenizer<'a, R>,
}

impl<'a, R: SourceReader + 'a> Parser<'a, R> {
    pub fn new(reader: &'a mut R) -> Parser<'a, R> {
        let tokenizer = Tokenizer::new(reader);
        Parser { tokenizer }
    }
    pub fn parse(mut self) -> Vec<SyntaxExpression> {
        while self.step() {};
        self.close()
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

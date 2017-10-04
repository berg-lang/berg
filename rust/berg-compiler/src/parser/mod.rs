pub mod char_data;
pub mod lexer;
pub mod source_buffer;
pub mod syntax_expression;

use public::*;
use parser::source_buffer::SourceBuffer;

/// Shared parsing state
#[derive(Debug)]
pub struct Parser<'p, 'c: 'p> {
    pub compiler: &'p Compiler<'c>,
    pub source: SourceIndex,
    pub buffer: &'p [u8],
    pub index: ByteIndex,
    pub char_data: CharData,
    pub expressions: Vec<SyntaxExpression>,
}

impl<'p, 'c: 'p> Parser<'p, 'c> {
    pub fn parse(compiler: &'c Compiler, source: SourceIndex) {
        let index = 0;
        let char_data = CharData::new();
        let expressions = vec![];
        let (char_data, expressions) = compiler.with_source(source, |s| {
            SourceBuffer::with_buffer(compiler, source, &s.source, |buffer| {
                let mut parser = Parser { compiler, source, buffer, index, char_data, expressions };
                while parser.step() {};
                (parser.char_data, parser.expressions)
            })
        });
        compiler.with_source_mut(source, |s| {
            s.char_data = Some(char_data);
            s.expressions = Some(expressions);
        });
    }

    pub fn report(&self, error: CompileError) {
        self.compiler.report(error)
    }

    fn step(&mut self) -> bool {
        lexer::term(self)
    }
}


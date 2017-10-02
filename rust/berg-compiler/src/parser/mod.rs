pub mod results;
mod grammar;
mod scanner;
mod stream_buffer;

use compiler::*;
use parser::grammar::*;
use parser::results::*;
use parser::stream_buffer::*;

use std::io;
use std::path::PathBuf;
use std::fs::File;

/// Shared parsing state
pub struct Parser<'p, 'c: 'p, Buf: StreamBuffer + 'p> {
    compiler: &'p Compiler<'c>,
    source: &'c Source,
    stream: Buf,
    char_data: CharData,
    expressions: Vec<SyntaxExpression>,
}

pub fn parse<'c>(compiler: &'c Compiler<'c>, source: &'c Source) -> Option<ParseResult> {
    match source {
        &Source::File(ref path) => {
            if let Some(stream) = open_file(compiler, source, path) {
                Some(Parser::parse_stream(stream, source, compiler))
            } else {
                None
            }
        }
        &Source::Memory(_, ref contents) => {
            let stream = MemoryStreamBuffer::new(contents);
            Some(Parser::parse_stream(stream, source, compiler))
        }
    }
}

fn open_file<'c>(compiler: &'c Compiler<'c>, source: &'c Source, path: &PathBuf) -> Option<IoStreamBuffer<File>> {
    let file = File::open(path);
    match file {
        Ok(read) => Some(IoStreamBuffer::new(read, 0)),
        Err(error) => {
            let error_type = match error.kind() {
                io::ErrorKind::NotFound => SourceNotFound,
                _ => IoOpenError,
            };
            compiler.report(error_type.io_open(source, error, path));
            None
        },
    }
}

impl<'p, 'c: 'p, Buf: StreamBuffer + 'p> Parser<'p, 'c, Buf> {
    fn parse_stream(stream: Buf, source: &'c Source, compiler: &'p Compiler<'c>) -> ParseResult {
        let char_data = CharData::new();
        let expressions = vec![];
        let mut parser = Parser { compiler, source, stream, char_data, expressions };
        while parser.step() {};
        parser.close()
    }

    fn report(&mut self, error: CompileError<'c>) {
        self.compiler.report(error)
    }

    fn step(&mut self) -> bool {
        Grammar::term(self)
    }

    fn close(self) -> ParseResult {
        ParseResult::new(self.char_data, self.expressions)
    }
}


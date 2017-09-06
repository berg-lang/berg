mod scanner;

use source::Source;
use compiler::Compiler;
use std::io;
use std::io::Read;

pub struct Parser<'a, R: io::Read> {
    compiler: &'a Compiler,
    source_index: usize,
    reader: io::BufReader<R>,
}

impl<'a, R: io::Read> Parser<'a, R> {
    pub fn new(compiler: &Compiler, source_index: usize, reader: io::BufReader<R>) -> Parser<R> {
        Parser { compiler, source_index, reader }
    }
    pub fn source(&mut self) -> &Source {
        &self.compiler.sources[self.source_index]
    }
    pub fn parse(&mut self) {
        let mut str = String::new();
        self.source().name();
        self.reader.read_to_string(&mut str).unwrap();
        println!("{}", str)
    }
}
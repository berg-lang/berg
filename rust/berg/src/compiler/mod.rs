mod parser;

use std::env;
use std::vec::Vec;
use std::iter::FromIterator;
use std::io;
use source::Source;
use compiler::parser::Parser;

pub struct Compiler {
    sources: Vec<Source>,
}

impl Compiler {
    pub fn from_env() -> Compiler {
        let arg_sources = env::args().map( |arg| Source::from_env(arg) );
        let sources = Vec::from_iter(arg_sources);
        Compiler { sources }
    }

    pub fn parse(&mut self) -> io::Result<()> {
        for (index,source) in self.sources.iter().enumerate() {
            match source {
                &Source::File(ref f) => {
                    let mut parser = Parser::new(&self, index, f.open()?);
                    parser.parse();
                },
            };
        }
        Ok(())
    }
}

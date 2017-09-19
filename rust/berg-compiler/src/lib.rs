#![feature(io)]

mod compile_error;
mod compile_errors;
mod internals;
mod line_column;
mod parser;
mod source;
mod source_metadata;

use internals::*;

pub struct Compiler {
    sources: Vec<Source>,
}

impl Compiler {
    pub fn from_env() -> Compiler {
        let arg_sources = env::args().map( |arg| Source::from_arg(arg) );
        let sources = Vec::from_iter(arg_sources);
        Compiler { sources }
    }

    pub fn parse(&self) {
        for source in &self.sources {
            Parser::parse(source);
        }
    }
}

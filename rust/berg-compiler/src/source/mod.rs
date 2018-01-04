#[macro_use]
mod compile_error_macros;

pub mod compile_errors;
pub(crate) mod line_column;
pub(crate) mod source_spec;
pub(crate) mod parse_result;

use interpreter::value::Block;
use std::cell::Ref;
use source::parse_result::ByteSlice;
use source::parse_result::ParseResult;
use std::borrow::Cow;
use compiler::Compiler;
use std::cell::RefCell;
use interpreter::evaluator::BlockState;
use source::source_spec::SourceSpec;
use std::ffi::OsStr;
use std::u32;
use parser;

index_type! {
    pub struct SourceIndex(pub u32) <= u32::MAX;
}

#[derive(Debug)]
pub struct Source {
    pub(crate) index: SourceIndex,
    pub(crate) source_spec: SourceSpec,
    pub(crate) parse_result: RefCell<ParseResult>,
    pub(crate) value: Block,
}

impl Source {
    pub(crate) fn new(source_spec: SourceSpec, index: SourceIndex) -> Source {
        Source {
            source_spec,
            index,
            parse_result: RefCell::new(ParseResult::new(index)),
            value: Block::new(BlockState::SourceNotStarted(index)),
        }
    }

    pub(crate) fn open(
        &self,
        compiler: &Compiler,
        parse_result: &mut ParseResult,
    ) -> Cow<ByteSlice> {
        self.source_spec.open(compiler, parse_result)
    }

    pub(crate) fn reopen(&self, compiler: &Compiler) -> Cow<ByteSlice> {
        self.source_spec.reopen(compiler)
    }

    pub(crate) fn parse<'a>(&'a self, compiler: &Compiler) -> Ref<'a, ParseResult> {
        {
            let mut parse_result = self.parse_result.borrow_mut();

            assert!(!parse_result.is_parsed);
            let buffer = self.open(compiler, &mut parse_result);
            parser::parse(&buffer, &mut parse_result);
            parse_result.is_parsed = true;

            println!("--------------------");
            println!("PARSE RESULT:");
            print!("{}", parse_result);
        }
        self.parse_result()
    }

    pub fn parse_result(&self) -> Ref<ParseResult> {
        let result = self.parse_result.borrow();
        assert!(result.is_parsed);
        result
    }

    pub(crate) fn name(&self) -> &OsStr {
        self.source_spec.name()
    }
}

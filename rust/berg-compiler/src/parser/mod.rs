mod ast_builder;
mod scanner;
mod tokenizer;

use compiler::Compiler;
use compiler::compile_errors::SourceCompileErrors;
use compiler::source_data::ParseData;
use compiler::source_spec::SourceSpec;

pub(crate) fn parse(
    compiler: &Compiler,
    errors: &mut SourceCompileErrors,
    source_spec: &SourceSpec
) -> ParseData {
    let buffer = source_spec.open(compiler, errors);
    ast_builder::build_ast(&buffer, errors)
}

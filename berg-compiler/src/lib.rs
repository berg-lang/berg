// Turn on all warnings
#![warn(clippy::all)]

use value::RootRef;

// Explicitly expose just the interfaces we want to expose
pub use crate::value::{
    BergResult, BergVal, BergValue, CompilerErrorCode, EvalException, Exception,
};

use crate::eval::evaluate_ast;
use std::borrow::Cow;
use std::path::Path;
#[macro_use]
extern crate matches;

pub(crate) mod eval;
pub(crate) mod value;

pub mod test;

pub fn evaluate_file(path: impl Into<Cow<'static, Path>>) -> impl BergValue {
    let ast = RootRef::from_env().parse_file(path);
    evaluate_ast(ast)
}

pub fn evaluate_bytes(
    name: impl Into<Cow<'static, str>>,
    buffer: impl Into<Cow<'static, [u8]>>,
) -> impl BergValue {
    let ast = RootRef::from_env().parse_bytes(name, buffer);
    evaluate_ast(ast)
}

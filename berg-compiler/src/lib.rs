// Turn on all warnings
#![warn(clippy::all)]

// Used for the faster hash algorithm
extern crate fnv;
// Used for arbitrary-precision numbers
extern crate num;
// Used to pool identifiers and operators (making identifier comparison faster)
extern crate string_interner;

// Explicitly expose just the interfaces we want to expose
pub use crate::value::{BergResult, BergVal, BergValue, ErrorVal, Error, ErrorCode};

use crate::eval::{evaluate_ast, RootRef};
use crate::parser::parse;
use crate::syntax::SourceRef;
use std::borrow::Cow;
use std::path::Path;

#[macro_use]
pub(crate) mod util;
#[macro_use]
pub(crate) mod syntax;
pub(crate) mod parser;
pub(crate) mod eval;
pub(crate) mod value;

pub mod test;

pub fn evaluate_file<'a, P: Into<Cow<'a, Path>>>(path: P) -> impl BergValue<'a> {
    let source = SourceRef::file(path.into(), RootRef::from_env());
    evaluate_ast(parse(source))
}

pub fn evaluate_bytes<'a>(name: &'a str, value: &'a [u8]) -> impl BergValue<'a> {
    let source = SourceRef::memory(name, value, RootRef::from_env());
    evaluate_ast(parse(source))
}

#![warn(clippy::all)]

extern crate fnv;
extern crate num;

pub use crate::value::{BergVal, BergValue, BergResult, Error, ErrorCode, Nothing};

use crate::eval::{RootRef, evaluate_ast};
use crate::syntax::SourceRef;
use crate::parser::parse;
use std::borrow::Cow;
use std::path::Path;

#[macro_use]
pub(crate) mod util;
pub(crate) mod eval;
pub(crate) mod parser;
pub(crate) mod syntax;
pub mod test;
pub(crate) mod value;

pub fn evaluate_file<'a, P: Into<Cow<'a, Path>>>(path: P) -> BergResult<'a> {
    let source = SourceRef::file(path.into(), RootRef::from_env());
    evaluate_ast(parse(source))
}

pub fn evaluate_bytes<'a>(name: &'a str, value: &'a [u8]) -> BergResult<'a> {
    let source = SourceRef::memory(name, value, RootRef::from_env());
    evaluate_ast(parse(source))
}

// #[cfg(test)]
// pub mod test;

// use value::Val;
// use std::path::PathBuf;

// use root::from_env;

// pub fn evaluate_file<P: Into<PathBuf>>(path: P) -> Val<'static> {
//     from_env().evaluate_file(path)
// }

// pub fn evaluate_bytes<'a>(name: &'a str, bytes: &'a [u8]) -> Val<'a> {
//     from_env().evaluate_bytes(name, bytes)
// }

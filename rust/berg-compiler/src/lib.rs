#![warn(clippy::all)]

extern crate fnv;
extern crate num;

pub use crate::error::{BergResult, Error, ErrorCode};
pub use crate::value::{BergVal, BergValue, Nothing};

use crate::eval::RootRef;
use crate::syntax::SourceRef;
use std::borrow::Cow;
use std::path::Path;

#[macro_use]
pub(crate) mod util;
pub(crate) mod error;
pub(crate) mod eval;
pub(crate) mod parser;
pub(crate) mod syntax;
pub mod test;
pub(crate) mod value;

pub fn evaluate_file<'a, P: Into<Cow<'a, Path>>>(path: P) -> BergResult<'a> {
    let root = RootRef::from_env();
    let source = SourceRef::file(path.into(), root);
    parser::parse(source).result()
}

pub fn evaluate_bytes<'a>(name: &'a str, value: &'a [u8]) -> BergResult<'a> {
    let root = RootRef::from_env();
    let source = SourceRef::memory(name, value, root);
    parser::parse(source).result()
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

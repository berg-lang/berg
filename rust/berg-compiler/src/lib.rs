#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]

extern crate fnv;
extern crate num;

pub use error::{BergResult, Error, ErrorCode};
pub use value::{BergVal, Nothing};

use eval::RootRef;
use syntax::SourceRef;
use std::path::Path;
use std::borrow::Cow;

#[macro_use]
pub(crate) mod util;
pub(crate) mod eval;
pub(crate) mod error;
pub(crate) mod parser;
pub(crate) mod syntax;
pub(crate) mod value;
pub mod test;

pub fn evaluate_file<'a, P: Into<Cow<'a, Path>>>(path: P) -> BergResult<'a> {
    let root = RootRef::from_env();
    SourceRef::file(path.into(), root).evaluate()
}

pub fn evaluate_bytes<'a>(name: &'a str, value: &'a [u8]) -> BergResult<'a> {
    let root = RootRef::from_env();
    SourceRef::memory(name, value, root).evaluate()
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

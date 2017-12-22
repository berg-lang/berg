#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

extern crate num;
extern crate fnv;

#[macro_use]
pub(crate) mod util;
mod ast;
mod compiler;
mod interpreter;
mod parser;
mod public;
mod source;
pub mod test;

pub use public::*;

#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]

extern crate num;
extern crate fnv;

#[macro_use]
pub mod util;
mod ast;
mod compiler;
mod interpreter;
mod parser;
mod public;
mod source;
#[cfg(test)]
pub mod test;

pub use public::*;

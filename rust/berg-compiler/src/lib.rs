#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

extern crate num;
extern crate fnv;

#[macro_use]
mod indexed_vec;
mod ast;
mod checker;
mod compiler;
mod parser;
mod public;

pub use public::*;

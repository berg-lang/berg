#![feature(plugin)]
#![plugin(clippy)]

extern crate num;

#[macro_use]
mod indexed_vec;
mod checker;
mod compiler;
mod parser;
mod public;

pub use public::*;

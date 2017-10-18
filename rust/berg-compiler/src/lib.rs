#![feature(exclusive_range_pattern)]
#![feature(inclusive_range_syntax)]
#![feature(inclusive_range)]
#![feature(range_contains)]
#![feature(collections_range)]
#![feature(step_trait)]
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


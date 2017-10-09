#![feature(exclusive_range_pattern)]
#![feature(inclusive_range_syntax)]
#![feature(inclusive_range)]
#![feature(range_contains)]
#![feature(collections_range)]

extern crate num;

mod compiler;
mod parser;
mod platonic_runtime;
mod public;

pub use public::*;

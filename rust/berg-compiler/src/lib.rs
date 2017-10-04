#![feature(exclusive_range_pattern)]
#![feature(inclusive_range_syntax)]
#![feature(inclusive_range)]
#![feature(range_contains)]
#![feature(collections_range)]

mod compiler;
mod parser;
mod public;

pub use public::*;

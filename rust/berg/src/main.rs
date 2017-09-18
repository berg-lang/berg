#![feature(io)]
#![allow(dead_code)]
extern crate env_logger;

mod compiler;

use compiler::Compiler;

fn main() {
    // Initialize logging
    env_logger::init().unwrap();

    // Get filename from command line
    let compiler = Compiler::from_env();
    compiler.parse();
}

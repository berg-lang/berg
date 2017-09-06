extern crate log;
extern crate env_logger;

mod compiler;
mod source;

use compiler::Compiler;

fn main() {
    // Initialize logging
    env_logger::init().unwrap();

    // Get filename from command line
    let mut compiler = Compiler::from_env();
    compiler.parse().unwrap()
}

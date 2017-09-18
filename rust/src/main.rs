//#[macro_use] extern crate log;
extern crate env_logger;
extern crate berg_compiler;

use berg_compiler::Compiler;

fn main() {
    // Initialize logging
    env_logger::init().unwrap();

    // Get filename from command line
    let compiler = Compiler::from_env();
    compiler.parse();
}

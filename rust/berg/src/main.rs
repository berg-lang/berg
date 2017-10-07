//#[macro_use] extern crate log;
extern crate berg_compiler;
extern crate docopt;
extern crate env_logger;
#[macro_use]
extern crate serde_derive;

use berg_compiler::*;
use docopt::Docopt;

const USAGE: &str = "
Berg compiler and evaluator.

Usage:
  berg check syntax <file>
  berg check syntax -e <expr>

Options:
  -h --help     Show this screen.
  --version     Show version.
  -e <expr>     Run this expression.
";


#[derive(Debug, Deserialize)]
struct Args {
    arg_file: Option<String>,
    flag_e: Option<String>,
    cmd_check: bool,
    cmd_syntax: bool,
}

fn main() {
    env_logger::init().unwrap();
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());
    assert!(args.cmd_check);
    assert!(args.cmd_syntax);

    let mut compiler = Compiler::from_env();
    add_source(&mut compiler, &args);
}

fn add_source(compiler: &mut Compiler, args: &Args) {
    if let Some(ref file) = args.arg_file {
        assert!(args.flag_e.is_none());
        compiler.add_file_source(file)
    } else if let Some(ref expr) = args.flag_e {
        compiler.add_memory_source(String::from("expr"), expr.clone())
    } else {
        panic!("No source passed: {:?}", args)
    }
}

mod binder;
mod grouper;
mod sequencer;
mod tokenizer;

use syntax::{AstRef, SourceRef};

pub fn parse<'a>(source: &SourceRef<'a>) -> AstRef<'a> {
    sequencer::Sequencer::parse(source)
}

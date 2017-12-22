mod grouper;
mod sequencer;
mod tokenizer;

use source::parse_result::{ByteSlice,ParseResult};
use parser::sequencer::Sequencer;

pub(crate) fn parse(buffer: &ByteSlice, parse_result: &mut ParseResult) {
    Sequencer::new().parse(buffer, parse_result);
}

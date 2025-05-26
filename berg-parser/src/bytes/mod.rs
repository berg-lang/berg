use berg_util::{IndexedSlice, index_type};
use std::ops::Range;

pub mod line_column;

index_type! {
    pub struct ByteIndex(pub u32) with Display,Debug <= u32::MAX;
}
pub type ByteSlice = IndexedSlice<u8, ByteIndex>;
pub type ByteRange = Range<ByteIndex>;

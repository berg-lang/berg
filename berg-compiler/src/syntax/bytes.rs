use crate::util::indexed_vec::IndexedSlice;
use std::ops::Range;
use std::u32;

index_type! {
    pub struct ByteIndex(pub u32) with Display,Debug <= u32::MAX;
}
pub type ByteSlice = IndexedSlice<u8, ByteIndex>;
pub type ByteRange = Range<ByteIndex>;

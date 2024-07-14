#![feature(portable_simd)]
#![feature(array_chunks)]

mod byte_chunk;
mod byte_mask;
mod algorithms;

pub use byte_chunk::*;
pub use byte_mask::*;
pub use algorithms::*;

// mod simd;

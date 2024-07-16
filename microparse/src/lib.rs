#![feature(portable_simd)]
#![feature(array_chunks)]

// mod algorithms;
pub mod arch;
mod chunk;
mod mask;
pub mod simd;

// pub use algorithms::*;
pub use chunk::*;
pub use arch::Arch;

pub mod escape;
pub mod fmt;
pub mod preceded_by;
pub mod separator;
pub mod sequence;
pub mod mask64;
pub mod string;
pub mod test;

pub use mask64::*;
pub use fmt::MaskFmt;

pub const BLOCK_SIZE: usize = 64;

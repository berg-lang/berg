pub mod escape;
pub mod preceded_by;
pub mod separator;
pub mod sequence;
pub mod string;

pub type Mask64 = u64;
pub const BLOCK_SIZE: usize = 64;
pub const ALL: u64 = 0xFFFF_FFFF_FFFF_FFFF;
pub const NONE: u64 = 0x0000_0000_0000_0000;
pub const ODD_BITS: u64 = 0xAAAA_AAAA_AAAA_AAAA;
pub const LAST: u64 = 0x8000_0000_0000_0000;
pub const FIRST: u64 = 0x0000_0000_0000_0001;
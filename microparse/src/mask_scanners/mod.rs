mod escape_scanner;
mod preceded_by;
mod string_scanner;
pub use escape_scanner::*;
pub use string_scanner::*;
pub use preceded_by::*;

pub type Mask64 = u64;
pub const BLOCK_SIZE: usize = size_of::<Mask64>();
pub const ALL: u64 = 0xFFFF_FFFF_FFFF_FFFF;
pub const NONE: u64 = 0x0000_0000_0000_0000;
pub const ODD_BITS: u64 = 0xAAAA_AAAA_AAAA_AAAA;

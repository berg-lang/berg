pub mod arm64_neon;
mod prefix_xor;
pub use prefix_xor::prefix_xor;

pub use arm64_neon as default;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Arch {
    Arm64Neon,
}

impl Arch {
    // Since PartialEq / Eq don't work in const context, we make a const version here
    pub const fn eq(&self, other: &Self) -> bool {
        (*self as isize) == (*other as isize)
    }

    pub const fn name(&self) -> &'static str {
        self.data().name
    }

    pub const fn chunk_len(&self) -> usize {
        self.data().chunk_len
    }

    pub const fn is_supported(&self) -> bool {
        self.data().is_supported
    }

    pub const fn is_enabled(&self) -> bool {
        self.data().is_enabled
    }

    const fn data(&self) -> &'static ArchData {
        let data = match self {
            Arch::Arm64Neon => &arm64_neon::ARCH_DATA,
        };
        assert!(data.arch.eq(self));
        data
    }
}

struct ArchData {
    pub arch: Arch,
    pub name: &'static str,
    pub chunk_len: usize,
    pub is_supported: bool,
    pub is_enabled: bool,
}

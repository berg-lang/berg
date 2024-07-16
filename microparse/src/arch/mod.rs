pub mod arm64_neon;
use std::simd::SimdElement;

pub use arm64_neon as default;

pub enum Arch {
    Arm64Neon,
}

impl Arch {
    pub const fn name(&self) -> &'static str {
        self.data().name
    }

    pub const fn is_supported(&self) -> &'static str {
        self.data().name
    }

    const fn data(&self) -> &'static ArchData {
        match self {
            Arch::Arm64Neon => &arm64_neon::ARCH_DATA,
        }
    }
}

pub trait Implementation {
    type Chunk<T: SimdElement>: crate::AnyChunk<Elem = T>;
}

struct ArchData {
    pub arch: Arch,
    pub name: &'static str,
    pub chunk_len: usize,
    pub supported: bool,
    pub enabled: bool,
}

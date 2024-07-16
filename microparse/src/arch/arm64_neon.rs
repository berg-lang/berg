use std::simd::SimdElement;

use crate::Chunk;

use super::{Arch, ArchData, Implementation};

cfg_if::cfg_if! {
    if #[cfg(any(target_arch = "aarch64", target_arch = "arm64ec"))] {
        const IS_SUPPORTED: bool = true;
    } else {
        const IS_SUPPORTED: bool = false;
    }
}

pub(super) const ARCH_DATA: ArchData = ArchData {
    name: "arm64_neon",
    arch: Arch::Arm64Neon,
    chunk_len: 16,
    supported: IS_SUPPORTED,
    enabled: IS_SUPPORTED,
};

const SIMD_BYTES: usize = 16;

struct Arm64Neon();

impl Implementation for Arm64Neon {
    type Chunk<T: SimdElement> = Chunk<T, { SIMD_BYTES / size_of<T> }>;
}

use super::{Arch, ArchData};

cfg_if::cfg_if! {
    if #[cfg(all(target_arch = "aarch64", target_feature = "neon"))] {
        const IS_SUPPORTED: bool = true;
    } else {
        const IS_SUPPORTED: bool = false;
    }
}

pub(super) const ARCH_DATA: ArchData = ArchData {
    name: "arm64_neon",
    arch: Arch::Arm64Neon,
    chunk_len: 16,
    is_supported: IS_SUPPORTED,
    is_enabled: IS_SUPPORTED,
};

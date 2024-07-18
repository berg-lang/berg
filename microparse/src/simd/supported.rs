use std::simd::{LaneCount, Simd, SimdElement, SupportedLaneCount};

/// A supported Simd<u8, N>.
pub trait SupportedSimd {
    /// The element type in this Simd
    type Elem: SimdElement;
    /// Number of lanes
    type LaneCount: SupportedLaneCount;
    /// Width of entire simd in bits (Self::LaneCount::LANES * size_of::<Self::Elem>() * 8)
    type SimdWidth: SupportedSimdWidth;
}

/// A system width (in bits) for the Simd type, e.g. 128, 256, 512, etc.
/// Meant to be used for generic simd algorithms to find the appropriately-sized simd type for the
/// architecture they are on.
pub struct SimdWidth<const BITS: usize>();
impl<const BITS: usize> SimdWidth<BITS> {
    // The width (in bits) of the simd type, including all lanes.
    pub const BITS: usize = BITS;
}
/// A supported system width (in bits) for the Simd type, e.g. 128, 256, 512, etc.
pub trait SupportedSimdWidth {
    const BITS: usize;
}

macro_rules! impl_supported_simd_widths {
    ($($bits:expr),*) => {
        $(
            impl SupportedSimdWidth for SimdWidth<{ $bits }> {
                const BITS: usize = $bits;
            }
        )*
    }
}
// TODO: 1024+ is not really supported, it's just what happens when you multiply u64*64 (max lane count)
impl_supported_simd_widths! {
    8, 16, 32, 64, 128, 256, 512
}

macro_rules! impl_supported_simds {
    (types: ($($t:ty),*); lanes: $lanes:tt;) => {
        $(impl_supported_simds!{type: $t; lanes: $lanes})*
    };
    (type: $t:ty; lanes: ($($lanes:literal),*)) => {
        $(
            impl SupportedSimd for Simd<$t, $lanes> {
                type Elem = $t;
                type LaneCount = LaneCount<$lanes>;
                type SimdWidth = SimdWidth<{ $lanes * size_of::<$t>() * 8 }>;
            }
        )*
    };
}

// TODO: ignoring Simd<*const T> and Simd<*mut T> for now. We really only wanted u8 anyway :)

impl_supported_simds! {
    types: (u64, i64, f64);
    lanes: (1, 2, 4, 8);
}
impl_supported_simds! {
    types: (u32, i32, f32);
    lanes: (1, 2, 4, 8, 16);
}
impl_supported_simds! {
    types: (u16, i16);
    lanes: (1, 2, 4, 8, 16, 32);
}
impl_supported_simds! {
    types: (u8, i8);
    lanes: (1, 2, 4, 8, 16, 32, 64);
}

cfg_if::cfg_if! {
    if #[cfg(target_pointer_width = "64")] {
        impl_supported_simds! {
            types: (usize, isize);
            lanes: (1, 2, 4, 8);
        }
    } else if #[cfg(target_pointer_width = "32")] {
        impl_supported_simds! {
            types: (usize, isize);
            lanes: (1, 2, 4, 8, 16);
        }
    } else if #[cfg(target_pointer_width = "16")] {
        impl_supported_simds! {
            types: (usize, isize);
            lanes: (1, 2, 4, 8, 16, 32);
        }
    } else {
        compile_error!("Unsupported target_pointer_width");
    }
}

macro_rules! define_simd_module_items {
    (const SIMD_BITS = $simd_bits:tt; $($more:tt)*) => {
        // pub type BitWidth = $crate::simd::bit_width::BitWidth<$simd_bits>;
        pub const SIMD_BITS: usize = $simd_bits;
        pub type PrimitiveMaskSimdU8 = define_simd_module_items!(@primitive_mask $simd_bits);
        pub const SIMD_BYTES: usize = SIMD_BITS / 8;
        pub const SIMD8_LANES: usize = SIMD_BITS / 8;
        pub const SIMD16_LANES: usize = SIMD_BITS / 16;
        pub const SIMD32_LANES: usize = SIMD_BITS / 32;
        pub const SIMD64_LANES: usize = SIMD_BITS / 64;
        pub type SimdU8 = std::simd::Simd<u8, SIMD8_LANES>;
        pub type SimdU16 = std::simd::Simd<u16, SIMD16_LANES>;
        pub type SimdU32 = std::simd::Simd<u32, SIMD32_LANES>;
        pub type SimdU64 = std::simd::Simd<u64, SIMD64_LANES>;
        pub type SimdI8 = std::simd::Simd<i8, SIMD8_LANES>;
        pub type SimdI16 = std::simd::Simd<i16, SIMD16_LANES>;
        pub type SimdI32 = std::simd::Simd<i32, SIMD32_LANES>;
        pub type SimdI64 = std::simd::Simd<i64, SIMD64_LANES>;
        pub type SimdMask8 = std::simd::Mask<<u8 as std::simd::SimdElement>::Mask, SIMD8_LANES>;
        pub type SimdMask16 = std::simd::Mask<<u16 as std::simd::SimdElement>::Mask, SIMD16_LANES>;
        pub type SimdMask32 = std::simd::Mask<<u32 as std::simd::SimdElement>::Mask, SIMD32_LANES>;
        pub type SimdMask64 = std::simd::Mask<<u64 as std::simd::SimdElement>::Mask, SIMD64_LANES>;
        pub const SIMD_PER_64_BYTES: usize = 64 / SIMD_BYTES;

        #[inline(always)]
        pub fn into_simd_chunks(chunk64: [u8; SIMD_BITS]) -> [[[u8; SIMD_BYTES]; SIMD_PER_64_BYTES]; SIMD64_LANES] {
            unsafe { std::mem::transmute(chunk64) }
        }

        #[inline(always)]
        pub fn as_simd_chunks(chunk64: &[u8; SIMD_BITS]) -> &[[[u8; SIMD_BYTES]; SIMD_PER_64_BYTES]; SIMD64_LANES] {
            unsafe { std::mem::transmute(chunk64) }
        }

        define_simd_module_items! { $($more)* }
    };
    (fn splat16(); $($more:tt)*) => {
        #[inline(always)]
        pub const fn splat16(val: std::simd::Simd<u8, 16>) -> SimdU8 {
            #[allow(clippy::missing_transmute_annotations)]
            SimdU8::from_array(unsafe { std::mem::transmute([val.to_array(); SIMD_BYTES / 16]) })
        }
        define_simd_module_items! { $($more:tt)* }
    };
    () => {};
    (@primitive_mask 64) => (u8);
    (@primitive_mask 128) => (u16);
    (@primitive_mask 256) => (u32);
    (@primitive_mask 512) => (u64);
}

// pub mod simd64 {
//     define_simd_module_items! {
//         const SIMD_BITS = 64;
//     }
// }

pub mod simd128 {
    define_simd_module_items! {
        const SIMD_BITS = 128;
        fn splat16();
    }
}

pub mod simd256 {
    define_simd_module_items! {
        const SIMD_BITS = 256;
        fn splat16();
    }
}

pub mod simd512 {
    define_simd_module_items! {
        const SIMD_BITS = 512;
        fn splat16();
    }
}

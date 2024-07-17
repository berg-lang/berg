//
// Perform a "cumulative bitwise xor," flipping bits each time a 1 is encountered.
//
#[inline]
pub fn prefix_xor(bitmask: u64) -> u64 {
    // cfg code pulled from Polars (polars_utils crate)
    #[cfg(all(target_arch = "x86_64", target_feature = "pclmulqdq"))]
    return intel_prefix_xor(bitmask);

    #[cfg(all(
        target_arch = "aarch64",
        target_feature = "neon",
        target_feature = "aes"
    ))]
    return arm_prefix_xor(bitmask);

    #[allow(unreachable_code)]
    fallback_prefix_xor(bitmask)
}

#[allow(dead_code)]
const ALL: u64 = u64::MAX;

#[cfg(all(target_arch = "x86_64", target_feature = "pclmulqdq", target_feature = "sse2"))]
fn intel_prefix_xor(bitmask: u64) -> u64 {
    use core::arch::x86_64::*;
    unsafe {
        let all_ones = _mm_set1_epi64x(ALL as i64);
        let result = _mm_clmulepi64_si128(_mm_set_epi64x(0ULL, bitmask), all_ones, 0);
        _mm_cvtsi128_si64(result) as u64
    }
}

#[cfg(all(
    target_arch = "aarch64",
    target_feature = "neon",
    target_feature = "aes"
))]
fn arm_prefix_xor(bitmask: u64) -> u64 {
    use core::arch::aarch64::*;
    unsafe { vmull_p64(bitmask, ALL) as u64 }
}

#[inline]
fn fallback_prefix_xor(mut bitmask: u64) -> u64 {
    // From simdjson:
    /////////////
    // On ARM we could do this with PMULL, but it is apparently slow.
    //
    //#ifdef __ARM_FEATURE_CRYPTO // some ARM processors lack this extension
    //return vmull_p64(-1ULL, bitmask);
    //#else
    // Analysis by @sebpop:
    // When diffing the assembly for src/stage1_find_marks.cpp I see that the eors are all spread out
    // in between other vector code, so effectively the extra cycles of the sequence do not matter
    // because the GPR units are idle otherwise and the critical path is on the FP side.
    // Also the PMULL requires two extra fmovs: GPR->FP (3 cycles in N1, 5 cycles in A72 )
    // and FP->GPR (2 cycles on N1 and 5 cycles on A72.)
    ///////////
    bitmask ^= bitmask << 1;
    bitmask ^= bitmask << 2;
    bitmask ^= bitmask << 4;
    bitmask ^= bitmask << 8;
    bitmask ^= bitmask << 16;
    bitmask ^= bitmask << 32;
    bitmask
}

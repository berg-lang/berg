#[inline]
pub fn prefix_xor(mut bitmask: u64) -> u64 {
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

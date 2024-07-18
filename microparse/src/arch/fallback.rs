use std::simd::Simd;

// TODO shouldn't need this here, but I want to actually code stuff instead of figure out arch stuff right now
pub type SimdU8 = Simd<u8, 64>;

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

pub fn lookup_lower16_ascii(lookup_table: SimdU8, keys: SimdU8) -> SimdU8 {
    let lookup_table = lookup_table.to_array();
    let mut keys = (keys & SimdU8::splat(0b0000_1111)).to_array();
    for i in 0..64 {
        keys[i] = lookup_table[keys[i] as usize];
    }
    SimdU8::from_array(keys)
}

pub const fn splat16(val: Simd<u8, 16>) -> SimdU8 {
    let val = val.to_array();
    SimdU8::from_array([
        val[0], val[1], val[2], val[3],
        val[4], val[5], val[6], val[7],
        val[8], val[9], val[10], val[11],
        val[12], val[13], val[14], val[15],

        val[0], val[1], val[2], val[3],
        val[4], val[5], val[6], val[7],
        val[8], val[9], val[10], val[11],
        val[12], val[13], val[14], val[15],

        val[0], val[1], val[2], val[3],
        val[4], val[5], val[6], val[7],
        val[8], val[9], val[10], val[11],
        val[12], val[13], val[14], val[15],

        val[0], val[1], val[2], val[3],
        val[4], val[5], val[6], val[7],
        val[8], val[9], val[10], val[11],
        val[12], val[13], val[14], val[15],
    ])
}
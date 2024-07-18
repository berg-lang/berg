pub use super::*;

pub const fn mask<const N: usize>(pos: [isize; N]) -> Mask64 {
    assert!(N < 64);
    let mut mask = 0;
    seq_macro::seq!(i in 0..64 {
        if i < N {
            let pos = pos[i];
            assert!(pos >= -64 && pos < 64);
            let index = if pos >= 0 { pos } else { 64+pos };
            assert!(index < 64);
            mask |= 1 << index;
        }
    });
    mask
}


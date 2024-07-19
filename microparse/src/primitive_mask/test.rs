use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;
use std::simd::cmp::SimdPartialEq;
use std::simd::u8x64;

pub use super::*;
pub use super::fmt::*;

pub fn pad_middle(input: &[u8]) -> [u8; 64] {
    if let Ok(array) = input.try_into() {
        assert!(input.len() == 64);
        return array;
    }
    assert!(input.len() < 64);
    let mut dot = 0;
    while dot+3 < input.len() {
        // Skip ... and put the rest at the end
        if input[dot] == b'.' && input[dot+1] == b'.' && input[dot+2] == b'.' {
            // Pick the char before ... to fill with, unless we're at the beginning
            let pad_char = if dot == 0 {
                assert!(input.len() >= 4);
                input[dot+3]
            } else {
                input[dot-1]
            };
            let mut result = [pad_char; 64];
            let start = &input[0..dot];
            let end = &input[dot+3..];
            result[0..dot].copy_from_slice(start);
            result[input.len()-end.len()..].copy_from_slice(end);
            return result;
        }
        dot += 1;
    }
    panic!("'...' not found in string {:?}", input);
}

pub fn backslashes(input: &[u8; 64]) -> InputMask {
    mask_char(input, b'\\')
}

pub fn non_spaces(input: &[u8; 64]) -> InputMask {
    negative_mask_char(input, b' ')
}

pub fn mask_char(input: &[u8; 64], ch: u8) -> InputMask {
    let mask = u8x64::from_array(*input).simd_eq(u8x64::splat(ch)).to_bitmask();
    InputMask::new(mask, *input)
}

pub fn negative_mask_char(input: &[u8; 64], ch: u8) -> InputMask {
    let result = mask_char(input, ch);
    result.with_mask(!result.mask)
}


pub const fn mask_bits_on<const N: usize>(pos: [isize; N]) -> Mask64 {
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

#[derive(Copy, Clone)]
pub struct InputMask {
    mask: Mask64,
    input: [u8; 64],
}

impl InputMask {
    pub fn new(mask: Mask64, input: [u8; 64]) -> Self {
        Self { mask, input }
    }

    pub fn with_mask(self, mask: Mask64) -> Self {
        Self { input: self.input, mask }
    }
}

impl PartialEq for InputMask {
    fn eq(&self, other: &Self) -> bool {
        self.mask == other.mask
    }
}
impl PartialEq<InputMask> for Mask64 {
    fn eq(&self, other: &InputMask) -> bool {
        *self == other.mask
    }
}

impl Deref for InputMask {
    type Target = Mask64;

    fn deref(&self) -> &Self::Target {
        &self.mask
    }
}

impl From<InputMask> for Mask64 {
    fn from(val: InputMask) -> Self {
        val.mask
    }
}

impl From<InputMask> for [u8; 64] {
    fn from(val: InputMask) -> Self {
        val.input
    }
}

impl AsRef<[u8; 64]> for InputMask {
    fn as_ref(&self) -> &[u8; 64] {
        &self.input
    }
}

impl AsRef<[u8]> for InputMask {
    fn as_ref(&self) -> &[u8] {
        &self.input
    }
}

impl AsRef<Mask64> for InputMask {
    fn as_ref(&self) -> &Mask64 {
        &self.mask
    }
}

impl Display for InputMask {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        Display::fmt(&self.mask.fmt_str(&self.input), f)
    }
}

impl Debug for InputMask {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        Debug::fmt(&self.mask.fmt_str(&self.input), f)
    }
}
use std::fmt::{Debug, Display, Write};
use super::Mask64;

pub trait MaskFmt {
    fn fmt_x(&self) -> MaskFmtChar;
    fn fmt_ch(&self, on: char, off: char) -> MaskFmtChar;
    fn fmt_str<'a>(&self, str: &'a impl AsRef<[u8]>) -> MaskFmtStr<'a>;
    fn fmt_str_custom<'a>(&self, str: &'a impl AsRef<[u8]>, off: char, unprintable: char) -> MaskFmtStr<'a>;
}

impl MaskFmt for Mask64 {
    fn fmt_x(&self) -> MaskFmtChar {
        self.fmt_ch('X', ' ')
    }
    fn fmt_ch(&self, on: char, off: char) -> MaskFmtChar {
        MaskFmtChar { mask: *self, on, off }
    }
    fn fmt_str<'a>(&self, str: &'a impl AsRef<[u8]>) -> MaskFmtStr<'a> {
        self.fmt_str_custom(str, ' ', '_')
    }
    fn fmt_str_custom<'a>(&self, str: &'a impl AsRef<[u8]>, off: char, unprintable: char) -> MaskFmtStr<'a> {
        MaskFmtStr { mask: *self, str: str.as_ref(), off, unprintable }
    }
}

#[derive(Copy, Clone)]
pub struct MaskFmtChar {
    pub mask: Mask64,
    pub on: char,
    pub off: char
}

#[derive(Copy, Clone)]
pub struct MaskFmtStr<'a> {
    pub mask: Mask64,
    pub str: &'a [u8],
    pub off: char,
    pub unprintable: char,
}

impl Display for MaskFmtChar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in 0..64 {
            let is_on = (self.mask & (1 << i)) != 0;
            f.write_char(if is_on { self.on } else { self.off })?;
        }
        Ok(())
    }
}

impl Debug for MaskFmtChar {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl<'a> Display for MaskFmtStr<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for i in 0..64 {
            let is_off = (self.mask & (1 << i)) == 0;
            let ch = if is_off {
                self.off
            } else if self.str[i].is_ascii_control() || !self.str[i].is_ascii() || (self.str[i] as char) == self.off {
                self.unprintable
            } else {
                self.str[i] as char
            };
            f.write_char(ch)?;
        }
        Ok(())
    }
}

impl<'a> Debug for MaskFmtStr<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

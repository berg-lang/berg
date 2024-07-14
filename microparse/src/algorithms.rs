use crate::ByteMask;

#[derive(Default, Debug, Clone)]
pub struct PrecededBy<const N: usize = 1> {
    pub prev_matches: ByteMask,
}

impl<const N: usize> PrecededBy<N> {
    #[inline]
    pub fn next(&mut self, matches: ByteMask) -> ByteMask {
        let result = matches.0 >> N | self.prev_matches.0 << (64-N);
        self.prev_matches = matches;
        ByteMask(result)
    }

    #[inline]
    pub fn any(&self) -> bool {
        self.prev_matches != 0
    }
}

#[derive(Default, Debug, Clone)]
pub struct Escapes {

}

#[derive(Default, Debug, Clone)]
pub struct Strings {

}

pub struct ValidateUtf8 {

}
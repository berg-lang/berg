use ast::IdentifierIndex;
use ast::intern_pool::*;
use ast::operators::Operators::*;

#[derive(Debug,Copy,Clone,PartialEq)]
pub enum Operators {
    Star = 0,
    Slash = 1,
    Plus = 2,
    Dash = 3,
    OpenParen = 4,
    CloseParen = 5,
    Unrecognized = 6,
}

impl Operators {
    pub fn identifier(&self) -> IdentifierIndex {
        IdentifierIndex(*self as u32)
    }
    pub fn string(&self) -> &str {
        match *self {
            Star => "*",
            Slash => "/",
            Plus => "+",
            Dash => "-",
            OpenParen => "(",
            CloseParen => ")",
            Unrecognized => unreachable!(),
        }
    }
    pub(crate) fn intern_all() -> InternPool<IdentifierIndex> {
        let mut identifiers = InternPool::default();
        for i in 0..(Operators::Unrecognized as u32) {
            let operator = Operators::from(IdentifierIndex(i));
            let actual_identifier = identifiers.add(operator.string());
            assert_eq!(actual_identifier, operator.identifier())
        }
        assert_eq!(identifiers.len(), Operators::Unrecognized.identifier());
        identifiers
    }
}

impl From<IdentifierIndex> for Operators {
    fn from(index: IdentifierIndex) -> Self {
        match usize::from(index) as u32 {
            0 => Star,
            1 => Slash,
            2 => Plus,
            3 => Dash,
            4 => OpenParen,
            5 => CloseParen,
            _ => Unrecognized,
        }
    }
}

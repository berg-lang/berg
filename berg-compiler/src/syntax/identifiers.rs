use std::fmt;
use std::num::NonZeroU32;
use std::ops::Range;
use std::u32;
use string_interner::{StringInterner, Symbol};

#[derive(Copy,Clone,PartialEq,Eq,Ord,PartialOrd)]
pub struct IdentifierIndex(NonZeroU32);

macro_rules! identifiers {
    { $($name:ident = $str:expr),* } => {
        const ALL_IDENTIFIERS: [(IdentifierIndex, &str); IdentifierIndices::COUNT as usize] = [
            $( ($name, $str) ),*
        ];
        // We use this enum to determine the indices
        #[allow(non_camel_case_types)]
        enum IdentifierIndices {
            $($name),*,
            COUNT
        }
        $(
            pub const $name: IdentifierIndex = IdentifierIndex(unsafe { NonZeroU32::new_unchecked(IdentifierIndices::$name as u32 + 1) });
        )*
        const IDENTIFIER_RANGE: Range<IdentifierIndex> = Range {
            start: IdentifierIndex(unsafe { NonZeroU32::new_unchecked(1) }),
            end: IdentifierIndex(unsafe { NonZeroU32::new_unchecked(IdentifierIndices::COUNT as u32 + 1) })
        };
    }
}
identifiers! {
    EMPTY_STRING = "",
    STAR = "*",
    SLASH = "/",
    PLUS = "+",
    DASH = "-",
    OPEN_PAREN = "(",
    CLOSE_PAREN = ")",
    OPEN_CURLY = "{",
    CLOSE_CURLY = "}",
    SEMICOLON = ";",
    AND_AND = "&&",
    OR_OR = "||",
    EXCLAMATION_POINT = "!",
    DOUBLE_EXCLAMATION_POINT = "!!",
    EQUAL_TO = "==",
    NOT_EQUAL_TO = "!=",
    GREATER_THAN = ">",
    LESS_THAN = "<",
    GREATER_EQUAL = ">=",
    LESS_EQUAL = "<=",
    PLUS_PLUS = "++",
    DASH_DASH = "--",
    COLON = ":",
    TRUE = "true",
    FALSE = "false",
    FIELDS = "Fields",
    OPERATORS = "Operators",
    PREFIX_OPERATORS = "PrefixOperators",
    SUFFIX_OPERATORS = "SuffixOperators",
    APPLY = "Call",
    NEWLINE = "\n",
    NOTHING = "nothing",
    NEXT = "Next",
    DOT = ".",
    SPACE = " ",
    COMMA = ","
}

pub(crate) fn intern_all() -> StringInterner<IdentifierIndex> {
    let mut identifiers = StringInterner::new();
    for &(operator, string) in ALL_IDENTIFIERS.iter() {
        let actual_identifier = identifiers.get_or_intern(string);
        assert_eq!(actual_identifier, operator);
    }
    assert_eq!(identifiers.len(), ALL_IDENTIFIERS.len());
    identifiers
}

impl IdentifierIndex {
    pub fn well_known_str(self) -> &'static str {
        self.as_str().unwrap()
    }
    pub(crate) fn as_str(self) -> Option<&'static str> {
        if self >= IDENTIFIER_RANGE.start && self < IDENTIFIER_RANGE.end {
            Some(ALL_IDENTIFIERS[self.to_usize()].1)
        } else {
            None
        }
    }
}

impl Symbol for IdentifierIndex {
	/// Creates a `IdentifierIndex` from the given `usize`.
	///
	/// # Panics
	///
	/// If the given `usize` is greater than `u32::MAX - 1`.
	fn from_usize(val: usize) -> Self {
		assert!(val < u32::MAX as usize);
        IdentifierIndex(unsafe { NonZeroU32::new_unchecked((val+1) as u32) })
	}

	fn to_usize(self) -> usize {
		(self.0.get() as usize) - 1
	}
}

impl fmt::Display for IdentifierIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(string) = self.as_str() {
            write!(f, "{}", string)
        } else {
            write!(f, "#{}", self.to_usize())
        }
    }
}
impl fmt::Debug for IdentifierIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(string) = self.as_str() {
            write!(f, "{}", string)
        } else {
            write!(f, "#{}", self.to_usize())
        }
    }
}

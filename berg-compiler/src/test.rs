use crate::eval::{evaluate_ast, RootRef};
use crate::parser;
use crate::syntax::identifiers::*;
use crate::syntax::{AstRef, ByteIndex, ByteRange, LineColumnRange, SourceRef};
use crate::util::from_range::BoundedRange;
use crate::util::from_range::IntoRange;
use crate::util::try_from::TryFrom;
use crate::value::{BergResult, BergVal, BergValue, Error, ErrorCode, EvalResult, NextVal};
use std::fmt;
use std::io;
use std::ops::Range;

pub fn expect<T: AsRef<[u8]> + ?Sized>(source: &T) -> ExpectBerg {
    ExpectBerg(source.as_ref())
}

pub fn expect_bytes(source: &[u8]) -> ExpectBerg {
    ExpectBerg(source)
}

pub fn test_source<'a, Bytes: Into<&'a [u8]>>(source: Bytes) -> SourceRef<'a> {
    SourceRef::memory("test.rs", source.into(), test_root())
}

pub fn test_root() -> RootRef {
    // Steal "source"
    let out: Vec<u8> = vec![];
    let err: Vec<u8> = vec![];
    let root_path = Err(io::Error::new(
        io::ErrorKind::Other,
        "SYSTEM ERROR: no relative path--this error should be impossible to trigger",
    ));
    RootRef::new(root_path, Box::new(out), Box::new(err))
}

pub struct ExpectBerg<'a>(pub &'a [u8]);

impl<'a> fmt::Display for ExpectBerg<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "test '{}'", String::from_utf8_lossy(self.0))
    }
}

pub trait ExpectedErrorRange {
    fn into_error_range(self, len: ByteIndex) -> ByteRange;
}
impl ExpectedErrorRange for usize {
    fn into_error_range(self, len: ByteIndex) -> ByteRange {
        ByteIndex::from(self).into_error_range(len)
    }
}
impl ExpectedErrorRange for ByteIndex {
    #[allow(clippy::range_plus_one)]
    fn into_error_range(self, _len: ByteIndex) -> ByteRange {
        Range {
            start: self,
            end: self + 1,
        }
    }
}
impl<R: BoundedRange<ByteIndex>, T: IntoRange<ByteIndex, Output = R>> ExpectedErrorRange for T {
    fn into_error_range(self, len: ByteIndex) -> ByteRange {
        let result = self.into_range().bounded_range(len);
        assert!(result.start + 1 != result.end);
        result
    }
}

impl<'a> ExpectBerg<'a> {
    fn parse(&self) -> AstRef<'a> {
        let ast = parser::parse(test_source(self.0));
        assert_eq!(
            self.0,
            ast.to_bytes().as_slice(),
            "Round trip failed!\nExpected:\n{}\n---------\nActual:\n{}\n---------\n",
            String::from_utf8_lossy(self.0),
            ast.to_string()
        );
        ast
    }

    pub fn bergvals_equal(expected: BergVal<'a>, actual: BergVal<'a>) -> EvalResult<'a, bool> {
        let result = expected.infix(EQUAL_TO, actual)?;
        result.into_native::<bool>()?
    }

    #[allow(clippy::needless_pass_by_value, clippy::wrong_self_convention)]
    pub fn to_yield<V: Into<BergVal<'a>> + fmt::Display>(self, expected: V)
    where
        BergVal<'a>: From<V>,
    {
        let actual = evaluate_ast(self.parse())
            .unwrap_or_else(|error| panic!("Unexpected error: {}", error));
        let expected = BergVal::from(expected);
        println!("actual: {}, expected: {}", actual, expected);
        assert!(
            Self::bergvals_equal(expected.clone(), actual.clone()).unwrap_or(false),
            "Wrong value returned! expected: {}, actual: {}",
            expected,
            actual
        );
    }

    fn consume_all(value: BergVal<'a>) -> BergResult<'a> {
        let mut values = vec![];
        let mut next = Some(value);
        while let Some(tail) = next {
            if let NextVal(Some((head, tail))) = tail.next_val()? {
                values.push(head);
                next = tail;
            } else {
                next = None;
            }
        }
        Ok(values.into())
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_error(self, code: ErrorCode, expected_range: impl ExpectedErrorRange) {
        let ast = parser::parse(test_source(self.0));
        let expected_range = ast
            .char_data
            .range(&expected_range.into_error_range(ast.char_data.size));
        let result = evaluate_ast(ast.clone());
        let result = result.and_then(Self::consume_all);
        assert!(
            result.is_err(),
            "No error produced by {}: expected {}, got value {}",
            self,
            error_string(code, expected_range),
            result.as_ref().unwrap()
        );
        let actual = Error::try_from(result.unwrap_err());
        assert!(
            actual.is_ok(),
            "Result of {} is an error, but of an unexpected type! Expected {}, got {}",
            self,
            error_string(code, expected_range),
            actual.as_ref().unwrap_err()
        );
        let actual = actual.unwrap();
        assert_eq!(
            code,
            actual.code(),
            "Wrong error code from {}! Expected {}, got {} at {}",
            self,
            error_string(code, expected_range),
            actual.code(),
            actual.location().range()
        );
        assert_eq!(
            expected_range,
            actual.location().range(),
            "Wrong error range from {}! Expected {}, got {} at {}",
            self,
            error_string(code, expected_range),
            actual.code(),
            actual.location().range()
        );
    }
}

fn error_string(code: ErrorCode, range: LineColumnRange) -> String {
    format!("{} at {}", code, range)
}

// struct DisplayResults<'p, 'a: 'p>(&'p Vec<BergResult<'a>>);

// impl<'p, 'a: 'p> fmt::Display for DisplayResults<'p, 'a> {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match self.0.len() {
//             0 => write!(f, "()"),
//             1 => match &self.0[0] { Ok(v) => write!(f, "{}", v), Err(e) => write!(f, "{}", e) },
//             _ => {
//                 match &self.0[0] { Ok(v) => write!(f, "({}", v)?, Err(e) => write!(f, "({}", e)? };
//                 for result in &self.0[1..] {
//                     match result { Ok(v) => write!(f, ",{}", v)?, Err(e) => write!(f, ",{}", e)? }
//                 }
//                 write!(f, ")")
//             }
//         }
//     }
// }

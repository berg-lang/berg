use crate::error::{Error, ErrorCode};
use crate::eval::RootRef;
use crate::parser;
use crate::syntax::{AstRef, ByteIndex, ByteRange, LineColumnRange, SourceRef};
use crate::util::from_range::BoundedRange;
use crate::util::from_range::IntoRange;
use crate::util::try_from::TryFrom;
use crate::util::type_name::TypeName;
use crate::value::BergVal;
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

pub trait ExpectedValue<'a>:
    TypeName + TryFrom<BergVal<'a>, Error = BergVal<'a>> + PartialEq<Self> + fmt::Display + fmt::Debug
{
}

impl<
        'a,
        V: TypeName
            + TryFrom<BergVal<'a>, Error = BergVal<'a>>
            + PartialEq<V>
            + fmt::Display
            + fmt::Debug,
    > ExpectedValue<'a> for V
{
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

    #[allow(clippy::needless_pass_by_value, clippy::wrong_self_convention)]
    pub fn to_yield<V: ExpectedValue<'a> + Clone>(self, expected_value: V) {
        let ast = self.parse();
        let result = ast.result_to::<V>();
        assert!(
            result.is_ok(),
            "Unexpected error {} in {}: expected {}",
            result.unwrap_err(),
            self,
            expected_value
        );
        let value = result.unwrap();
        assert_eq!(
            expected_value, value,
            "Wrong result from {}! Expected {}, got {}",
            self, expected_value, value
        );
    }
    #[allow(clippy::needless_pass_by_value, clippy::wrong_self_convention)]
    pub fn to_yield_tuple<V: ExpectedValue<'a> + Clone>(self, expected_value: &[V])
    where
        Vec<V>: TypeName + TryFrom<BergVal<'a>, Error = BergVal<'a>>,
    {
        let ast = self.parse();
        let result = ast.result_to::<Vec<V>>();

        assert!(
            result.is_ok(),
            "Unexpected error {} in {}: expected [{}]",
            result.unwrap_err(),
            self,
            {
                let i: Vec<String> = expected_value.iter().map(|v| format!("{}", v)).collect();
                i.join(",")
            }
        );
        let value = result.unwrap();
        assert_eq!(
            Vec::from(expected_value),
            value,
            "Wrong result from {}! Expected [{}], got [{}]",
            self,
            {
                let i: Vec<String> = expected_value.iter().map(|v| format!("{}", v)).collect();
                i.join(",")
            },
            {
                let i: Vec<String> = value.iter().map(|v| format!("{}", v)).collect();
                i.join(",")
            }
        );
    }
    #[allow(clippy::wrong_self_convention)]
    pub fn to_error(self, code: ErrorCode, expected_range: impl ExpectedErrorRange) {
        let ast = parser::parse(test_source(self.0));
        let expected_range = ast
            .char_data()
            .range(&expected_range.into_error_range(ast.char_data().size));
        let result = ast.result();
        assert!(
            result.is_err(),
            "No error produced by {}: expected {}, got value {}",
            self,
            error_string(code, expected_range),
            result.as_ref().unwrap()
        );
        let value = Error::try_from(result.unwrap_err());
        assert!(
            value.is_ok(),
            "Result of {} is an error, but of an unexpected type! Expected {}, got {}",
            self,
            error_string(code, expected_range),
            value.as_ref().unwrap_err()
        );
        let error = value.unwrap();
        assert_eq!(
            code,
            error.code(),
            "Wrong error code from {}! Expected {}, got {} at {}",
            self,
            error_string(code, expected_range),
            error.code(),
            error.location().range()
        );
        assert_eq!(
            expected_range,
            error.location().range(),
            "Wrong error range from {}! Expected {}, got {} at {}",
            self,
            error_string(code, expected_range),
            error.code(),
            error.location().range()
        );
    }
}

fn error_string(code: ErrorCode, range: LineColumnRange) -> String {
    format!("{} at {}", code, range)
}

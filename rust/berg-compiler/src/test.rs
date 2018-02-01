use eval::RootRef;
use parser::SourceRef;
use std::fmt;
use std::io;
use std::ops::Range;
use util::try_from::TryFrom;
use util::type_name::TypeName;
use value::{BergErrorStack, BergVal, ErrorCode};

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

impl<'a> ExpectBerg<'a> {
    #[cfg_attr(feature = "clippy", allow(needless_pass_by_value, wrong_self_convention))]
    pub fn to_yield<
        V: TypeName
            + TryFrom<BergVal<'a>, Error = BergVal<'a>>
            + PartialEq<V>
            + fmt::Display
            + fmt::Debug,
    >(
        self,
        expected_value: V,
    ) {
        let source = test_source(self.0);
        let result = source.complete();
        assert!(
            result.is_ok(),
            "Unexpected error {} in {}: expected {}",
            result.unwrap_err(),
            self,
            expected_value
        );
        let value = result.unwrap().downcast::<V>();
        assert!(
            value.is_ok(),
            "Result of {} is the wrong type! Expected {}, got {}",
            self,
            expected_value,
            value.unwrap_err()
        );
        let value = value.unwrap();
        assert_eq!(
            expected_value, value,
            "Wrong result from {}! Expected {}, got {}",
            self, expected_value, value
        );
    }
    #[cfg_attr(feature = "clippy", allow(wrong_self_convention))]
    pub fn to_error(self, code: ErrorCode, range: Range<usize>) {
        let source = test_source(self.0);
        let result = source.complete();
        assert!(
            result.is_err(),
            "No error produced by {}: expected {}, got value {}",
            self,
            error_string(code, range),
            result.as_ref().unwrap()
        );
        let value = BergErrorStack::try_from(result.unwrap_err());
        assert!(
            value.is_ok(),
            "Result of {} is an error, but of an unexpected type! Expected {}, got {}",
            self,
            error_string(code, range),
            value.as_ref().unwrap_err()
        );
        let error = value.unwrap();
        assert_eq!(
            code,
            error.code(),
            "Wrong error code from {}! Expected {}, got {} at {}",
            self,
            error_string(code, range),
            error.code(),
            error.location().range()
        );
        let expected_range = Range {
            start: range.start.into(),
            end: range.end.into(),
        };
        assert_eq!(
            expected_range,
            *error.location().byte_range(),
            "Wrong error range from {}! Expected {}, got {} at {}",
            self,
            error_string(code, range),
            error.code(),
            error.location().range()
        );
    }
}

fn error_string(code: ErrorCode, range: Range<usize>) -> String {
    match range.end - range.start {
        0 => format!("{} at {} (zero width)", code, range.start + 1),
        1 => format!("{} at {}", code, range.start),
        _ => format!("{} at {}-{}", code, range.start, range.end - 1),
    }
}

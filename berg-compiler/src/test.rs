use crate::eval::{evaluate_ast, RootRef};
use crate::parser;
use crate::syntax::identifiers::*;
use crate::syntax::{AstRef, ByteIndex, ByteRange, LineColumnRange, SourceRef};
use crate::util::from_range::BoundedRange;
use crate::util::from_range::IntoRange;
use crate::value::*;
use std::fmt;
use std::io;
use std::ops::Range;
pub use ErrorCode::*;

///
/// Test a string containing Berg source code.
/// 
/// `source` can be anything that can be converted into a string of bytes:
/// String, &str, string literal, or reference to a byte array.
/// 
/// # Examples
/// 
/// ```
/// use berg_compiler::test::*;
/// expect("1 + 1").to_yield(2);
/// expect(&[0x0]).to_error(UnsupportedCharacters, 0);
/// ```
/// 
pub fn expect<T: AsRef<[u8]> + ?Sized>(source: &T) -> ExpectBerg {
    ExpectBerg(source.as_ref())
}

///
/// A Berg test with a fluent interface.
/// 
/// Generally you will call [`expect()`] to create this.
/// 
/// # Examples
/// 
/// ```
/// use berg_compiler::test::*;
/// expect("1 + 1").to_yield(2);
/// expect("1 / 0").to_error(DivideByZero, 4);
/// ```
/// 
pub struct ExpectBerg<'a>(pub &'a [u8]);

///
/// An expected error range.
/// 
/// Anything that implements this can be passed to [`to_error()`]. It is
/// implemented on:
/// * `usize`: `to_error(DivideByZero, 4)` and
/// * ranges:  (so you can write `expect_error(DivideByZero, 1..2))`).
/// 
pub trait ExpectedErrorRange {
    ///
    /// Convert this into an actual ByteRange.
    /// 
    /// `len` will be the length of the actual source (in bytes), and is used
    /// to create an explicit range for unbounded ranges (like `1..`). This
    /// allows `expect("1/(0)").to_error(DivideByZero, 2..)` to match the actual
    /// error range `2..5`.
    /// 
    fn into_error_range(self, len: ByteIndex) -> ByteRange;
}

impl<'a> ExpectBerg<'a> {
    ///
    /// Test that the given value is returned when the Berg source is compiled and run.
    /// 
    /// `expected` can be anything convertible into a `BergVal`, including
    /// numbers and booleans.
    /// 
    /// The [`tuple!`] macro can be used to check for more complex values like
    /// arrays and arrays of arrays.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use berg_compiler::test::*;
    /// expect("1 + 1").to_yield(2);
    /// expect("1 == 2").to_yield(false);
    /// ```
    /// 
    #[allow(clippy::needless_pass_by_value, clippy::wrong_self_convention)]
    pub fn to_yield<V: Into<BergVal<'a>> + fmt::Display>(self, expected: V)
    where
        BergVal<'a>: From<V>,
    {
        println!("Source:");
        println!("{}", String::from_utf8_lossy(self.0));
        println!("");
        let actual = evaluate_ast(self.parse())
            .unwrap_or_else(|error| panic!("Unexpected error: {}", error));
        let expected = BergVal::from(expected);
        println!("actual: {}, expected: {}", actual, expected);
        let equal = bergvals_equal(expected.clone(), actual.clone());
        assert!(
            equal.clone().unwrap_or(false),
            "Wrong value returned! expected: {}, actual: {}. Equal: {}",
            expected,
            actual,
            equal.display()
        );
    }

    ///
    /// Test that an error with the given `code` and location (`expected_range`)
    /// is produced when the Berg source is compiled and run.
    /// 
    /// `expected_range` can be an index or range of indices into the string.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use berg_compiler::test::*;
    /// expect("1 / 0").to_error(DivideByZero, 4);
    /// expect("1 / (0)").to_error(DivideByZero, 4..);
    /// expect("(1+1) += 2").to_error(AssignmentTargetMustBeIdentifier, 0..=4);
    /// ```
    /// 
    /// # Panics
    /// 
    /// * If no error is produced:
    ///   ```should_panic
    ///   use berg_compiler::test::*;
    ///   expect("1 / 1").to_error(DivideByZero, 4);
    ///   ```
    /// * If an error is produced with the wrong code
    ///   ```should_panic
    ///   use berg_compiler::test::*;
    ///   expect("1 / 0").to_error(NoSuchField, 4);
    ///   ```
    /// * If an error is produced with the wrong location
    ///   ```should_panic
    ///   use berg_compiler::test::*;
    ///   expect("1 / 0").to_error(DivideByZero, 2);
    ///   expect("(1 / 0) + 1").to_error(DivideByZero, 4..=6);
    ///   ```
    /// 
    #[allow(clippy::wrong_self_convention)]
    pub fn to_error(self, code: ErrorCode, expected_range: impl ExpectedErrorRange) {
        println!("Source:");
        println!("{}", String::from_utf8_lossy(self.0));
        println!("");
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
            self.error_string(code, expected_range, &ast),
            result.as_ref().unwrap()
        );
        match result.unwrap_err() {
            ErrorVal::Error(actual) => {
                assert_eq!(
                    code,
                    actual.code(),
                    "Wrong error code from {}! Expected {}, got {}",
                    self,
                    self.error_string(code, expected_range, &ast),
                    self.error_string(actual.code(), actual.location().range(), &ast)
                );
                assert_eq!(
                    expected_range,
                    actual.location().range(),
                    "Wrong error range from {}! Expected {}, got {}",
                    self,
                    self.error_string(code, expected_range, &ast),
                    self.error_string(actual.code(), actual.location().range(), &ast)
                );
            },
            actual => panic!(
                "Result of {} is an error, but of an unexpected type! Expected {}, got {}",
                self,
                self.error_string(code, expected_range, &ast),
                actual
            ),
        }
    }

    fn parse(&self) -> AstRef<'a> {
        let ast = parser::parse(test_source(self.0));
        assert_eq!(
            self.0,
            ast.to_bytes().as_slice(),
            "Round trip failed!\nExpected:\n{}\n---------\nActual:\n{}\n---------\nDebug:\n{:?}\n---------",
            String::from_utf8_lossy(self.0),
            ast.to_string(),
            ast.token_ranges
        );
        ast
    }

    fn consume_all(mut value: BergVal<'a>) -> BergResult<'a> {
        let mut values = vec![];
        loop {
            match value.next_val()? {
                None => break,
                Some(NextVal { head, tail }) => {
                    values.push(head);
                    value = tail?;
                }
            }
        }
        Ok(values.into())
    }

    fn error_string(&self, code: ErrorCode, range: LineColumnRange, ast: &AstRef<'a>) -> String {
        let byte_range = ast.char_data.byte_range(range).into_range();
        format!("{} at {} ({})", code, range, String::from_utf8_lossy(&self.0[byte_range]))
    }
}

fn test_source<'a, Bytes: Into<&'a [u8]>>(source: Bytes) -> SourceRef<'a> {
    SourceRef::memory("test.rs", source.into(), test_root())
}

fn test_root() -> RootRef {
    // Steal "source"
    let out: Vec<u8> = vec![];
    let err: Vec<u8> = vec![];
    let root_path = Err(io::Error::new(
        io::ErrorKind::Other,
        "SYSTEM ERROR: no relative path--this error should be impossible to trigger",
    ));
    RootRef::new(root_path, Box::new(out), Box::new(err))
}

impl<'a> fmt::Display for ExpectBerg<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "test '{}'", String::from_utf8_lossy(self.0))
    }
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

///
/// Check if two BergVals are equal. Just calls `infix`.
/// 
fn bergvals_equal<'a>(expected: BergVal<'a>, actual: BergVal<'a>) -> Result<bool, ErrorVal<'a>> {
    Ok(expected.infix(EQUAL_TO, actual.into())?.into_native::<bool>()?)
}

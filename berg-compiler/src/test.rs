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
pub use CompilerErrorCode::*;

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
#[derive(Debug)]
pub struct ExpectBerg<'a>(pub &'a [u8]);

///
/// An expected value.
///
pub trait ExpectedValue<'a>: fmt::Display + Clone {
    fn matches(self, actual: BergVal<'a>) -> Result<bool, EvalException<'a>>;
}
impl<'a> ExpectedValue<'a> for CompilerErrorCode {
    fn matches(self, actual: BergVal<'a>) -> Result<bool, EvalException<'a>> {
        Ok(self == actual.into_native::<CompilerError>()?.code())
    }
}
impl<'a, T: Into<BergVal<'a>> + fmt::Display + Clone> ExpectedValue<'a> for T {
    fn matches(self, actual: BergVal<'a>) -> Result<bool, EvalException<'a>> {
        self.into()
            .infix(EQUAL_TO, actual.into())?
            .into_native::<bool>()
    }
}

///
/// An expected error range.
///
/// Anything that implements this can be passed to [`to_error()`]. It is
/// implemented on:
/// * `usize`: `to_error(DivideByZero, 4)` and
/// * ranges:  (so you can write `expect_error(DivideByZero, 1..2))`).
/// * &str and &[u8]: these will find the first instance of the substring in the
///   source string, so you don't have to character count.
///
pub trait ExpectedErrorRange: fmt::Debug {
    ///
    /// Convert this into an actual ByteRange.
    ///
    /// `source` is the original source, and is used to calculate the end of
    /// unbounded ranges and to find substrings.
    ///
    fn into_error_range(self, source: &[u8]) -> ByteRange;
    ///
    /// Find this error range within a particular line
    ///
    fn line(self, line: usize) -> ExpectedErrorRangeWithin<Self, ExpectLine>
    where
        Self: Sized,
    {
        self.within(ExpectLine(line))
    }
    ///
    /// Find this error range after the end of the other one.
    ///
    fn after<T: ExpectedErrorRange>(
        self,
        after: T,
    ) -> ExpectedErrorRangeWithin<Self, ExpectedErrorRangeAfter<T>>
    where
        Self: Sized,
        T: Sized,
    {
        self.within(ExpectedErrorRangeAfter(after))
    }
    ///
    /// Find this error range before the start of the other one.
    ///
    fn before<T: ExpectedErrorRange>(
        self,
        before: T,
    ) -> ExpectedErrorRangeWithin<Self, ExpectedErrorRangeBefore<T>>
    where
        Self: Sized,
        T: Sized,
    {
        self.within(ExpectedErrorRangeBefore(before))
    }
    ///
    /// Find this error range inside the other one.
    ///
    fn within<T: ExpectedErrorRange>(self, within: T) -> ExpectedErrorRangeWithin<Self, T>
    where
        Self: Sized,
        T: Sized,
    {
        ExpectedErrorRangeWithin {
            error_range: self,
            within,
        }
    }
}

///
/// Finds an error range within another range.
///
/// Used by [`ExpectedErrorRange::after()`].
///
#[derive(Debug)]
pub struct ExpectedErrorRangeWithin<T: ExpectedErrorRange, Within: ExpectedErrorRange> {
    error_range: T,
    within: Within,
}

///
/// Selects all source before the given range.
///
/// Used by [`ExpectedErrorRange::after()`].
///
#[derive(Debug)]
pub struct ExpectedErrorRangeAfter<T: ExpectedErrorRange>(T);

///
/// Selects all source after the given range.
///
#[derive(Debug)]
pub struct ExpectedErrorRangeBefore<T: ExpectedErrorRange>(T);

///
/// Represents the starting point of the line with the given number (starting at 1).
///
/// Used by [`ExpectedErrorRange::line()`].
///
#[derive(Debug)]
pub struct ExpectLine(usize);

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
    pub fn to_yield(self, expected: impl ExpectedValue<'a>) {
        println!("Source:");
        println!("{}", String::from_utf8_lossy(self.0));
        println!();
        let actual = evaluate_ast(self.parse())
            .and_then(Self::evaluate_all)
            .unwrap_or_else(|e| panic!("Unexpected error from {}: {}", self, e));
        println!("actual: {}, expected: {}", actual, expected);
        assert!(
            expected
                .clone()
                .matches(actual.clone())
                .unwrap_or_else(|e| panic!("Unexpected error from {}: {}", self, e)),
            "Wrong value returned from {}! expected: {}, actual: {}.",
            self,
            expected,
            actual
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
    pub fn to_error(
        self,
        expected_value: impl ExpectedValue<'a>,
        expected_range: impl ExpectedErrorRange,
    ) {
        // Run the Berg
        println!("Source:");
        println!("{}", String::from_utf8_lossy(self.0));
        println!();
        let ast = parser::parse(test_source(self.0));
        let expected_range = ast
            .char_data
            .range(&expected_range.into_error_range(self.0.as_ref()));
        let result = evaluate_ast(ast.clone());
        let result = result.and_then(Self::evaluate_all);
        assert!(
            result.is_err(),
            "No error produced by {}: expected {}, got value {}",
            self,
            self.error_string(&expected_value, expected_range, &ast),
            result.as_ref().unwrap()
        );

        let actual = result.unwrap_err();
        let actual_range = actual.location().range();
        assert!(
            expected_value
                .clone()
                .matches(actual.value.clone())
                .unwrap_or_else(|e| panic!("Unexpected error: {}", e)),
            "Wrong error returned from {}! expected: {}, actual: {}.",
            self,
            self.error_string(&expected_value, expected_range, &ast),
            self.error_string(&actual.value, actual_range, &ast)
        );
        assert_eq!(
            expected_range,
            actual_range,
            "Wrong error range from {}! Expected {}, got {}",
            self,
            self.error_string(&expected_value, expected_range, &ast),
            self.error_string(&actual.value, actual_range, &ast)
        )
    }

    fn parse(&self) -> AstRef<'a> {
        let ast = parser::parse(test_source(self.0));
        assert_eq!(
            self.0,
            ast.to_bytes().as_slice(),
            "Round trip failed!\nExpected:\n{}\n---------\nActual:\n{}\n---------\nDebug:\n{:?}\n---------",
            String::from_utf8_lossy(self.0),
            *ast,
            ast.token_ranges
        );
        ast
    }

    fn evaluate_all(value: BergVal<'a>) -> Result<BergVal<'a>, Exception<'a>> {
        let mut value = value.evaluate()?;
        if value.is_single_primitive() {
            return value.ok();
        }

        let mut values = vec![];
        loop {
            use EvalException::*;
            let NextVal { head, tail } = value
                .next_val()
                // Strip the EvalExceptions (which absolutely will not be happening
                // because the errors are being thrown by a BlockVal).
                // TODO this makes it clear that the external interface needs work:
                // If we truly expect this not to throw "local" errors, which I think
                // we do, then we need a result type that only returns Exception.
                .map_err(|e| match e {
                    Error(e) => e,
                    Thrown(..) => unreachable!(),
                })?;

            match head {
                None => break,
                Some(mut head) => {
                    // Consume recursively
                    head = Self::evaluate_all(head)?;
                    values.push(head);
                    value = tail;
                }
            }
        }
        Ok(values.into())
    }

    fn error_string(
        &self,
        value: &dyn fmt::Display,
        range: LineColumnRange,
        ast: &AstRef<'a>,
    ) -> String {
        format!("{} at {}", value, self.error_range_string(range, ast))
    }
    fn error_range_string(&self, range: LineColumnRange, ast: &AstRef<'a>) -> String {
        let byte_range = ast.char_data.byte_range(range).into_range();
        format!(
            "{} ({})",
            range,
            String::from_utf8_lossy(&self.0[byte_range])
        )
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

impl ExpectedErrorRange for &str {
    fn into_error_range(self, source: &[u8]) -> ByteRange {
        find_substring_in_source(self.as_ref(), source)
    }
}
impl ExpectedErrorRange for &[u8] {
    fn into_error_range(self, source: &[u8]) -> ByteRange {
        find_substring_in_source(self, source)
    }
}
impl ExpectedErrorRange for usize {
    fn into_error_range(self, source: &[u8]) -> ByteRange {
        ByteIndex::from(self).into_error_range(source)
    }
}
impl ExpectedErrorRange for ByteIndex {
    #[allow(clippy::range_plus_one)]
    fn into_error_range(self, _source: &[u8]) -> ByteRange {
        Range {
            start: self,
            end: self + 1,
        }
    }
}
impl ExpectedErrorRange for ExpectLine {
    fn into_error_range(self, source: &[u8]) -> ByteRange {
        let start = find_line_start(self.0, source);
        let end = next_line_start(&source[start..]);
        start.into()..(start + end).into()
    }
}
impl<T: ExpectedErrorRange, Within: ExpectedErrorRange> ExpectedErrorRange
    for ExpectedErrorRangeWithin<T, Within>
{
    fn into_error_range(self, source: &[u8]) -> ByteRange {
        let within_range = self.within.into_error_range(source);
        println!("within {:?}", within_range);
        let mut range = self
            .error_range
            .into_error_range(&source[within_range.start.into()..within_range.end.into()]);
        println!("range {:?}", range);
        range.start += usize::from(within_range.start);
        range.end += usize::from(within_range.start);
        println!("result {:?}", range);
        range
    }
}
impl<T: ExpectedErrorRange> ExpectedErrorRange for ExpectedErrorRangeBefore<T> {
    fn into_error_range(self, source: &[u8]) -> ByteRange {
        let before_range = self.0.into_error_range(source);
        ByteIndex(0)..before_range.start
    }
}
impl<T: ExpectedErrorRange> ExpectedErrorRange for ExpectedErrorRangeAfter<T> {
    fn into_error_range(self, source: &[u8]) -> ByteRange {
        let after_range = self.0.into_error_range(source);
        after_range.end..ByteIndex::from(source.len())
    }
}

fn next_line_start(source: &[u8]) -> usize {
    for i in 0..source.len() {
        // If we find \n, \r or \r\n, return the position after it.
        if source[i] == b'\n' {
            return i + 1;
        } else if source[i] == b'\r' {
            if let Some(b'\n') = source.get(i + 1) {
                return i + 2;
            } else {
                return i + 1;
            }
        }
    }
    source.len()
}
fn find_line_start(line: usize, source: &[u8]) -> usize {
    let mut current_line = 1;
    let mut current_line_start = 0;
    while current_line < line {
        current_line_start += next_line_start(&source[current_line_start..]);
        current_line += 1;
    }
    current_line_start
}
impl<R: BoundedRange<ByteIndex>, T: IntoRange<ByteIndex, Output = R> + fmt::Debug>
    ExpectedErrorRange for T
{
    fn into_error_range(self, source: &[u8]) -> ByteRange {
        let result = self.into_range().bounded_range(source.len().into());
        assert!(result.start + 1 != result.end);
        result
    }
}

fn find_substring_in_source(substring: &[u8], source: &[u8]) -> ByteRange {
    let start = source
        .windows(substring.len())
        .position(|window| window == substring)
        .unwrap();
    ByteIndex::from(start)..ByteIndex::from(start + substring.len())
}

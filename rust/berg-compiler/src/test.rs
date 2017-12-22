use source::parse_result::ByteIndex;
use source::parse_result::ByteRange;
use source::compile_errors::CompileError;
use interpreter::value::Value;
use source::compile_errors::CompileErrorLocation;
use std::fmt::Formatter;
use std::fmt::Display;
use source::compile_errors::CompileErrorCode;
use interpreter::value::Errors;
use compiler::Compiler;
use std::ops::Range;
use util::display_arg::DisplayContext;
use std::fmt;
use std::mem;

pub fn expect<T: Into<Vec<u8>>>(source: T) -> ExpectBerg {
    ExpectBerg::new(source)
}

pub struct ExpectBerg {
    source: Vec<u8>,
    expected_value: Value,
    expected_errors: Vec<ExpectedError>,
    expected_warnings: Vec<ExpectedError>,
}

impl ExpectBerg {
    pub fn new<T: Into<Vec<u8>>>(source: T) -> Self {
        ExpectBerg { source: source.into(), expected_value: Value::Nothing, expected_errors: vec![], expected_warnings: vec![] }
    }
    #[allow(wrong_self_convention)]
    pub fn to_yield<V: Into<Value>>(mut self, value: V) -> Self {
        assert!(self.expected_value == Value::Nothing); // Can only set it once
        self.expected_value = value.into();
        self
    }
    pub fn and_yield<V: Into<Value>>(self, value: V) -> Self {
        self.to_yield(value)
    }
    #[allow(wrong_self_convention)]
    pub fn to_error<L: Into<ExpectedLocation>>(mut self, code: CompileErrorCode, location: L) -> Self {
        self.expected_errors.push(ExpectedError { code, location: location.into() });
        self
    }
    pub fn and_error<L: Into<ExpectedLocation>>(self, code: CompileErrorCode, location: L) -> Self {
        self.to_error(code, location)
    }
    #[allow(wrong_self_convention)]
    pub fn to_warn<L: Into<ExpectedLocation>>(mut self, code: CompileErrorCode, location: L) -> Self {
        self.expected_warnings.push(ExpectedError { code, location: location.into() });
        self
    }
    pub fn and_warn<L: Into<ExpectedLocation>>(self, code: CompileErrorCode, location: L) -> Self {
        self.to_warn(code, location)
    }

    pub fn run(mut self) {
        // Steal "source"
        let mut source: Vec<u8> = vec![];
        mem::swap(&mut self.source, &mut source);
        let out: Vec<u8> = vec![];
        let err: Vec<u8> = vec![];
        let mut compiler = Compiler::new(Err("ERROR: no relative path--this error should be impossible to trigger".into()), Box::new(out), Box::new(err));
        let source = compiler.add_memory_source("[test expr]".into(), source);
        let value = compiler.run(source);
        println!("RESULT: {}\n", value.disp(&compiler));
        match value {
            Value::Errors(Errors { errors, value }) => self.test_result(&value, errors, &compiler),
            value => self.test_result(&value, vec![], &compiler),
        }
    }
    fn test_result(mut self, value: &Value, mut errors: Vec<Box<CompileError>>, compiler: &Compiler) {
        let mut messages = vec![];

        // Compare values
        if self.expected_value != *value {
            messages.push(format!("Incorrect value! Expected {}, got {}", self.expected_value.disp(compiler), value.disp(compiler)));
        }
        
        // Compare errors
        while let Some(error) = errors.pop() {
            match self.expected_errors.iter().position(|expected_error| expected_error.matches(error.as_ref(), compiler)) {
                Some(index) => { self.expected_errors.remove(index); },
                None => messages.push(format!("Unexpected error produced: {}", error.disp(compiler))),
            }
        }
        while let Some(expected_error) = self.expected_errors.pop() {
            messages.push(format!("Expected error not produced: {}", expected_error));
        }

        assert!(messages.is_empty(), format!("{}", messages.iter().fold(String::new(), |prev,message| prev+message+"\n")));
    }
}

pub struct ExpectedError { pub code: CompileErrorCode, pub location: ExpectedLocation }

impl ExpectedError {
    pub fn matches(&self, error: &CompileError, compiler: &Compiler) -> bool {
        self.code == error.code() && self.location.matches(compiler, &error.message(compiler).location)
    }
}

impl Display for ExpectedError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{} at {}", self.code.to_string(), self.location)
    }
}

pub struct ExpectedLocation(pub ByteRange);

impl From<ByteRange> for ExpectedLocation { fn from(range: ByteRange) -> Self { ExpectedLocation(range) } }
impl From<Range<u32>> for ExpectedLocation { fn from(range: Range<u32>) -> Self { ByteRange { start: ByteIndex(range.start), end: ByteIndex(range.end) }.into() } }
impl From<u32> for ExpectedLocation { fn from(loc: u32) -> Self { (loc..(loc+1)).into() } }

impl ExpectedLocation {
    pub fn matches(&self, compiler: &Compiler, location: &CompileErrorLocation) -> bool {
        match *location {
            CompileErrorLocation::SourceRange(ref range) => {
                let range = range.range(compiler);
                range.start == self.0.start && range.end == self.0.end
            },
            _ => false,
        }
    }
}

impl Display for ExpectedLocation {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if self.0.end - self.0.start == 1 {
            write!(f, "{}", self.0.start)
        } else {
            write!(f, "{}..{}", self.0.start, self.0.end)
        }
    }
}

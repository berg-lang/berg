use berg::*;
use compile_errors::*;
use std::fs::File;
use std::io;
use std::io::{BufReader,CharsError,ErrorKind,Read};
use std::ops::Range;
use std::path::PathBuf;
use std::str;
use std::str::*;

struct Scanner<'a, Buf: StreamBuffer> {
    stream: &mut 'a Buf;
    context: &mut 'a ParseContext;
    index: ByteIndex;
}
impl<'a> Scanner<'a, Buf: StreamBuffer> {
    pub fn new(stream: &'a mut StreamBuffer, context: &'a mut ParseContext) -> Self {
        Self { stream, context, index: 0 }
    }
    pub fn index(&self) -> ByteIndex {
        self.index
    }
    pub fn consume(&mut self, size: usize) -> Box<[u8]> {
        self.index += size;
        self.stream.consume(size)
    }
    pub fn discard(&mut self, size: usize) {
        self.index += size;
        self.stream.discard(size)
    }

    fn current_buffer(&self) -> &'a [u8] {
        self.stream.current_buffer()
    }
    fn fill_buffer(&mut self, min: usize=0, max: usize=READ_SIZE) -> Result<&'a [u8], ())> {
        match self.stream.fill_buffer() {
            Ok(result) => result,
            Err(error) if error.Kind == ErrorKind::UnexpectedEof => Err(),
            Err(error) => { self.report(IoReadError(error)); Err(()) },
        }
    }

    fn scan(&'a mut Self) -> &'a mut ScannerThread {
        ScannerThread::new(self)
    }
}

struct ScannerThread<'a, Buf: StreamBuffer> {
    scanner: &'a mut Scanner<'a, Buf>;
    accepted: usize;
}

impl<'a: StreamBuffer> ScannerThread<'a, Buf> {
    pub fn new(scanner: &'a mut Scanner) -> Self {
        Self { scanner, accepted: 0}
    }
    pub fn accept(mut self, size: usize=1) {
        self.accepted += 1;
    }
    pub fn token(mut self, type: TokenType) -> TokenSpan {
        let start = self.scanner.index();
        let bytes = self.scanner.consume(accepted);
        let end = self.scanner.index();
        let string = unsafe { String::from_utf8_unchecked(bytes) };
        TokenSpan::new(type, string, start..end);
    }
    pub fn discard(mut self) {
        self.scanner.discard()
    }

    fn consume_utf8(&mut self) {

    }
    // pub fn error_token(error_type: CompileError, type: TokenType) {

    // }
    // pub fn error(error_type: CompileError)
}

impl<'a, Buf: StreamBuffer> Index<usize,Output=Result<u8, ()>> for ScannerThread<'a, Buf> {
    fn index(&self, index: usize) -> Self::Output { self.fill_buffer(index+1)?[index] }
}
impl<'a, R: RangeFull<usize>> Index<R,Output=u8> for ScannerThread<'a> {
    fn index(&self, range: R) -> u8 { self.current_buffer() }
}
impl<'a, R: Range<usize>> Index<R,Output=Result<&'a [u8], ()>> for ScannerThread<'a> {
    fn index(&self, range: R) -> Self::Output { self.fill_buffer(range.end)?[range] }
}
impl<'a, R: RangeFrom<usize>> Index<R,Output=Result<&'a u8, ()>> for ScannerThread<'a> {
    fn index(&self, range: R) -> Self::Output { self.fill_buffer(range.start)?[range] }
}
impl<'a, R: RangeTo<usize>> Index<R,Output=Result<&'a [u8], ()>> for ScannerThread<'a> {
    fn index(&self, range: R) -> Self::Output { self.fill_buffer(range.end)?[range] }
}
impl<'a, R: RangeInclusive<usize>> Index<R,Output=Result<u8, ()>> for IoStreamBuffer<'a> {
    fn index(&self, range: R) -> Self::Output { self.fill_buffer(range.end+1)?[range] }
}
impl<'a, R: RangeToInclusive<usize>> Index<R,Output=Result<u8, ()>> for IoStreamBuffer<'a> {
    fn index(&self, range: R) -> Self::Output { self.fill_buffer(range.end+1)?[range] }
}

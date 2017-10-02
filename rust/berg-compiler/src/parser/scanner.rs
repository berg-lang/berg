use parser::Parser;
use parser::results::*;
use parser::stream_buffer::*;

use std::ops::Range;
use std::ops::RangeInclusive;

pub struct Scanner<'s, 'p: 's, 'c: 'p, Buf: StreamBuffer + 'p> {
    parser: &'s mut Parser<'p, 'c, Buf>,
    accepted: ByteIndex,
}

impl<'s, 'p: 's, 'c: 'p, Buf: StreamBuffer> Scanner<'s, 'p, 'c, Buf> {
    pub fn new(parser: &'s mut Parser<'p, 'c, Buf>) -> Self {
        Scanner { parser, accepted: 0 }
    }

    pub fn start(&self) -> ByteIndex { self.parser.stream.offset() }
    pub fn index(&self) -> ByteIndex { self.start() + self.accepted }
    pub fn accept(&mut self, size: usize) { self.accepted += size; }
    pub fn reset(&mut self) { self.accepted = 0; }
    pub fn buffer<'b>(&'b self) -> &'b [u8] { self.parser.stream.current_buffer() }

    pub fn term(mut self, expression_type: SyntaxExpressionType) {
        let start = self.start();
        let bytes = self.parser.stream.consume(self.accepted);
        let string = unsafe { String::from_utf8_unchecked(bytes) };
        let expression = SyntaxExpression::new(expression_type, start, string);
        self.parser.expressions.push(expression);
        self.reset();
    }
    pub fn error(mut self, error_type: ErrorType) {
        let string = unsafe { String::from_utf8_unchecked(self.parser.stream.consume(self.accepted)) };
        let error = error_type.at(self.parser.source, self.start(), &string);
        self.parser.report(error);
        self.reset();
    }
    pub fn error_invalid(mut self, error_type: ErrorType) {
        let bytes = self.parser.stream.consume(self.accepted);
        let error = error_type.invalid(self.parser.source, self.start(), bytes);
        self.parser.report(error);
        self.reset();
    }
    pub fn discard(mut self) {
        self.parser.stream.discard(self.accepted);
        self.reset();
    }
    pub fn mark_newline(self) {
        let index = self.index();
        self.parser.char_data.append_line(index);
        self.discard();
    }

    pub fn peek(&mut self, index: ByteIndex) -> Result<u8, ()> {
        self.fill_buffer(index+1)?;
        Ok(self.buffer()[index])
    }
    pub fn fill_buffer(&mut self, min: usize) -> Result<(), ()> {
        let err = self.parser.stream.fill_buffer(self.accepted + min).err();
        if let Some(error) = err {
            let compile_error = IoReadError.io_read(self.parser.source, self.index(),error);
            self.parser.report(compile_error);
            Err(())
        } else {
            Ok(())
        }
    }

    pub fn accept_byte(&mut self) -> Result<u8,()> {
        let accepted = self.accepted;
        let byte = self.peek(accepted)?;
        self.accept(1);
        Ok(byte)
    }
    pub fn one<A: Accept>(&mut self, accept: A) -> bool {
        let accepted = self.accepted;
        if let Ok(byte) = self.peek(accepted) {
            if accept.accept(byte) {
                self.accept(1);
                true
            } else {
                false
            }
        } else {
            false
        }
    }
    pub fn many<A: Accept>(&mut self, accept: A) -> bool {
        let byte_result = { self.peek(0) };
        if let Ok(byte) = byte_result {
            if accept.accept(byte) {
                let byte_result = self.peek(0);
                while let Ok(byte) = byte_result {
                    if !accept.accept(byte) {
                        break;
                    }
                }
                true
            } else {
                false
            }
        } else {
            false
        }
    }
    pub fn exactly<A: Accept>(&mut self, n: usize, accept: A) -> bool {
        if self.fill_buffer(n).is_err() { return false; }
        self.buffer().iter().take(5).all(|byte| accept.accept(*byte));
        true
    }
}

pub trait Accept {
    fn accept(&self, byte: u8) -> bool;
}
impl<A: Accept, B: Accept> Accept for (A, B) {
    fn accept(&self, byte: u8) -> bool {
        let (ref a, ref b) = *self;
        a.accept(byte) || b.accept(byte)
    }
}
impl<A: Accept, B: Accept, C: Accept> Accept for (A, B, C) {
    fn accept(&self, byte: u8) -> bool {
        let (ref a, ref b, ref c) = *self;
        a.accept(byte) || b.accept(byte) || c.accept(byte)
    }
}
impl Accept for u8 {
    fn accept(&self, byte: u8) -> bool {
        *self == byte
    }
}
impl Accept for Range<u8> {
    fn accept(&self, byte: u8) -> bool {
        self.contains(byte)
    }
}
impl Accept for RangeInclusive<u8> {
    fn accept(&self, byte: u8) -> bool {
        self.contains(byte)
    }
}

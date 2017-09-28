use berg::*;
use source_reader::*;
use tokenizer::*;
use grammar::*;
use char_definitions::ascii::*;

/// Shared parsing state
pub struct Parser<'a, Buf: StreamBuffer + 'a> {
    scanner: Scanner<'a, Buf>,
    expressions: Vec<SyntaxExpression>,
}

pub struct ParseContext<'a> {
    berg: &'a Berg,
    source: &'a Source,
    errors: CompileErrors,
}

impl ParseContext {
    pub fn new(berg: &'a Berg, source: &'a Source) -> Self {
        let errors = CompileErrors::new();
        Self { berg, source, errors }
    }
}

impl<'a, Buf: StreamBuffer + 'a> Parser<'a, Buf> {
    pub fn new(buffer: &'a mut Buf, context: ParseContext) -> Parser<'a, Buf> {
        let scanner = Scanner::new(buffer, &context);
        let expressions = vec![];
        Parser { scanner, expressions }
    }
    pub fn parse(mut self) -> Vec<SyntaxExpression> {
        while self.step() {};
        self.close()
    }

    const 

    fn step(&mut self) -> bool {
        let scan = &mut self.scanner.scan();
        let ch = scan[0]?;
        scan.accept();
        match ch {
            '0'...'9' => {
                scan.accept_all('0'...'9');
                scan.token(IntegerLiteral);
            },
            // ASCII
            0..UTF8_CONT    => scan.error(UnsupportedCharacter),
            // 2-byte UTF-8
            UTF8_2..UTF8_3   => {
                if scan.accept(UTF8_CONT) {
                    scan.error(UnsupportedCharacter);
                } else {
                    scan.error(InvalidUtf8);
                }
            },
            // 3-byte UTF-8
            UTF8_3..UTF8_4   => {
                if scan.accept(UTF8_CONT) &&
                   scan.accept(UTF8_CONT) {
                    scan.error(UnsupportedCharacter)
                } else {
                    scan.error(InvalidUtf8)
                }
            },
            // 4-byte UTF-8
            UTF8_4..UTF8_INVALID   => {
                if scan.accept(UTF8_CONT..UTF8_2) &&
                   scan.accept(UTF8_CONT..UTF8_2) &&
                   scan.accept(UTF8_CONT..UTF8_2) {
                    scan.error(UnsupportedCharacter)
                } else {
                    scan.error(InvalidUtf8)
                }
            },
            // Invalid UTF-8
            _ => {
                scan.accept_all(UTF8_CONT..UTF8_2, UTF8_INVALID..0xFFu8);
                scan.error(InvalidUtf8)
            }
        }
    }

    fn close(self) -> Vec<SyntaxExpression> {
        self.tokenizer.close()
    }

    // Start of a UTF-8 continuation byte
    const UTF8_CONT    = 0b10000000u8;
    // Start of a UTF-8 2-byte leading byte
    const UTF8_2       = 0b11000000u8;
    // Start of a UTF-8 3-byte leading byte
    const UTF8_3       = 0b11100000u8;
    // Start of a UTF-8 3-byte leading byte
    const UTF8_4       = 0b11110000u8;
    // Invalid UTF-8 bytes from here to 256. Can never occur.
    const UTF8_INVALID = 0b11111000u8;
}




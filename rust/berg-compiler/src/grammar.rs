use CompileError::*;

    macro_rules! lex {
        ()
            => ;

        (@match $($pattern:expr => $arm:stmt),+ | $($post:tt)+ ) => @match $($pattern => $arm),+ , @pattern($pattern:expr => $arm:stmt));
        (@match $condition:expr, => token($token:ident) $($post:tt)* ) => 

        (@match $condition:expr, => error($name:ident) $($post:tt)* ) => 
        (@match $condition:expr, => error($name:ident, $token:ident) $($post:tt)* ) => 
        (@match $condition:expr, $($post:tt)*)
        ($if:expr => token($token:ident))
            => if $if {};
        ($if:expr => error($error:ident, $token:ident));
        ($if:expr) => $if;


        ( !$e:tt $($post:tt)* )
            => !lex!($e) { lex!(@if !lex!($e), $($post)* ) ;
        ( ( ))
        ( ( $($e:tt)+ ) $($post:tt)* )
            => lex!(@if lex!( $($e)+ ), $($post)* );
        ( $a:tt-$b:tt   $($post:tt)* )
            => lex!(@if ch >= lex_byte!($a) && ch <= lex_byte!($b), $($post)*);
        ( $a:tt...$b:tt $($post:tt)* )
            => lex!(@if ch >= lex_byte!($a) && ch <= lex_byte!($b), $($post)*);
        ( $a:tt..$b:tt  $($post:tt)* )
            => lex!(@if ch >= lex_byte!($a) && ch < lex_byte!($b),  $($post)*);
        ( $e:tt *       $($post:tt)* )
            => lex!(@if { while lex!($e) {}; true },                $($post)*);
        ( $e:tt +       $($post:tt)* )
            => lex!(@if lex!($e $e*),                               $($post)* );
        ( _ $(post:tt)+ )
            => lex!($($post)*);
        ( $e:ident $($post:tt)* )
            => lex!(@if self.$name(i, ch), $($post)* );
    }

    lexers!(
        Integer = ('0'...'9')+;
     
        ValidUtf8 = Ascii
                  | Utf8Leading2 Utf8Continue
                  | Utf8Leading3 Utf8Continue Utf8Continue
                  | Utf8Leading4 Utf8Continue Utf8Continue Utf8Continue
     
        InvalidUtf8Byte = Utf8InvalidByte | Utf8Continue
     
        Term  = Integer => token(IntegerLiteral))
              | ValidUtf8Char => error(ValidUtf8)
              | (InvalidUtf8 | Utf8Continue)+ => error(InvalidUtf8);
     
        Expression = Term+;
    )

pub struct MatchThread {
    index: usize,
    MatchState: MatchState,
}

pub struct MatchState {
    start: usize,
}

pub fn 
pub fn match_term(stream: &StreamBuffer) {
    let buffer = stream.buffer()
}
macro_rules! byte_matcher {
    ($(begin:tt)...$(end:tt)) => ch >= $begin && ch < $end
    ($(begin:tt)..$(end:tt)) => ch >= $begin && ch <= $end
    ($(single:tt)) => ch == $single;
}
macro_rules! bytes {
    ( ( $(name:ident) = $(e:expr)|+ );* ) => $(pub fn $name = $(byte_matcher!($e))||*;);* ;
}
bytes!(
    Supported = Ident|Punctuation;

    Ident = IdentStart|Digit;
    IdentStart = Alpha|Underscore;
    Alpha = 'a'..'z'|'A'..'Z';

    Digit = '0'..'9';
    BinaryDigit = '0'..'1';
    OctalDigit = '0'..'7';
    HexDigit = Digit | 'A'..'F' | 'a'..'f';
    Ascii             = 0b00000000...011000000;
    Utf8Continuation  = 0b10000000...0b11000000;
    Utf8LeadingWidth2 = 0b11000000..0b11011111;
    Utf8LeadingWidth3 = 0b11100000..0b11101111;
    Utf8LeadingWidth4 = 0b11110000..0b11110111;
    InvalidUtf8Byte   = 0b11111000..0b11111111;
)

pub fn 

byte_match!(
    '0' -> {
        'x'|'X' -> 

        },
        'o'|'O'
    },
    Digit -> {

    },
    Digit
        
    },
    Digit => {},
    
    
)

pub fn atomic_term(scanner: &Scanner) {
    match scan(Digit) {

    }
    lex!(
        Digit+ => {

        },
        IdentStart Ident+ => {

        }
    )
}

sequence!()



    Sp = " "|"\t";
    space Newline = "\r\n"|"\r"|"\n";
    Invalid = InvalidUtf8Byte* error(InvalidUtf8);
    Unsupported = ValidUtf8Char error(UnsupportedCharacter);

    // Tokens
    term Number = Digit+ (
        Ident+ <error(IdentifierStartsWithNumber)> |
        <IntegerLiteral>
            ("." infix() )?
        _ => Exponent?(Float?(term(IntegerLiteral)))
    )

    infix Float = "." <Integer> => ;
    infix Exponent = <
    postfix Imaginary = 

    DecimalFloat = Integer "." 

    AfterInt = {
        ["."] decimals:Digit+ {
            ["eE"] ["+-"]? Digit+ {
                ["iI"] -> ImaginaryLiteral,
                Ident+ -> error(IdentifierStartsWithNumber),
            }
        },
        Ident+ -> error(IdentifierStartsWithNumber),
        _ => 
    };
)

    pub const Punctuation = any(vec![
        Dot, ExclamationPoint, QuestionMark, Tilde, At,
        Minus, Plus, Star, Slash, Hash,
        Quote, SingleQuote, Ampersand, Pipe,
        OpenParen, OpenBracket, OpenCurlyBrace, OpenAngle,
        CloseParen, CloseBracket, CloseCurlyBrace, CloseAngle,
    ])
    );

    tokens!(
        Zero = [0]*
    )

    // NOTE: this stops working when some unicode characters are supported.
    pub const Supported = any2(Ident, Punctuation);
    pub const Digit = r('0','9');
    pub const I = any2(b('i'), b('I'));
    pub const E = any2(b('e'), b('E'));

    pub const Zero = b('0');
    pub const X = any2(b('x'), b('X'));
    pub const O = any2(b('o'), b('O'));
    pub const B = any2(b('b'), b('B'));
    pub const HexDigit = any3(Digit, r('A','F'), r('a','f'));
    pub const BinaryDigit = r('0','1');
    pub const OctalDigit = r('0','7');

    pub const Alpha = any2(r('A','Z'), r('a','z'));
    pub const IdentStart = any2(Alpha, Underscore);
    pub const Ident = any3(Alpha, Digit, Underscore);

    pub const Punctuation = any(vec![
        Dot, ExclamationPoint, QuestionMark, Tilde, At,
        Minus, Plus, Star, Slash, Hash,
        Quote, SingleQuote, Ampersand, Pipe,
        OpenParen, OpenBracket, OpenCurlyBrace, OpenAngle,
        CloseParen, CloseBracket, CloseCurlyBrace, CloseAngle,
    ])
    pub const Dot = b('.');
    pub const ExclamationPoint = b('!');
    pub const QuestionMark = b('?');
    pub const Tilde = b(')
    pub const At = b('@');
    pub const Minus = b('-');
    pub const Plus = b('+');
    pub const Star = b('*');
    pub const Slash = b('/');
    pub const Hash = b('#');
    pub const Quote = b('"');
    pub const SingleQuote = b('\'');
    pub const Ampersand = b('&');
    pub const Pipe = b('|');

    pub const OpenParen = b('(');
    pub const OpenBracket = b('[');
    pub const OpenCurlyBrace = b('{');
    pub const OpenAngle = b('<');
    pub const CloseParen = b('(');
    pub const CloseBracket = b('[');
    pub const CloseCurlyBrace = b('{');
    pub const CloseAngle = b('>');

    type ByteRecognizer = fn(u8)->bool;

    fn any(recognizers: Vec<ByteRecognizer>) {
        |ch| recognizers.any(|recognizer| recognizer(ch))
    }
    fn any2(a: ByteRecognizer, b: ByteRecognizer) -> ByteRecognizer {
        |ch| a(ch) || b(ch)
    }
    fn any3(a: ByteRecognizer, b: ByteRecognizer, c: ByteRecognizer) -> ByteRecognizer {
        |ch| a(ch) || b(ch) || c(ch)
    }
    fn any4(a: ByteRecognizer, b: ByteRecognizer, c: ByteRecognizer, c: ByteRecognizer) -> ByteRecognizer {
        |ch| a(ch) || b(ch) || c(ch) || d(ch)
    }
    fn not(inner: ByteRecognizer) -> ByteRecognizer {
        |ch| !inner(ch)
    }
    fn r(start: u8, end: u8) -> ByteRecognizer {
        |ch| ch >= start && ch <= end
    }
    fn b(start: u8, end: u8) -> ByteRecognizer {
        |ch| ch >= start && ch <= end
    }
}

mod recognizers {
    trait SequenceRecognizer<T> {
        pub fn recognize(scanner: &Scanner) -> T;
    }
    trait ByteRecognizer<T> {
        pub fn recognize(ch: u8) -> (T, Next);
    }
    struct TrieByteRecognizer<T, R: Recognizer> {

    }
    struct Sequence
}

mod utf8 {
    pub const AsciiByte            = chars::r(0b00000000, 0b01111111);
    pub const Utf8ContinuationByte = chars::r(0b10000000, 0b10111111);
    pub const Utf8LeadingWidth2    = chars::r(0b11000000, 0b11011111);
    pub const Utf8LeadingWidth3    = chars::r(0b11100000, 0b11101111);
    pub const Utf8LeadingWidth4    = chars::r(0b11110000, 0b11110111);
    pub const InvalidUtf8Byte      = chars::r(0b11111000, 0b11111111);

    pub fn accept_valid_utf8_until(scanner: &mut Scanner, recognize: ByteRecognizer) -> bool {
        if let Some(buf) = scanner.fill_buffer(1) && recognize(buf[0]) {
            false
        } else {
            loop {
                self.accept_single_valid_utf8(scanner)
                if let Some(buf) = scanner.fill_buffer(1) && recognize(buf[0]) {
                    break;
                }
            }
            true
        }
    }
    pub fn accept_valid_utf8(scanner: &mut Scanner) -> bool {
        if accept_single_valid_utf8(scanner) {
            while accept_single_valid_utf8(scanner) {}
            true
        } else {
            false
        }
    }
    pub fn accept_single_valid_utf8(scanner: &mut Scanner) -> bool {
        if let size = valid_utf8_byte_width(scanner) && size > 0 {
            scanner.accept(size);
            true
        } else {
            false
        }
    }
    pub fn valid_utf8_byte_width(scanner: &mut Scanner) -> Option<usize> {
        if let Some(buf) = scanner.fill_buffer(1) {
            // Determine the character width from the first character
            let type = utf8_byte_type(buf[0]);
            match type {
                Invalid => Some(0),
                // Cannot have continuation characters at the beginning of the char.
                Continuation => Some(0),
                Ascii => Some(1),
                _ => {
                    // Check that the continuation bytes start with 10xxxxxx
                    if let Some(buf) = scanner.slice(type.width()) {
                        let remaining_bytes = buf.iter().skip(1);
                        if remaining_bytes.all(|ch| utf8_byte_type == Continuation) {
                            Some(type.width())
                        }
                    }
                }
            }
        } else {
            None
        }
    }
    // Accepts all characters until the first valid UTF-8 character.
    pub fn accept_invalid_utf8(scanner: &mut Scanner) -> bool {
        if let Some(0) = valid_utf8_byte_width(scanner) {
            scanner.accept(1);
            while let Some(0) = valid_utf8_byte_width(scanner) {
                scanner.accept(1);
            }
            true
        } else {
            false
        }
    }

    enum Utf8ByteType {
        Ascii,
        Continuation,
        TwoByte,
        ThreeByte,
        FourByte,
        Invalid,
    }
    impl Utf8ByteType {
        fn width(&self) -> {
            match *self {
                Ascii => 1,
                TwoByte => 2,
                ThreeByte => 3,
                FourByte => 4,
                Invalid => 0,
                Continuation => 0,
            }
        }
    }
    fn utf8_byte_type(first_byte: u8) -> Utf8ByteType {
        // 0xxxxxxx is ASCII (1 byte).
        if (ch & 0b10000000) == 0 {
            Ascii
        // 10xxxxxx is a continuation byte.
        } else if (ch & 0b01000000) == 0 {
            Continuation
        // 110xxxxx starts a 2-byte character.
        } else if (ch & 0b00100000) == 0 {
            TwoByte,
        // 1110xxxx starts a 3-byte character.
        } else if (ch & 0b00010000) == 0 {
            ThreeByte,
        // 11110xxx starts a 4-byte character.
        } else if (ch & 0b00001000) == 0 {
            FourByte,
        // 11111xxx is always invalid.
        } else {
            Invalid,
        }
    }
}

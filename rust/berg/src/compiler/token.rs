struct Token(TokenType, String);
struct TokenSpan(Token, usize);

enum TokenType {
    Eof,
    Space,
    Newline,
    SingleLineComment,
    InfixOperator,
    PrefixOperator,
    PostfixOperator,
    Bareword,
    IntegerLiteral,
    FloatLiteral,
    ImaginaryLiteral,
    HexadecimalLiteral,
    OctalLiteral,
    BinaryLiteral,
}

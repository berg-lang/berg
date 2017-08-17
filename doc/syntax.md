Berg Syntax
===========

Berg syntax lets you write expressions, with one statement per line, in UTF-8 format.

Design Goals
------------

- Editing Flow: Major text and document editors read and write Berg smoothly *by default.*
- Tool Integration: Writing development tools and editor extensions is as easy as possible.
- Unicode Support: Unicode is supported wherever possible.
- Shareable: code written anywhere can be compiled anywhere. Copy/paste to/from any editor/viewer.
- Fast Compile: parsing should be fast and small, with the right structures to support fast typecheck and compilation.

### Editor Experience

When we read and write source code, we rely on the editors, IDEs, and other tools to *visualize* it for us. These tools vary widely in their capabilities and display. Berg needs to read and write comfortably in all of them, preferably working well with default editor features like colorization and autocorrect. Additionally, many of these IDEs and editors.

Blocks and Expressions
----------------------

### Values

Values represent the results of computational processes (from something as simple as retrieving the integer literal `1`, to the result of a fibonacci sequence). Everything--primitives, processes, objects, functions--is a value.

Values are accessible solely through their **properties**, which may perform visible **side effects** such as I/O, logging, or network API calls, in the course of their computation. Properties always have the same value and will only ever perform side effects once, making values essentially **readonly** things that converge on values.

A value can be **forked** (possibly with parameters), creating a new value based off the old one. Properties that have already been calculated will not change their value; side effects that have already been performed will not be re-performed.

### Types

A **type** describes the range of things that can be done with a property:

* The set of properties and their types.
* All possible pending side effects of completing the property.
* Whether the property is complete.
* Whether the property *can* complete, and reason why not (e.g. it is not bound).
* The origin expressions for the value of this property

### Side Effect Descriptions

A side effect description describes an external change that may happen in the course of completing a value. These descriptions are domain-specific and built so that the type system and compiler can order side effects that need to be ordered without forcing everything to happen in order. System calls generally have types with side effects.

### Blocks

A block forkable code--a set of Berg statements which:

* Is a **process**.
* May be **partial**--i.e. have unbound global or local properties which may be *applied* to it. Code with unbound variables cannot be run and will throw an exception if accessed.
* Can be **forked** by applying arguments to it.

### Values

A **value** is a concrete set of properties. Everything in Berg is a value. Examples include function calls and objects.

A **property** is a name with a value (instance). A property value may be **not set** (in which case callers will wait on it) or **abstract** (has not been set, like a function parameter or a class member).

An **instance** is a live expression, statement, block, class or function, which has been instantiated with an input context.

A **primitive** is an instance of a primitive type like int, float or boolean.
    An **object**

### Expressions

An **expression** is any unbound Berg code (including, for example, literals, operators, property references, function calls).
    - It has an input block, output block and result **type** inferred from the expression itself.
    - It can be **bound** to a context (input) expression, and produces a new expression with a tighter type.
    - Most expressions produce a **result** which can be applied to other expressions.
    - Some expressions (like property declarations) modify or replace the **output block**.
    - An expression can be **instantiated** (given a live input value) and then **run**.
    - Expressions bind their component expressions in reading order, left to right, when they are themselves bound.
    - Unbound variables are considered an error when the entire source file is parsed without encountering a declaration or using statement.

* **Apply:** An expression can be *applied* to another expression, performing a function call or an instantiation.

A **function** is a block that returns the $Value of the last expression.

A **class** is a block with properties. It can be instantiated like any other expression, and the resulting value is called an **object**.

### Types

A **type** is a description of the properties an expression can or will have.

Block Structure
---------------

Blocks and statements are the visual structure defining the flow of Berg expressions.

A *block* is a rectangular block of text containing a series of horizontally aligned statements. A block has a *margin*--the indent level that all statements start on--and a *boundary*, a minimum indent level that all lines in the block must stay inside.

```
# The margin of this block is 0. It has no boundary.
if x == y
    # The margin of this block is 4 (just before the `z`).
    # Its boundary is 1 (just after the parent margin)
    z = x * x
    z *= 2
```

### Statement Breaks

A *statement break* starts a new statement, and happens anytime a line break happens between two separate expressions. More specifically, if the first statement ends with an expression or postfix operator, and the second line begins with an expression or prefix operator, a statement break is caused.

What the statement break *means* depends on where it happens.

- If the new statement is *more* indented than the block margin, a new child block is created with that margin and *applied* (as a function argument) to the current statement.
- If the new statement is *on* the block margin, it *extends* the block.
- If the new statement is on the *parent* block margin, it *closes* the block (and extends the parent block).

```
if x == y
    # This statement is *applied* to the "if" as a new child block.
    z = x * x
    # This statement *extends* the child block, and thus happens after z = x * x.
    z *= 2
# This statement *closes* the child block (and *extends* the block with the if statement).
print "hello world"
```

#### Extend

When a block is *extended*, the new statements are *appended* to it. New statements run after the existing ones, and both new and old statements are *rebound* (in case the new statement introduces a new property for it to access!).

Apply (like any other function argument) does not extend or rebind the statement it applies to.

### Statement Continuations

When a line break happens and one of the expressions is *incomplete*, the statement *continues* across the lines, and the line break is ignored. Specifically, if the first line ends with a prefix or infix operator, or the second line begins with an infix or postfix operator, continuation happens.

Line continuations can have any indent they want, as long as they are within the block boundary. The indent does not affect them. One exception: if the *second* line begins with a prefix or expression, it cannot be "undented" and must either be on the margin or indented. Here are a few possible indentations for a line continuation:

```
# Lined Up
if x == y
    a*b*c
    + d*e*f
    + g*h*i
if x == y
    a*b*c +
    d*e*f +
    g*h*i

# Indented
if x == y
    a*b*c
      + d*e*f
      + g*h*i
if x == y
    a*b*c +
      d*e*f +
      g*h*i

# Outdented
if x == y
      a*b*c
    + d*e*f
    + g*h*i
```

### Declaration Operator

The declaration operator `:` acts a little differently from other operators. If you use a normal operator, it will not create a new statement on the next line. However, `:` at the end of a line *will* create a child block. This is not affected by comments on the line after the `:`.

```
Wheel:
    Spokes: 4
    Type: "Rubber"
```

### Ending a Block

Blocks end automatically when you start a new statement in the *parent* block, or at the end of the file. You can also end a block with an `end` statement in the parent block.

```
if x == y
    z = x * x
    z *= 2

print "hello" # this ends the previous block

if x == y
    z = x * x
    z *= 2
end
```

Indenting blocks creates a nice ramp for *scanning*, because you can see one indent show up at a time. Berg disallows *closing* more than one block at a time to create the same gentle ramp back down.

### A Word About Semicolons and Braces

Semicolons and braces are allowed in Berg for convenience (so that you can place blocks and multiple statements on a single line). They are not intended to be placed after every statement. To ensure a similar look across source code, `;` is disallowed at the end of a line and `{` is not allowed at the end of a line if it is preceded by `apply` or by a block-creating operator like `:`.

### Comments and Indentation

Because they are visible, comment-only lines can end a block.

```
foreach foo in foos
    if x == y
        z = x * y
        z *= 2
    # This comment ends if x == y
# This comment ends foreach foo in foos
```

Comments can also be the start of a statement, as long as there is a complete expression on either side. This is important for knowing where a block really begins.

```
if x == y
    # this block begins
    # not with a bang, but with a hash

    z = x * y
    z *= 2
```

As a side effect, comments can have an effect on the margin of a child block as well:

```
if x == y
    # This block starts here
        # And therefore this statement is misaligned!
        z = x * y
    # And this one is not.
    z *= 2

```

### Comment Attachment

Berg specifies which statement or term an operand "attaches to" in order to ensure documentation comments attach correctly for tooling.

* Comments after a statement break attach to the following statement (not to the entire block).
* Comments between an operator and operand associate with the first child of the operator (an ancestor of the operand).
* Comments immediately following a declaration operator (`:`) are associated with the declaration.
* Comments at the bottom of the file attach to the entire source expression.

```
Foo: 1 + 2 + 3 # This comment is associated with the `Foo` declaration

# This comment is associated with Car
Car:
    # This comment is associated with Wheel
    Wheel:
        # This comment is associated with Spoke
        Spoke: # This comment is associated with Spoke too
            # This comment is associated with the statement print "hello" + " world"
            print "hello" +
                " world
            # This comment is associated with that statement too
        # This comment is associated with Wheel
    # This comment is associated with Car
# This comment is associated with nothing (the file, basically)
```

Identifiers
-----------

Berg identifiers are Unicode, case sensitive, and can contain letters, numbers and underscores. Identifiers cannot start with a number.

### Unicode Identifiers

Unicode identifier support follows the recommendations from [Unicode identifier syntax revision 27](http://www.unicode.org/reports/tr31/#Default_Identifier_Syntax), adding leading underscores. Non-normalized Unicode characters are disallowed (if there are two ways to write a Unicode character, we only accept the "canonical" one). Only recommended scripts (the ones in modern use with large communities) are supported to keep potential security issues caused by confusingly similar identifiers to a minimum.

In total, identifiers can start with any Unicode `Letter`, `Letter_Number`, or `Other_XID_Start=True` character, or underscore; and can contain any of those characters *plus* `Nonspacing_Mark`, `Spacing_Combining_Mark`, `Decimal_Digit_Number`, `Connector_Punctuation` and `Other_XID_Continue=True` character.  Identifiers *cannot* contain `Pattern_Syntax` and `Pattern_White_Space` characters. They *must* be in Normalized Form C of the Unicode standard.

Only identifiers from the recommended scripts in [Table 5](http://www.unicode.org/reports/tr31/#Table_Recommended_Scripts) of the specification: specifically, the Common, Inherited, Arabic, Armenian, Bengali, Bopomofo, Cyrillic, Devanagari, Ethiopic, Georgian, Greek, Gujarati, Gurmukhi, Han, Hangul, Hebrew, Hiragana, Kannada, Katakana, Khmer, Lao, Latin, Malayalam, Myanmar, Oriya, Sinhala, Tamil, Telugu, Thaana, Thai, and Tibetan scripts.

Numeric Literals
----------------

Berg has arbitrary-sized integer and floating-point literals, with integers specifiable as decimal, hexadecimal, octal or binary.

    ```
    NumericLiteral: IntegerLiteral | FloatingPointLiteral | HexadecimalLiteral | OctalLiteral | BinaryLiteral
    ```

Berg does **not** support general Unicode digits, only Basic Latin.

### Integer Literals

Integers are whole numbers with no fractional part. Integer literals can be arbitrarily sized. Examples:

    ```
    0
    123
    9999
    123847613874631876431867
    ```

Grammar:

    ```
    IntegerLiteral: Digit+
    Digit: "0".."9"
    ```

### Floating-Point Literals

Floating-point literals are numbers with decimal points and exponents. Examples:

    ```
    123.456
    123.456e10
    123e10
    123.456e-10
    123.456e+10
    ```

Grammar:

    ```
    FloatingPointLiteral: DecimalPointLiteral | ExponentLiteral
    DecimalPointLiteral: IntegerLiteral "." IntegerLiteral
    ExponentLiteral: (IntegerLiteral | DecimalPointLiteral) ("e" | "E") (IntegerLiteral | ExponentSignLiteral)
    ExponentSignLiteral: ("+" | "-") IntegerLiteral
    ```

### Imaginary Literals

Any floating-point or integer literal can be suffixed with `i` to create an imaginary number. Examples:

    ```
    100i
    99.99i
    99.99e-10i
    ```

Grammar:

    ```
    ImaginaryLiteral: (IntegerLiteral | FloatingPointLiteral) ("i" | "I")
    ```

### Hexadecimal, Octal and Binary Literals

Numbers prefaced with `0x`, `0o` and `0b` are interpreted as hexadecimal (base 16), octal (base 8), and binary (base 2). They may be arbitrary precision, and a-f are used to represent the digits 10-15. Examples:

    ```
    0x100
    0xABC
    0x01af0
    0o777
    0b010011
    ```


Grammar:

    ```
    HexLiteral: "0" ("x"|"X") HexDigit+
    HexDigit: Digit | "A".."F" | "a".."f"
    BinaryLiteral: "0" ("b"|"B") BinaryDigit+
    BinaryDigit: "0" | "1"
    OctalLiteral: "0" ("o"|"O") OctalDigit+
    OctalDigit: "0".."7"
    ```

String Literals
---------------

String literals in Berg start and end with quote (`"`), support escape sequences and string interpolation. Any UTF-8 character except `\`, `"` and newline are treated directly as part of the string.

    ```
    "Hello World"
    ```

Grammar:

    ```
    StringLiteral: "\"" StringPart+ "\""
    StringPart: EscapeSequence | !Newline
    EscapeSequence: "\\" ("0"|"\\"|"t"|"n"|"r"|"\"" | "u" HexDigit+ | "u{" HexDigit+ "}")
    Newline: "\r" | "\n"
    ```

### String Escapes

Escape sequences starting with the `\` character are supported:

| Name            | Escape                   |
|-----------------|--------------------------|
| Null Character  | `\0`                     |
| Backslash       | `\\`                     |
| Horizontal Tab  | `\t`                     |
| Line Feed       | `\n`                     |
| Carriage Return | `\r`                     |
| Double Quote    | `\"`                     |
| Unicode scalar  | `\unnnnnn`, `\u{nnnnnn}` |
|-----------------|--------------------------|

`\` followed by anything else is an error.

### String Interpolation

The escape sequence \( starts an *interpolated expression*, which runs a Berg expression and concatenates the result into the string by calling its ToString property. Expressions in interpolated strings may not contain newlines.

    ```
    "I have over \(number*1000)!"
    ```

Interpolated strings may be nested arbitrarily.

    ```
    "the quick brown \(animal == "fox" ? "fox" : "non-fox") jumped over the lazy dog"
    ```

Whitespace
----------

Berg source code is in *text documents,* and is meant to be read by the human eye. As such, we give visual space similar meanings to paragraphs, sentences, words and sections of a book or paper.

* Adjacent expressions on the same line are treated as function calls
* Separate statements are on separate lines
* Indentation indicates block nesting

Whitespace also determines line and column numbers for error messages.

### Horizontal Space

Horizontal space is used as a separator between numbers and identifiers, and to indicate nesting level via indentation.

| Code Point    | Name                      | Abbreviation | Notes |
|---------------|---------------------------|--------------|-------|
| U+0009        | CHARACTER TABULATION      | <TAB>        |       |
| U+000B        | LINE TABULATION           | <VT>         |       |
| U+0020        | SPACE                     | <SP>         |       |
| U+00A0        | NO-BREAK SPACE            | <NBSP>       |       |
| U+FEFF        | ZERO WIDTH NO-BREAK SPACE | <ZWNBSP>     | Disallowed between two characters |
| “Zs” Category | Space_Separator property  | <USP>        |       |

#### Zero-Width Space

In general, characters which do not change the file visually are disallowed from causing changes in meaning for security reasons. Zero-Width Spaces (outside of strings and comments) can have this effect. We do not completely disallow all of them, because there are a few cases where they can legitimately appear. Here is a list.

U+FEFF (ZERO WIDTH NO-BREAK SPACE) sometimes appears at the beginning of the file as a byte order mark (BOM), or in the middle of the file when several files have been concatenated. U+FEFF is treated as whitespace generally, but is *disallowed* between two printable characters outside of comments and strings.

U+200C (ZERO WIDTH NON-JOINER) and U+200D (ZERO WIDTH JOINER) can cause some words to display legitimately differently. They are allowed in the middle of identifiers (not the first or last character), and disallowed elsewhere   outside of comments and strings.

### Newlines

Newlines always end a line, starting new characters at the beginning of the next line. Spaces after newline are considered indent. CR followed by LF, the Windows default newline, is considered a single newline for all relevant purposes.

| Code Point    | Name                      | Abbreviation | Notes
|------------------------------
| U+000A U+000D |                           | <CR> <LF>    |
| U+000A        | CARRIAGE RETURN           | <CR>         |
| U+000C        | FORM FEED                 | <FF>         |
| U+000D        | LINE FEED                 | <LF>         |
| U+0085        | NEXT LINE                 | <NEL>        |
| U+2028        | LINE SEPARATOR            | <LS>         | Never separates statements
| U+2029        | PARAGRAPH SEPARATOR       | <PS>         | Always separates statements

NOTE: not all newlines have the exact same meaning: Line Separator and Paragraph Separator are unambiguous, unlike other newlines.

When a line causes a block to be unindented / closed, it is treated as a statement separator. When the newline is between an operator and its operand, it is treated as whitespace and the expression is parsed as if it were a single line. Otherwise, it is treated as a statement separator.

#### Unicode Line Separator and Paragraph Separator

The Unicode Line Separator (LS) character *never* acts as a statement separator. Using LS where an LF would have separated statements triggers an `apply argument` function call without creating a block.

The Unicode Paragraph Separator (PS) character always acts as a statement separator, and *prevents* this from happening. Using PS where an LF would have continued statements will either change the type of operator inferred, or cause an error.

JKEISER NOTE: I'm sort of uncomfortable with this. Doesn't this mean that two visually identical files will behave differently? Or are LS and PS displayed differently from LF and each other?

### Line / Column Number

When calculating the line or column number for error messages, any character except Newline and TAB cause a single column increase. The following table is used:

| Character         |                        |
|-------------------|------------------------|
| Newline           | +1 line, column = 1    |
| TAB               | next 4-column tab stop |
| Horizontal Space  | +1 column              |
| Anything Else     | +1 column              |


Source Text
-----------

### Visual Ordering and Indentation

In Berg, as in most languages, we use the ordering and placement (affected by whitespace) of text to determine its meaning. For example, horizontal space separates words (`ifrit` means something very different from `if rit`). Because of this, it matters what *font* and *software* will display Berg source code. Berg makes these assumptions about how text will be displayed:

* Berg assumes that non-zero-width characters will take up *some* horizontal space--for example, that 2 spaces will always be larger than 1.
* Berg assumes that all lines begin at the same horizontal position.
* Berg assumes that vertical space characters such as newline will always move downwards past the current line, to the beginning of the next.
* Berg assumes that zero-width characters will take no space.
* Berg does **not** assume a particular width or height for any character ([some Unicode characters are extra large](http://denisbider.blogspot.com/2015/09/when-monospace-fonts-arent-unicode.html).
* Berg does **not** assume tabs have a certain size, or any other space character.

For more on the intended size and function of many Unicode space characters (which Berg ignores without causing an issue), see [this image from Wikipedia](https://en.wikipedia.org/wiki/Whitespace_character#/media/File:Punctuation-Spaces.svg).

### Error Locations: Lines and Columns

While the meaning of Berg programs are not affected by character width, the compiler outputs errors with *lines* and *columns* in them which assume a monospaced display. With respect to lines and columns, Berg *does* assume a monospaced font. It treats all characters as taking up a single column, except:

* Newlines go to the next line, column 1.
* Vertical tabs go to the nearest 6-line tab stop (1, 7, 13, ...).
* Horizontal tabs go to the nearest 8-column tab stop (1, 9, 17, ...).
* Zero-width spaces take up 1 space.

Unicode
-------

Berg source code has pervasive Unicode support, allowing international characters in properties, operators and strings. However, in order to keep Berg's portability high, and prevent subtle bugs, there are some things it explicitly does *not* make multilingual.

### Normalized Property Names

In Unicode, some characters have multiple representations. Berg property names **must be** in normalized form. If a property name contains "a"+"sharp accent," it is considered an error and must be replaced with "sharp a" to compile. This is done to help prevent subtle bugs from happening in development tools that also parse Berg files. When filenames are used as property names, this also applies to the filename.

This is because Unicode has special *modifier* codepoints like "add a sharp accent to this letter," and when you see "a" followed by "sharp accent," it's the same as the "accented a" character. The issue is that these two characters look *exactly the same* in a text editor, and make debugging life difficult sometimes.

Unicode specifies a way to normalize these, so that they can be compared. It also specifies *compatibility*, going even further and saying (for example) that the two-character "ff" and the "ff" character are the same. We don't use compatibility in the source code at all, preferring that there not be multiple ways to express identical statements.

### No Multilingual Numbers

Unicode allows you to write numbers in many languages, but Berg only supports Basic Latin to support tool development. Berg does not support them, requiring you to write Basic Latin 0-9. Hex requires Basic Latin "A-F" and "a-f". The exponent character in floats ("e" or "E") and the imaginary suffix ("i" or "I") are Basic Latin as well.

### No Multilingual Keywords

Berg keywords and standard library functions are English only. The reason for a single language is to support portability: we want programs and tutorials to work anywhere. English is the choice because that's how most other popular languages have chosen as well.

### String Characters

It's worth pointing out, with the above issues, that string literals will not normalize: the string will directly transcribe any valid Unicode, byte for byte, in UTF-8, with the exception of `\\` and `"`.

### Comment Characters

Comments can contain any character except a newline.

### Only Valid Unicode

Some Unicode characters are invalid. Only Unicode between U+0000 and U+10FFFF. These may not be placed *anywhere* in the source code, including comments and strings.

Syntax Errors
-------------

### General Syntax Errors

* **Missing Parameter:** Two operators are next to each other.

  Example:
  ```
  a ||  || c
  ```

  Error Location: zero width, directly after the first operator.
  
  Suggestions: delete one or the other operator, or write an expression between them.

  Resume: Parser inserts error expression between operators.

* **Unrecognized Unicode Characters:** a string of valid Unicode characters not recognized in the grammar.

### Block and Statement Errors

* **Inconsistent Space Error:** when a line has different space characters (tabs, etc.) than the open indent.
* **Multiple Undent Error:** when a new statement closes more than two blocks.
* **Undented Continuation Error:** when a continuation line lies on or above the parent margin.
* **Misaligned Indent Error:** when a new statement starts *between* two margins.
* **Ambiguous Continuation Error:** when a continuation line *could* be a new statement (previous line ends with an operator, next starts with operand), and starts *between* the two margins.
* **Redundant Braces:** `{` and `}` are surrounded by `block`/`apply`/`:` (infix block operator) and `extend`, and therefore can (and should) be removed without having any effect. This is a style error enforced by the compiler to keep a measure of visual consistency across Berg source.
* **Redundant Semicolon:** `;` at the end of a line (even if there is a comment after it). This is a style error enforced by the compiler to keep a measure of visual consistency across Berg source.

### Identifier Errors

* **Identifier Starting With Number:** An identifier was found starting with a number.
* **Non-Normalized Identifier:** Unicode identifier *not* normalized with NFKC.
* **Identifier In Unsupported Unicode Script:** An identifier from an unsupported Unicode script for security reasons.

### Numeric Literal Errors

* **Integer With Leading Zero:** A valid decimal number with more than one digit, starting with `0`. (Because it sounds like octal.)
* **Binary Literal With Decimal Digits:** A binary literal with decimal digits 2-9 in it. If there are identifier characters after these digits, IdentifierStartingWithNumber is emitted instead.
* **Octal Literal With Decimal Digits:** A binary literal with decimal digits 8-9 in it. If there are identifier characters after these digits, IdentifierStartingWithNumber is emitted instead.
* **Float Literal With Empty Exponent:** A float like `1.2e` not followed by a number. `1.2e-` or `1.2e+` with no digits will emit the error against `1.2e` and parse the sign as a separate symbol. If there are identifier characters following `1.2e`, IdentifierStartingWithNumber is emitted instead.

### Encoding Errors

* **Unsupported Encoding:** if a zero appears in the first 1024 bytes, the file is considered to be an unsupported encoding. If FEFF or FFFE shows up at the beginning of the file, we specifically suggest it is UCS-2. Invalid UTF-8 (see [the UTF-8 RFC](https://tools.ietf.org/html/rfc3629#section-3) and [Wikipedia](https://en.wikipedia.org/wiki/UTF-8#Invalid_byte_sequences) for details). This causes parsing to stop.
* **Invalid Unicode Character:** if an invalid Unicode character (outside the range or unused in Unicode 10.0) is encountered. These are ignored and not emitted. TODO use FEFF as a File Separator?
* **Ridiculously Long Unicode Character:** If a Unicode character (grapheme) larger than 64 codepoints (for a max of 256 bytes) is encountered. These may be truncated before passing to the scanner.

### Numeric Literal Errors

The following errors are specific to numeric literals:

* **Identifier Starts With Number:** Numbers *must not* be followed immediately by property names.

    ```
    # ERROR: names cannot begin with numbers. Perhaps you meant to place an operator between `99.99` and `Percent`?
    2001ASpaceOdyssey
    99.99Percent
    99.99iRobot
    0x1y
    ```

* **Floating Point Number Without Leading Zero:** Floating-point numbers starting only with `.` (without any digits) are **illegal** to prevent confusions between it and the `.` operator.

    ```
    # ERROR: Floating-point numbers must have a leading zero before the decimal point. Did you mean "0.123"?
    .123
    # ERROR: Integers cannot be prefaced with zero. Remove the leading zero to compile, or use Integer.FromOctal("0666").
    00
    0666
    0999
    ```

* **Integer cannot start with zero:** `0`-prefixed octal numbers are **not supported** and any multi-digit integer starting with 0 is illegal. Because some languages support 0xxx as octal representation and some do not, it is now ambiguous from an intuition standpoint. Treated it as an error prevents misunderstandings.

    ```
    # ERROR: Integers cannot be prefaced with zero. Did you mean "666" or perhaps octal "0o666"?
    00
    000666
    0999
    ```

* **Decimal digits in binary or octal number:** When `0b01241235` or `0o9999` happen.

* Floating-point numbers ending with `e`, `e+` or `e-` without a following digit yield the "empty exponent" error.

    ```
    # ERROR: Exponents must have integers. Did you mean "123e1"?
    123e
    123e-
    123e+
    ```

* **Floating point hexadecimal, binary or octal number.** Hexadecimal, binary and octal numbers cannot have decimal points or be imaginary.

    ```
    # ERROR: Hexadecimal numbers cannot have decimal points. Perhaps you meant "1.1" instead?
    0x1.1
    0o1.1
    0b1.1
    ```

    ```
    # ERROR: Binary numbers cannot be imaginary. Did you mean to place an operator like * or + between "0x1" and "i"?
    0x1i
    0o1i
    0b1i
    ```

* **Missing digits in hexadecimal, binary or octal number.** Hexadecimal numbers must have at least one digit.

    ```
    # ERROR: Hexadecimal, binary and octal numbers must have at least one digit. Did you mean "0x0"?
    0x
    ```

### String Literal Error Messages

* **Unterminated String Literal:** String is not closed or contains unescaped quotes. Attempts to guess the right place to close based on the location of the first operator, or if there are multiple lines with this error, suggests it may be a multiline string requiring \n.

    ```
    # ERROR: String is not closed or contains unescaped quotes. Did you mean '"hi there"'?
    if x == "hi there || x == "hello there"
       friendlyX = x + " my friend"
    ```

    ```
    # ERROR: String contains unescaped quote(s) or is unclosed. Did you mean '"look at my \"air quote\""'?
    Out.Print "look at my "air quote""
    ```

    ```
    # ERROR: String contains unescaped quote(s), has multiple lines, or is unclosed.
    Out.Print "look at my
       favorite string!"
    ```

* **Unsupported string escape character:** String escape character not supported.

    ```
    # ERROR: String escape "\k" not supported. Did you mean "\\k"?
    Out.Print "Hi there \k"
    ```

* **Missing Unicode character in Unicode escape:** If \u or \U is not followed by hex or {

* **Unrecognized character in Unicode escape block:** Anything except space or hexadecimal digits inside a Unicode escape block.

* **Out-of-range Unicode character in Unicode escape escape:** Unicode escape sequence out of range.

    ```
    # ERROR: Unicode escape sequence references a character that is out of range for valid Unicode (between \U000000 and \U10FFFF). Did you mean "\u{FFFFF}FFUUUUUUUUU"?
    Out.Print "\uFFFFFFFUUUUUUUUU"
    ```

* **Unterminated Unicode escape block:** Unicode escape sequence is not closed.

    ```
    # ERROR: Unicode escape sequence is missing a } at the end. Did you mean "First Line\u{0A}Second Line"?
    Out.Print "First Line\u{0ASecond Line"
    ```

* **Unterminated interpolated expression:** Expression inside string is not closed.

    ```
    # ERROR: Expression inside string is missing a ) at the end. Did you mean \(i+1)"?
    Out.Print "Number is \(i+1"
    Out.Print "hi"
    ```

* **Multiline expression in string:** Cannot have a multiline expression inside a string.

    ```
    # ERROR: Strings cannot contain multiline expressions. Did you mean \(i+1)?
    Out.Print "Number is \(i+
      1)"
    ```


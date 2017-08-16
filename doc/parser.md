Berg Parser
===========

The Berg parser is designed to be small, fast, extensible, and produce friendly error messages.

* **Fast and Small:** Allocations happen infrequently. It is cache-friendly, minimizing lookups and dependencies. Tokens (including strings) are interned, keeping size (and thus cache) low and comparison fast.
* **Replicable:** The parser core is simple, small, unchanging, and replicable in multiple languages. The core is designed to load in most of the actual language definition and errors from Berg files.
* **Delightful Errors:** Errors tell you your actual mistake and how to remedy it. We find more than one error if there is one. There are no warnings, only errors.
* **Berg-Shaped Features:** Indented blocks and multiline strings, string interpolation, postfix match operators (`a+`), and operatorless function calls (`a b`) all cause interesting decisions as to the shape of the parser.

Design
------

The parser is designed to *bootstrap*--that is, it has a small core parser algorithm that parses a subset of the language, and then loads some core Berg files to flesh out the rest of the language. This allows the language definition to evolve

The bootstrap parser is designed to be *configurable*--it can be given operator and identifier information, and will parse files into AST. This allows us to bootstrap like so:

1. Initialize the core parser class with a minimum, hardcoded grammar.
2. Initialize the core parser class based on the resulting ASTs.
3. Now you have a fully functional Berg parser!

Core Parser Design
------------------

A parser takes in a set of grammar definitions and a UTF-8 file or stream, and outputs a Berg expression syntax tree, comments, line/column data, and errors. These results are sufficient for most tool interaction including binding and typechecking, interpreting, colorization, outlining (the +/- dropdowns), documentation comments, error messages, automatic error fixup, and source reformatting.

The parser is entirely geared toward parsing operators and literals with precedence. It splits parsing into three phases: scanning, tokenizing and tree building. Each phase *will continue* if there is an error, doing its best to guess the intent and find more errors.

- **Stream:** The *stream* yields Unicode characters from the source.
- **Scanner**: Breaks the character stream up into distinct *symbols.*
- **Tokenizer**: *Directionalizes* the symbols into a coherent ordering of *tokens*, where every operator has an operand.
- **Parser**: *Nests* the tokens into an expression tree, determining block and statement boundaries, and ordering operations with precedence and associativity.

### Core Concepts:

* **Expression:** An *expression* is series of literals, identifiers, or operators with operands as subexpressions, that forms a single computation (no missing operands).
* **Block:** A *block* is a rectangular block of text containing a series of horizontally aligned statements. A block has a *margin*--the indent level that all statements start on--and a *boundary*, a minimum indent level that all lines must stay completely inside, generally defined as "deeper than the parent block." Statements in a block are joined using *extend*.
* **Statement:** A *statement* is a single, complete expression taking up one or more lines of a block. Its first line is on the block margin, and it may have multiple *continuation* lines as long as they are within the block boundary.
* **Child Block:** Indented expressions (expressions that start past the margin) always create a new *child block* on the new margin. If the current statement ends with an operator, the block is passed as an argument to the operator. If the current statement is a complete expression, the block is passed to the expression using the *apply* operator.
* **Symbol:** Distinct piece of text that represent individual operations, expressions, space or comments (for example, `1/-245` is four symbols). Some symbols may correspond to multiple `tokens` , deciding whether `+` for example should be infix (`a plus b`), prefix (`positive x`), or postfix (`x repeated many times`). Symbols do not have precedence because prefix and infix `+`, for example, have *different* precedences.
* **Token:** Specific operators, expressions, spaces and comments, with precedence, associativity and fixity (infix, postfix, prefix, or expression).
* **Source:** A *source* uniquely identifies a given Berg source file. It can be opened to produce a stream, and has the filename or URL of the source for printing.
* **Syntax Tree:** The parser output is a *syntax tree*, a tree of operations, identifiers, and literals showing the order of operations. The syntax tree also has debug data: comments, token positions, and line/column information for printing. The syntax tree is a complete representation of all non-comment, non-whitespace in the file. Strings, numbers, identifiers, operators, linked together so that (for example) you know that in `1 + 2`, `1` is the left operand of `+` and `2` is the right operand. The tree can be traversed top-down, bottom-up, or in iterated in lexical order (reading order).
* **Operation:** Syntax trees are composed of *operations*, which identify the token type, operands, parents and next/previous token in the file, as well as the actual string value.

Parsing will always create a syntax tree. If there are parse errors, the parser will insert "error tokens" in the file at that point detailing the error so they can be reported.

### Grammar

The *grammar* is the definition of the language. It is specified in terms of tokens--operators and literals with precedence. A grammar is intimately tied to all parts of the process, but is most strongly tied to the *scanner.* It has a list of all the *symbols* and *tokens* in the language.

#### Symbol Type

  A *symbol* type defines the physical text for each token--for example, `"+"` or `a series of decimal digit characters [0-9]+`--as well as the different possible ways that text could be interpreted (`+` could be prefix (positive), infix (plus) or postfix (match many)).
  
  A symbol may be ambiguous if it has multiple variants (token types) with different fixity: an expression, infix, postfix and prefix. Symbol types are scanned by the scanner, and the symbol is analyzed in context to decide which token was meant. For example, the `+` symbol has  variants. When we read `1 + 2`, we pick the infix (plus) variant, but when we read `1 * +2`, we pick prefix (positive).

  A symbol contains this data:
  
  - *Variants:* the infix, prefix, postfix and expression token types representing variants of this symbol.
  - *Variant Preference:* Whether this symbol prefers infix over prefix, and whether it prefers expression over postfix. Most symbols use `true` for these.

#### Tokens

  A *token* type defines an actual operator or literal with several properties:

  - *Fixity:* The fixity of the token type--expression (no operands), infix (binary operator), postfix or prefix (unary operator before and after).
  - *Precedence:* The precedence of the operator (if it is an operator). This determines the *order of operations--which should happen first--when two adjacent operators are next to each other. For example, if `+` has precedence 8, and `*` has precedence 7. `1 + 2 * 3` is equivalent to `(1 + 2) * 3` and `1 * 2 + 3` is equivalent to `(1 * 2) + 3`. Prefix and postfix operators can have precedence as well: `- 2 * 3` is equivalent to `(-2) * 3` rather than `-(2 * 3)`. Lower precedence is tighter--when precedence 1 is next to precedence 2, precedence 1 always happens first.
  - *Associativity:* the associativity of an operator determines how to order when two operators with equal precedence. Associativity can be left or right.
  - *Require Next Operators:* Some operators, like parentheses `(1 + 2)` and the ternary operator `1 ? 2 : 3`, are incomplete until one or more "next" operators is found. This specifies what those operators (tokens) are. Everything to the right of the operator will be the right child of the operator when this happens (no precedence or associativity is checked).
  - *Kind:* Whether this is a comment, space or operation (operator/expression)

#### Comments

Comments are not added to the expression tree, but *are* attached to the expression next to them in the debug data. Because that expression could be before or after the associated expression, the tokenizer treat comments as having both prefix and postfix variants, and resolves them the same as `--` and `++` (which basically attach to the nearest expression, whether it is on the left or right).

Comments are *not* space. While they are not executed or compiled, comments *do* impact the reader's ability to scan and determine blocks. Therefore, comments can affect (and are affected by) indentation--in fact, they act just the same as a prefix or postfix operator would.

#### Whitespace and Newline

Plain whitespace symbols are not emitted by the tokenizer or saved into the tree or debug data (with the exception of certain newlines). Whitespace, like comments, can be thought of as having prefix / postfix variants; however, we generally 

Newlines indicate the beginning of a new statement or block, *unless* the lines are joined by an operator. Since we need to know what is on the next line before we can decide this, we treat it as normal space (with a treat can be normal space (if either side is an operator), or they can mean `extend` or `apply` (depending on indentation). We treat newline as an ambiguous symbol with prefix and postfix space variants, *and* an infix variant. The postfix variant is *preferred* over the infix variant, meaning we'd rather not break  where postfix variant is preferred (otherwise if the next line started with an ambiguous .  checks the current indentation, decides whether to `apply` or `extend`, and creates a unique token with the indent level embedded in it (as a number).


Stream
------

The stream actually reads the UTF-8 source, yielding Unicode characters. Invalid UTF-8 sequences are detected here, emitted as errors (treating the invalid sequence as a single character), and skipped. The stream keeps track of both the character index and the byte index into the source UTF-8. The stream is responsible for adding source line/column information to the debug data.

* **Codepoint:** We specify characters in terms of Unicode, and codepoints are single unicode values. A codepoint is what is typically thought of when one reads Unicode in most languages; however, a codepoint is only a *part* of a single character.
* **Character:** The definition of character is important. When we say character in this specification, we universally  mean a single printed character (in particular, a Unicode *default extended grapheme cluster* a la [http://www.unicode.org/reports/tr29/#Default_Grapheme_Cluster_Table](http://www.unicode.org/reports/tr29/#Conformance) or rules [in this chart](http://www.unicode.org/Public/10.0.0/ucd/auxiliary/GraphemeBreakTest.html#rules). NOTE that this means CRLF is a *single character*.

### Line Debug Data

The scanner writes out newlines. Any newline character (the `BK` class from [Unicode Line Breaking](http://www.unicode.org/reports/tr14/tr14-32.html#BK)) anywhere in the text causes the byte, codepoint and character index for the given line to be written out to debug data.

The specific list of characters as of the last reading:

```
 LF:    Line Feed, U+000A
 VT:    Vertical Tab, U+000B
 FF:    Form Feed, U+000C
 CR:    Carriage Return, U+000D
 CR+LF: CR (U+000D) followed by LF (U+000A)
 NEL:   Next Line, U+0085
 LS:    Line Separator, U+2028
 PS:    Paragraph Separator, U+2029
```

### Stream Errors

The stream outputs errors into the error list: error characters are yielded as U+FEFF (not a character).

### Scanner

The scanner reads characters from the stream based on a grammar, producing a symbol for each distinct "word" or operator.

#### Compound Terms

The scanner can take into account *local* context, such as surrounding space, when deciding what symbol to output. For example, compound term rules are implemented by looking at preceding and following space when there are many variants of a symbol, and emitting a symbol type with only the valid variants if it's at the beginning or end of a compound term:

- If there is leading space only, and the symbol has a prefix variant, return a symbol type infix variant.
- If there is leading space only, and the symbol has an expression variant, return a symbol type without the postfix variant.
- If there is trailing space only, and the symbol has a postfix variant, return a symbol type without the infix variant.
- If there is trailing space only, and the symbol has an expression variant, return a symbol type without the prefix variant.

#### Space

Spaces are not significant in Berg except for indentation and compound terms. Spaces are skipped in the scanner (except in strings, comments and indentation).

##### Indentation

The scanner *does* emit a special space symbol for indentation. When a new non-blank line is found, the spaces at the beginning of the line are considered indent. Non-blank line indent is defined as:

```
Non-Blank Line: (StartOfFile | Newline) Indent = Space* !(EndOfFile | Newline)
```

The indent symbol has infix, prefix and postfix variants so that looking at the indent will tell us how the previous and next lines start and end (with operators or operands). If the Newline is a Line Separator, no indent symbol is output. If it is a Paragraph Separator character, a Paragraph Indent symbol (infix-only) is output.

Indent symbols are used and stripped in the tokenizer, though in some cases they are replaced with `apply`, `extend` or `block` operators to join blocks together.

#### Comments

Comments are placed in the comment data section and not emitted as symbols. They will be given proper parents after parsing.

### Tokenizer

The *tokenizer* takes the stream of symbols, decides which variants to use (infix/prefix/postfix/expression) and inserts `extend`, `apply` and `block` at block and statement boundaries.

In general, Berg symbols are resolved with their *preferred* variant--if you say `(-2)`, the `-` will be prefix, and if you say `1-2` it will be infix.

#### Preferred Variant

Tokenization starts by reading through the symbols picking a preferred variant for each one.

1. We prefer an operand to start with (i.e. we would rather the entire file start with `1` or `x` than `/` or `||`).
2. We prefer the matching variant (infix or postfix if we prefer operator, prefix or expression if we prefer operand).
   - If there is 1 matching variant, we prefer that.
   - If there are 2 matching variants, the symbol decides which it prefers. Indent prefers postfix over infix, and `+` (as with most operators) prefers infix over postfix, for example.
   - If there are 0, we prefer nothing. (This is a hitch and and when it is repaired, we will repeat this with a preference for operator instead of operand, or vice versa.)
3. If the preferred variant is infix or prefix, we prefer the next symbol be an operand. If expression or postfix, we prefer operator.

#### Disambiguation

However, sometimes when we make a preferred choice, there is a "hitch" later on that shows it lead to a broken expression--one where an operator doesn't have an operand. In these cases, we resolve the hitch by picking a less preferred option. We do this in one of three ways, in order of priority:

1. Picking a non-preferred version of an ambiguous symbol (like `+`) as early as possible.
2. Inserting apply ("function call"--which bridges operand/operand gaps like `foo bar`) as early as possible.
3. Inserting "missing parameter" (to bridge operator/operator gaps like `1 / / 4`) as early as possible.

The disambiguation algorithm implements this. While the actual code is small, the algorithm requires careful explanation which we give here.

##### Hitches

If we reach an operator with *no* matching variant (e.g. `1` when we need an operator or `*` when we need an operand), we have found a *hitch*. We need the previous symbol to be the opposite (infix or prefix instead of expression or postfix, or vice versa) before we can proceed.

First, we make the switch:
- If the first symbol has 2 matching variants, we pick the non-preferred variant.
- Otherwise, we insert apply or missing parameter

Then, we output the opposite variant of each symbol:
- If we prefer operator, pick the preferred variant for *operand* and then pick the opposite variant of that (infix/expression are opposites, and prefix/postfix).

Then, we redo this symbol with the new "prefer operator" (which will be the opposite of what it was).

##### Safe Points

To allow us to parse incrementally and make the hitch-fixing algorithm simpler overall, we detect "safe" points where we guarantee that we will *not* insert or pick a non-preferred variant before this symbol, even if a hitch occurs.

The actual rules:

- **Unchangeable Outcome:**
  If a symbol has a preferred variant but *no opposite* (infix and expression are opposites, and prefix and postfix), everything to the left is safe to output.

  If a symbol has only 1 matching variant and no opposite, this symbol is safe as well. Note: some symbols with two variants--like `-`--are unambiguous by this definition.

- **Non-Preferred > Apply > Missing Parameter:**
  If a symbol has 2 matching variants, *and* we would insert apply/missing on a hitch, everything to the left is safe to output.

  If we prefer an operator, and we would insert missing on a hitch, everything to the left is safe to output.

To output safe symbols, we pick the preferred matching variant of each.

#### Comment Attachment

After outputting an infix or postfix token, we set the parent of any comments before it to the previous token. After outputting a prefix or expression token, we set the parent of those comments to the token we outputted. This allows doc comments to be attached to the correct things.

On close, we set the parent of any comments to the last token.

#### Block Recognition

Indents mean multiple things depending on whether they actually start an expression (or are just a continuation), and whether they are more or less indented than previous indents. Block recognition generates `extend`, `apply` or `block` operators depending on relative indent levels.

The generated block operators `extend`, `apply` and `block` *always* have lower precedence than any normal operator, and they all associate left. They are generated with a "nesting level" that corresponds to the size of the stack, and precedence between the operators depends entirely on nesting level. *All* blocks in the file are separated by these operators. These two facts mean that every expression in the Berg source (with the exception of any at the very top level) is inside the block operator to its left.

```
# === extend ===
# x = 1
# y = 2
# print x + y
#
# === apply ===
#
# myfunction
#     x: 1
#     y: 2
#     x + y
If infix indent
    Pop all > indent
    If indent == open indent
        If more than 2 popped, emit Multiple Undent Error
        Output `extend`
    Else
        If any popped, or Paragraph Indent, emit Misaligned Indent Error
        Output `apply`

# === continuation ===
# foobar:
#     CheckForFirstProblem
#     || CheckForSecondProblem
#     || CheckForThirdProblem
#
# foobar:
#        CheckForFirstProblem
#     || CheckForSecondProblem
#     || CheckForThirdProblem
#
# foobar:
#     CheckForFirstProblem
#         || CheckForSecondProblem
#         || CheckForThirdProblem
If postfix indent
    If previous token is indent, skip as continuation
    Pop until > parent open indent
    If any popped, emit Continuation Undent Error unless already errored
    Skip as continuation

# === block ===
# a:
#    b: 1
#    c: 2
#
# === continuation ===
# CheckForFirstProblem ||
#   CheckForSecondProblem ||
#   CheckForThirdProblem
#
# CheckForFirstProblem ||
# CheckForSecondProblem ||
# CheckForThirdProblem
If prefix indent
    If previous token starts a block (like ':' '(' or '{')
        If indent > open indent
            Output `block`
        Else
            Pop all >= indent
            Error: Output `missing parameter` token unless already errored
            Output `extend`
    Else
        Pop until > parent open indent
        If indent == open indent
            If previous token is indent, skip as continuation
            Error: Output `missing parameter` (or `redundant semicolon at end of statement` if previous token was `;`) token unless already errored
            Output `extend`
        Else
            If any popped
                If previous token is indent, Error: output Misaligned Indent
                Else Error: Undented Continuation Error unless already errored
            Skip as continuation
```

On close, we close all blocks simultaneously.

### Parser

The *parser* takes the stream of tokens and builds the syntax tree. It picks the correct expression hierarchy / order of operations using precedence and associativity.

#### Association

Association takes normal tokens and builds the syntax tree from them using normal precedence and association rules. It does not allow hierarchy outside the bounds of the current block, however.

```
If token is space: skip
Insert into syntax tree or comment subtree.
Insert into open tokens at correct place according to precedence & associativity, popping and setting parents as needed.
```

OTHER STUFF FOR PROBABLY THE FUTURE
==================================

Language Definition Files
-------------------------

The Berg language definitions are written in a strictly limited subset of the Berg language: that is, the bootstrap parser is a subset of the Berg parser. They are designed to be ASCII, readable in YAML, and easily parseable and interpretable with a custom parser. They have a small set of operators, are strictly limited in how lines are laid out, and call a very small set of Berg functions (to parse Unicode).

### Bootstrap Grammar

To read the grammar definitions, the bootstrap process must, at a minimum, support these types of lines (with 4-spaced indents and the exact spaces given, no more and no less). Blank lines will have no spaces in them.

```
[      first line] Grammar.Berg.Parser.SubgrammarDefinition
[        category]     Space|Expression|Binary|Suffix|Prefix:
[      blank line] 
[   token comment]         #[any character except \r or \n]
[token definition]         <Identifier>: <Expression>
```

Identifiers are case-sensitive English and alpha-only (a-z and A-Z)
Expressions are a sequence of subexpressions separated by ", " (comma and space). The only expression that will ever contain the "," character is the exact expression `","`, and it will not be in a sequence. It can be one of these three forms:

```
Expression:
[            comma only] ","
[subexpression sequence] <SubExpression>[, <SubExpression>]*

SubExpression:
[string] "<StringCharacter>+"
[string] "<StringCharacter>+"

SubExpression: " || <RegularExpression>
Expression: "\",\"" || 
        Newline/Space/Comment: <Expression>
        Space: <Expression>
            BareString: /[^"]+/

EscapeUnicodeGrammar:
    BergGrammar
        Suffix:
            HexDigits: /[0-9a-f]+/
            EscapeUnicodeEnd: "}"


RegularExpressionGrammar:
    BergGrammar
        Infix:
            CharacterSpecificationStart: "
```

The bootstrap parser will be sufficient to read in `Berg/Parser/Grammar.berg`, which can be used for the full parse.

NOTE: this grammar is in `Berg/Parser/Bootstrap/BootstrapGrammar.berg`. It will only ever change on major versions, and will try not to change then, either. It is designed to be parseable by a YAML parser, or a custom parser that counts leading spaces, splits on : and interprets surrounding "...", '...', /[0-9a-f]/.


    Characters: 
    Operators:
```

The bootstrap parser

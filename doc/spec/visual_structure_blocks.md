Blocks and Statements: The Visual Syntax of Berg
================================================

Berg source files are nested rectangular *blocks* of text. Anything indented to the right is a *nested* inside anything to the left. The statements on the margin of a block are a *sequence,* running from top to bottom.

```
:Person
    :Name
    :Greet(friend)
        Print "Hello #{friend.Name}"
    :ExchangeNames(friend)
        Print "I am #{Name}. What is yours?"
        Print "I am #{friend.Name}."
```

Key Goals
---------

Berg's visual structure is designed to be:

Scannable:::

  By scanning the outline of a Berg file, you can understand the structure and relationships before you have read a single word. This means you can reliably *orient* yourself to the structure and relationships in a Berg source file, before your conscious mind processes the actual code.

Efficient::

  Berg visual blocks and statements are designed for efficient parsing in several ways:
  
  * Blocks and expressions are *forward only*, creating great cache locality.
  * Blocks and expressions are *unambiguous*, eliminating expensive backtracking.
  * Blocks are *easily skippable* when they are not needed.
  * Language simplicity decreases the need for abstraction and with it, the need to allocate lots of small objects.
  * Reading-order precedence eliminates the need for most (but not all) parent pointers, decreasing the memory and cache burden of complex ASTs.
 
Replicable::

  To achieve tool-friendliness and smooth transition from other languages, Berg tries to make it easy to write the parser in a new language.
  
  * Expression lines being *independent* of each other makes the first level of parser design easy to imagine, decreasing the burden: read each line and attach to the previous.
  * Phrase boundaries allow the parser to decide which form of an operator to choose (postfix, infix or prefix) early, during lexing.
  * Operator "clumping" simplifies parsing expressions and eliminates the need to special-case long operators like `++` or `<=>`.
  * Between phrase boundaries and operator clumping, parsers can create a correct expression tree without having to know anything about what operators are actually supported, delaying that to later phases where more information is known.
  * Reading-order precedence eliminates an entire precedence algorithm that would otherwise need to be written.
  * Universal block rules restrict the number of actual decisions that have to be made.

Extensible::

  Because block containment is rigorously defined, we can allow extensible parsing. Extensible languages allowing embedded DSLs have a common problem: the DSLs tend to break visual flow, and moreover, since the DSL is essentially in charge of parsing, you have to trust it not to "break out of its cage" and start parsing everything else. Visual block structure cages the DSL in a scannable way, solving both problems.
  
  In fact, because Berg properties are lazy, parsing can even be incremental and partial: we can read most of a file as Berg expressions, and if some parts need to be processed by a DSL that hasn't been loaded yet (or may not ever be *needed*), they can be left as a text block.

  Compound operator parsing and universal precedence rules mean that extensions can define new operators in Berg modules, allowing operators to be actually interpreted during binding and typechecking when dependencies between modules come into play.

Accurate Errors::

  "Unclosed string" and "unclosed brace" errors plague many languages because there is no other way to determine the boundary of a block and therefore they cannot determine where you *meant* to put the closing brace, sometimes even until the end of the file. "Unexpected close brace" is a similar error with a similar issue.

  Berg's block boundaries prevent brace errors from spilling past the end of the block, and strings are always restricted either to a single line, or localized to a child block. Further, Berg's restriction that a line cannot have unclosed operators inside it makes most such errors *line-local*, unless the unclosed operator is at the beginning or end of the line. 

  This also incidentally makes Berg editor-friendly, in that while you are typing, errors you create will generally be local to the line or block you are in. This prevents some of the "serial refresh" issues that can happen where an open string causes the rest of the document to be recolored until you close the string, as well as keeping the errors where your eyes are.

Blocks
------

The base visual structural unit of Berg is the block. A block is a rectangular region of text: a set of lines with a margin, where all the text is on or after the margin. Blocks are in a hierarchy, where one block can have sub-blocks with deeper margins.

To see a block, look at a single line of code and draw a line from its first character both up and down, until it hits something. That is the *block* that line is in. Every single line with text on it is in a block.

Blank lines are treated as if they had the same indent as the next non-blank line. Comment-only lines *can* start or end a block.

NOTE: Different kinds of blocks may even have different content: comment blocks, string blocks, data blocks and code blocks, for example.

ERROR: It is illegal to indent or unindent more than one level at a time in Berg expressions: so an expression indented to 8 characters, followed by an expression indented to 16, followed by an expression indented to 12 characters, is illegal. Similarly, you cannot unindent more than one at a time (except at the end of the file), so if you need to unindent multiple levels, place a comment on each parent level you are closing.

Code Blocks
-----------

A *code block* is a block with a Berg expression inside. The visual structure groups code by margin (block/statement) and line, and separates groups on a line with spaces (phrases/terms).

### Units of structure

Code Block::

  A rectangular block of text. Contains a series of *statements* that run in order and build a block incrementally, each statement *extending* the block produced by the previous statement.

  Blank lines are considered a part of the *more indented* block before or after them. Comments are part of the current block.

Statement::

  A single expression across one or more lines, that acts as a unit. Lines correspond to visual lines in the input, and are separated by line separators (e.g. line feed).

  Continuation: Two subsequent lines are part of the same statement if they are joined by an operator, at the end of the first line or the beginning of the second.

  Indenting: If statement is followed by a child block, it is considered a part of the statement, either as an operand of the end operator, or *applied* to the result of the statement.

  Outdenting: If a block starts with a child block and then a line starting with an operator, the child block is the *left operand* of the operator.

  ERROR: A statement and child block followed by a line starting with an operator is an error.

  ERROR: A child block at the beginning of a block, followed by a line *without* an operator, is an error.

  NOTE: Statements also have *comments* associated with them. Blank lines and comments at the beginning or end of a block are part of the first and last statement, respectively. The presence of a comment at the end of a line does not change the line's meaning in any way. Comments between lines likewise do not change anything (for example, if a comment is between a line and a child block, the child is still part of the block.

Phrase::

  Whitespace-separated expressions in a statement, composed of *terms* separated by space and operators. Each term is run in order, along with the operators between them, to produce the phrase's result.

  Multiple phrases may be on a single line, in which case they are run in reverse order, each phrase *applied* to the previous phrase with the `apply` operator.

  Phrase breaks are recognizable as two non-operator terms in a single statement, separated by space, comments and newlines.

Term::

  "Clumped" expressions on a line, composed of *atomics* separated by operators, with no spaces between them. An operator may be at the beginning or end of a term, in which case it is considered *prefix* or *postfix*, respectively.

Atomic::

  A single indivisible expression on a line. Includes:

  - An identifier (`Foo`)
  - A numeric literal (`123`, `0xDEADBEEF` or `2.0e-10i`)
  - A delimited expression (i.e. `(...)`, `{...}`, `[...]` or `"..."`)
  - A function call (like `(f)x` or `f(x)`)--any pair of atomics right after each other. The argument is applied using *apply*.

FAQ
---

**Will this surprise C++, Java or C# programmers?**

We don't think so. In fact, we recommend an experiment. Take code you have written, and change nothing except removing semicolons from the end of a line, and remove braces from multiline blocks. You will almost certainly find that Berg's rules match yours.

We've picked some random examples from the most popular open source repositories on Github (as measured by [Git Most Wanted](http://gitmostwanted.com/)), and it checks out:

TODO do this.

### Semicolons and Brackets

**Why not mandatory semicolons?**

Berg's principle of *low contextual overhead* includes syntax that isn't *necessary* to write readable code. Even in code with semicolons Indentation and newlines are necessary to write readable code.

**Wouldn't semicolons make it more obvious where a statement ends, though?**

Because we already follow this style in the real world, we just add semicolons and braces to it :) In the real world, the next line after a statement is always at the same level or before it. Continuations always happen when the next line starts with an operator, and when the 

**Why can't I use open brackets and semicolons the way I usually do?**

Berg does support brackets and semicolons, and if you used brackets at indent/undent and semicolons at statement breaks, everything would work out exactly the same as it does now. Semicolons and brackets are for *clarity*--for situations where you want small blocks as function arguments or as part of an expression, or when a single operation is really 2-3 quick statements that look clearer on one line.

Berg disallows this because having a consistent visual style helps people tell when it is looking at Berg code--it becomes a sort of "mark" that cues your brain. The semicolons make you have to wonder if you're really looking at something else.

**But what if it's just for my code?**

All code ends up being example code, whether for coworkers or others. People who learn from your code would teach others those habits and rob them of the benefits of the lower overhead.

Berg strives as few purely stylistic choices as possible, and only where they will have a huge impact. Disallowing redundant semicolons and brackets is one of them.

 it's clearer to put three super small ideas on the same line (like a variable swap perhaps)., or using formatters to create minified or obfuscated code. They 

**Won't this be surprising?**

We actually think it will be *less* surprising. Berg accepts most normal nesting, 
for very quick *outline scanning*: your brain is very good at seeing the visual relationships between things by looking at the outline. Particularly, Berg  and scannability rests on *outline scanning*: A key design point for is *outline scanning*: to Berg's scannability is that lines are *closed* and their relationship is 100% determined by their left side (the indent and the presence or absence of an operator on the left). This means 

A key point for scannability is that lines are *closed* and their relationship is 100% defined by the operators on the right and left sides. You can scan the outline and know how the Berg program is structured without knowing anything at all. Open parentheses in the middle of the statement breaks that.


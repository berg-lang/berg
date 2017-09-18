Language Comparisons
====================

Language Choices
----------------

The languages of choice boil down to three categories: proven industry languages, popular indie languages, and non-programming languages. The third category is probably the most surprising--but I feel we have some things to learn from people who have tried to write other text document formats that people could actually read and write.

| Language   | Category | Specification 
|------------|----------|---------------
| JavaScript | Industry | [8.0, June 2017](https://www.ecma-international.org/ecma-262/8.0/index.html)
| Python     | Industry | [3.6.2, Dec 2016](https://docs.python.org/3.6/reference/index.html)
| Java       | Industry |
| C#         | Industry |
| C++        | Industry |
| PHP        | Industry |
| Swift      | Indie    | [4.0, June 2016](https://developer.apple.com/library/content/documentation/Swift/Conceptual/Swift_Programming_Language/AboutTheLanguageReference.html#//apple_ref/doc/uid/TP40014097-CH29-ID345)
| Go         | Indie    | [3.2, Nov 2016](https://golang.org/ref/spec)
| Rust       | Indie    |
| Elixir     | Indie    |
| Scala      | Indie    |
| Haskell    | Indie    |
| YAML       | Doc      | [1.2, Oct 2009](http://yaml.org/spec/1.2/spec.html)
| Markdown   | Doc      | [CommonMark 0.27, June 2016](http://spec.commonmark.org/0.27/)
| RST        | Doc      |
| JSON       | Doc      |
| HTML       | Doc      |
| CSS        | Doc      |
| TeX        | Doc      |

Unicode
-------

### Encoding

| Language | Character Set | Default | Detection   | Encoding Header
|----------|---------------|---------|-------------|---------------------------------
| Go       | Unicode       | UTF-8   | UTF-8 BOM   |
| Python   | Unicode       | UTF-8   | UTF-8 BOM   | Emacs / VIM *coding regex" in first 2 lines
| YAML     | Unicode [any] |         | BOM / ASCII |

### Allowed Characters

| Language | Allowed         | Disallowed | Special 
|----------|-----------------|------------|------------------------------------------
| Python   | *               |            | End of file must be newline
| Go       | *               | NULL       | BOM ignored
| Markdown | *               |            | NULL replaced with REPLACEMENT CHARACTER
| YAML     | * TAB LF CR NEL | Non-printable: C0 (U+0-1F) DEL C1 (U+80-9F) Surrogate ()U+D800-DFFF) U+FFFE-FFFF       | First character must be BOM or ASCII

Whitespace
----------

### Whitespace

| Language    | Horizontal                  | Newline          | Tab | Significance    |
|-------------|-----------------------------|------------------|-----|-----------------|
| JavaScript  | SP TAB FF VT Zs NBSP ZWNBSP | CRLF LF CR LS PS | N/A | Term Separation |
| Python      | SP TAB FF                   | CRLF LF CR       | 8   | Statement Separation, Indentation
| Swift       | SP TAB FF VT NULL           | CRLF? LF CR      | N/A | Statement Separation, Compound Terms
| Go          | SP TAB CR                   | LF               | N/A | 
| Markdown    | SP TAB FF VT Zs             | CRLF LF CR       | 4   | Term Separation, Statement Separation, Block Separation, Indentation |
| YAML        | SP TAB                      | CRLF LF CR       | N/A | Term Separation, Statement Separation, Block Separation, Indentation |

### Newline Effect

| Language   | Newline Statement Separation                | Blank Line | Newline Effect Otherwise
|------------|---------------------------------------------|------------|--------------------------|
| Berg       | If indent lies on block margin              | None       | Continue
| Javascript | If lines joined by operator                 | None       | Continue
| Python     | Always except in () or [], or \ at line end | None       | Continue
| Swift      | Always except if second line starts with operator | None       | Continue
| Go         | After identifier/literal/postfix            | None       | Continue
| Markdown   | If next line begins with new construct      | Block Separator | Space
| YAML       | If next line begins with new construct      | None       | LF

### Indentation and Nesting

| Language   | Start              | Characters  | Tab Stop | Blank Lines | Notes
|------------|--------------------|-------------|----------|-------------|-------------------
| Javascript | N/A
| Python     | First line after : | NSP TAB FF  | 8        | Ignored     | FF, then tabs, then spaces at beginning of line. FF in rest of line undefined. Continuations can be outside the indent. Indent must be a multiple of 4.
| Swift      | N/A
| Go         | N/A
| Markdown   | ?                  | SP TAB ??? | 4        | Ignored     |
| YAML       | ?                  | SP         | N/A      | Ignored (? Don't see mention in spec, but yamllint.com ignores) |

Comments
--------

| Language    | Single Line | Multiline | Characters  | Escape |
|-------------|-------------|-----------|-------------|--------|
| JavaScript  | //          | /* */     | *           | No     |
| YAML        | #           |           | *           | No     |

### Fenced Blocks

| Language    | Start     | Indent |
|-------------|-----------|--------|
| Markdown    | ---+ ===+ | ?      |

Identifiers
-----------

### Identifier Characters

| Language   | Characters      | First  | Last | Escape         | Normalization
|------------|-----------------|--------|------|----------------|---------------
| JavaScript | ID ZWNJ ZWJ $ _ | minus ZWNJ ZWJ, digits |      | \XXXX, \{XXXX} | Case-significant, normalized by parser. Not sure if NFC or NKFC???
| Python     | XID _           |        |      |                | NKFC, normalized by parser
| Swift      | A-Z a-z 0-9 _   | minus digits/combining |      | `reserved` |
| Go         | Lu Ll Lt Lm Lo _ Nd | minus Nd
| Markdown   | N/A
| YAML       | A-Z a-z 0-9 -   |       |      | None |

Strings
-------

### Quoted Strings

| Language | Characters                | Escapes
|----------|---------------------------|---------
| YAML     | All except U+0-8, U+10-1F | \0 \a \b \t \<TAB> \n \v \f \r \e \<SP> \" \/ \\ \N \_ \L \P \xXX \uXXXX \UXXXXXXXX

### Scalar String Characters

| Language | Characters                | Newline         |
|----------|---------------------------|-----------------|
| YAML     |                           | Translate to \n |




Editors:
- All: Atom, Sublime Text, UltraEdit, Notepad++, BBEdit, Bluefish, TextPad, vim, emacs
- Windows: Notepad, Notepad++, Notepad2
- Unix: gedit, Nano
- Mac: TextEdit

- notepad
- vim
- Emacs
- Atom
- Textpad
- Visual Studio Code

Sources for Popularity:
- [bynext.com - 1 person - 2016/08/20](http://www.bynext.com/2016/08/20/the-11-best-code-editors-available-in-2017/): Atom, Sublime Text, UltraEdit, Notepad++, Coffeecup?, BBEdit, Bluefish, Brackets, Coda, ICEDeveloper, CodeRunner 2
- [codeanywhere.com - 600 people - 2015/01/13](https://blog.codeanywhere.com/most-popular-ides-code-editors/): Notepad++, Sublime Text, Eclipse, NetBeans, IntelliJ, Vim, Visual Studio, PhpStorm, Atom, Emacs, Zend
- [webpagefx.com 2009/03/27](https://www.webpagefx.com/blog/web-design/the-15-most-popular-text-editors-for-developers/): Notepad++, TextMate, Coda, Vim, PSPad, Aptana, Komodo, Dreamweaver, UltraEdit, TextPad, gedit, GNU Emacs, E, EditPlus, SciTE
- [slant.co - 2000 people](https://www.slant.co/topics/12/~best-programming-text-editors): Vim, Spacemacs, Neovim, Sublime Text, Visual Studio Code, Atom, Emacs, Notepad++, Geany, Brackets, SciTE, Gedit, BBedit, MacVim, TextMate, Komodo Edit, SlickEdit, WebStorm, PSPad, UltraEdit
- [sitepoint.com survey - 100 Pythonists - 2015/03/04](https://www.sitepoint.com/which-code-editors-do-pythonists-use/): Sublime Text, Vim, Emacs, Notepad++, TextWrangler, IDLE, Atom, Aquamacs, GNU Nano, Kate, gedit

The list of document editors:

- WordPad
- Word
- PowerPoint
- Excel
- Browser
- OpenOffice

The list of IDEs:

- Visual Studio
- Visual Studio Code
- Xcode

The list of visualizers:

- Windows shell
- Unix shell

The list of debuggers:

- gdb

### Languages


Popular languages:

- [stackify.com - 2017/06/22](https://stackify.com/trendiest-programming-languages-hottest-sought-programming-languages-2017/):
    - Github Active Repositories:
        - 75%: JavaScript
        - 50%: Java, Python, CSS
        - 25%: PHP, Ruby, C++
        - 00%: C, Shell, C#, Objective-C, R, VimL, Go, Perl, CoffeeScript, TeX, Swift, Scala, Emacs Lisp
    - Github Most Used:
        - 75%: JavaScript
        - 50%: Java, CSS
        - 25%: Python, PHP, Ruby, C++
        - 00%: C, Shell, C#, Objective-C, Go, R, TeX, VimL, Perl, Scala, CoffeeScript, Emacs Lisp, Swift
    - Job Listings:
        - 75%: Java, JavaScript
        - 50%: PHP, CSS, SQL
        - 25%: C#
        - 00%: C++, Python, C, Shell
    - StackOverflow Most-Loved:
        - 75%: Rust, Swift, F#, Scala, Go, Clojure, React, Haskell, Python, C#
        - 50%: Node.js
    - StackOverflow Most-Popular:
        - 75%: JavaScript, SQL
        - 50%: Java, C#
        - 25%: PHP, Python, C++, C, Node.js, AngularJS
        - 00%: Objective-C, Ruby
    - StackOverflow Top Tech:
        - 75%: JavaScript, Java
        - 50%: Android, Python, C#, PHP
        - 25%: JQuery, C++, HTML, iOS, CSS
    - Usersnap Recommendations: JavaScript, Java, Python, Elixir, Rust, Go, TypeScript, PHP, Ruby on Rails, C#, Swift
    - SimpleProgrammer.com Recommendations: JavaScript, Python, Elixir, Rust, Swift
    - Search Volume:
        - 75%: C, Go
        - 50%: Swift, R, Rust
        - 25%: Python, Java, C++, Ruby
        - 00%: Scala, PHP, Perl, C#, Visual Basic, JavaScript, CSS, Clojure, Objective-C, Shell, Powershell, TeX
    - Recommendations: JavaScript, Java, Python, Ruby, CSS, R

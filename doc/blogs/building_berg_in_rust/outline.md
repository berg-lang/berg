Building Berg In Rust
=====================

Intro: Goals. Basics of language syntax and why. Why rust?

Day 1: Learning that stealing is OK. Wrote a shell that just tried to read a file, with some of the abstractions I wanted; started out passing references around, discovered quickly I had to litter the program with lifetimes. Then I realized I should be transferring ownership clearly instead, and everything got magically nicer.

Day 2: Writing a lexer. Wrote a lot of the abstractions I needed for the syntax tree, and a simple int/bareword/operator parser. Plugging that in now ...

Day 2.5: Discovering that Rust used to have structural typing. Maybe I am on the right track! https://www.youtube.com/watch?v=olbTX95hdbg

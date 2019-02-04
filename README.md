# Berg

Berg is a practical declarative programming language for distributed systems. It focuses on ease of adoption, safety, concurrency and deployability (in roughly that order).

The language syntax *enables smooth adoption*, *minimizes cognitive load*, and strives to be *correct by default.*

Its major features include:

* Structurally typed: objects with the right properties can be used *anywhere.*
* Optionally typed: types are optional everywhere (inferred from caller context), enabling a smooth ramp from wild, rapid prototyping to well-specified, self-documented production code.
* Implicit async/await: fundamentally lazy property access works just like async/await, making declarative programs much easier to write.
* Compiled: Berg can be compiled to native code and packaged as a single executable.
* Prototype inheritance: Objects inherit from other objects, allowing for things like "partial classes" and curried functions.
* First class types, modules and functions: Everything in Berg is first class, allowing powerful dependency injection.
* Actor model: Berg processes are "actors," with message passing mediated by lazy properties
* Ownership model: Actors own their memory but can transfer ownership to other actors through messages.
* RAII/Lifetime managed memory: Berg is not garbage collected;
* Dependency injectable: Any module dependency can be overridden or removed, allowing for limitless sandboxing and test dependency injection.
* Extensible: All language constructs, including operators, parsing and even execution, are written in Berg and overrideable. DSLs can be written either with or without parsing.

## Goals

A huge list of design goals:

* Easy to pick up and use for Java, C++, Javascript, Ruby, and Python users.
* Prototyping with safety: Ceremony such as types are inferred by default. Berg is strongly typed, you don't write types until you need them for performance or interoperability with external systems.
* Smooth ramp to production: Explicit ceremony (such as typing) is **allowed** and encouraged as you ramp towards production, to reduce compile times and restrict library contracts.
* Designed for the network: lazy language constructs allow for long-running external calls. Timeouts, retries, and exponential backoff are easy to implement.
* Designed to avoid bottlenecking: Distributed systems reliability requires backpressure and queueing. These are easy to build in.
* Designed for versioning: The ability to work with multiple versions of APIs is facilitated greatly by the language and runtime.
* Designed for heterogeneous data sources: Parsing is built-in, fast and easy. Joining data between multiple data sources can be done efficiently with clear code.
* Designed for operability: Berg processes can access shared information about the environment it is in, from the host all the way up to service-specific data such as master, database location, etc.
* Designed for security: secrets management and rotation, server and client certificates, and are designed into APIs from day one.
* Designed for CD: a flexible testing framework
* Designed for self-host: Berg services can be brought up local or remote, and have good integration with service discovery to enable this.
* Simple install: Berg services can be copied to a target environment and started.
* Serviceable: Berg services have strong hooks to packaging and service running systems.
* Designed for scale: Berg has a pervasive ability to emit telemetry about latency, throughput and error rate
* Context-sensitive: Berg removes the need for top-level static objects like a Logger class, with first class support for context.
* Gradually adoptable: works will within, and containing, other languages.

* Has , designed for development speed, clarity and safety. It has a proactively lazy actor-model runtime.

declarative programming language and proactively lazy actor-model runtime for distributed systems focused on development speed and clarity with runtime safety.

## Principles

- Low Conceptual Overhead: the language and runtime minimizes as far as possible the number of concepts you encounter when reading or writing code. This principle applies at any scale, be it a line, block, file, module or program. It means the runtime model has only one kind of block (ultimately),
- Natural For Imperative/Object-Oriented Programmers: People coming from nearly any mainstream language are doing imperative, and usually object-oriented, programming.
- Clear, Safe, Performant By Default: **The easiest thing to write should do the right thing.** You should not have to learn extra concepts or add extra modifiers to your code to make something clearer, safer, or more performant.
- High Knowledge Transfer: Nobody works in just one language anymore, and nobody has time to learn a whole system. Anywhere possible, Berg borrows syntax and semantics from other languages and runtimes to reduce the cost of context switching between languages.
- Low Surprise: **The easiest thing to do does not cause bad surprises.** You *should* have to learn extra concepts and add modifiers before you can shoot yourself in the foot in unexpected ways.
- Low Controversy: Berg strives for as few controversial topics as possible
- Smooth Entry Ramp: Writing your first program should be as fast as possible.
- Scannability: There should be enough markers to visually find pieces of the code you're looking for.
- Debuggable: The compiler and runtime *must* emit humane error messages that give useful information on the point of the error.
- Speedy Development Cycle:

Developer Getting Started
-------------------------

1. Install Rust. On Windows, [install rustup.exe](https://win.rustup.rs/).
2. Go to a command prompt, and run:

```
cargo build
```


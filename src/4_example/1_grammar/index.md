# Compiling Grammars and Parsing

First we will implement compilation of pest grammars, and parsing text with a compiled grammar.
A [pest grammar](https://pest.rs/book/grammars/peg.html) contains named rules that describe how to parse something.
For example, `number = { ASCII_DIGIT+ }` means that a `number` is parsed by parsing 1 or more `ASCII_DIGIT`, with `ASCII_DIGIT` being a builtin rule that parses ASCII numbers 0-9.

Add the following dev-dependencies to `pie/Cargo.toml`:

```diff2html linebyline
{{#include ../../gen/4_example/1_grammar/a_1_Cargo.toml.diff}}
```

- [pest](https://crates.io/crates/pest) is the library for parsing with pest grammars.
- [pest_meta](https://crates.io/crates/pest_meta) validates, optimises, and compiles pest grammars.
- [pest_vm](https://crates.io/crates/pest_vm) provides parsing with a compiled pest grammar, without having to generate Rust code for grammars, enabling interactive use.

Create the `pie/examples/parser_dev/main.rs` file and add an empty main function to it:

```rust,
{{#include a_2_main.rs}}
```

Confirm the example can be run with `cargo run --example parser_dev`.

Let's implement the pest grammar compiler and parser.
Add `parse` as a public module to `pie/examples/parser_dev/main.rs`:

```diff2html linebyline
{{#include ../../gen/4_example/1_grammar/a_3_main_parse_mod.rs.diff}}
```

We will add larger chunks of code from now on, compared to the rest of the tutorial, to keep things going.
Create the `pie/examples/parser_dev/parse.rs` file and add to it:

```rust,
{{#include a_4_grammar.rs}}
```

The `CompiledGrammar` struct contains a parsed pest grammar, consisting of a `Vec` of optimised parsing rules, and a hash set of rule names.
We will use this struct as an output of a task in the future, so we derive `Clone`, `Eq`, and `Debug`.

The `new` function takes text of a pest grammar, and an optional file path for error reporting, and creates a `CompilerGrammar` or an error in the form of a `String`.
We're using `String`s as errors in this example for simplicity.

We compile the grammar with `pest_meta::parse_and_optimize`.
If successful, we gather the rule names into a hash set and return a `CompiledGrammar`.
If not, multiple errors are returned, which are first preprocessed with `with_path` and `renamed_rules`, and then written to a single String with `writeln!`, which is returned as the error.

Now we implement parsing using a `CompiledGrammar`.
Add the `parse` method to `pie/examples/parser_dev/parse.rs`:

```diff2html linebyline
{{#include ../../gen/4_example/1_grammar/a_5_parse.rs.diff}}
```

`parse` takes the text of the program to parse, the rule name to start parsing with, and an optional file path for error reporting.

We first check whether `rule_name` exists by looking for it in `self.rule_names`, and return an error if it does not exist.
We have to do this because `pest_vm` panics when the rule name does not exist, which would kill the entire program.

If the rule name is valid, we create a `pest_vm::Vm` and `parse`.
If successful, we get a `pairs` iterator that describes how the program was parsed, which are typically used to [create an Abstract Syntax Tree (AST) in Rust code](https://pest.rs/book/examples/json.html#ast-generation).
However, for simplicity we just format the pairs as a `String` and return that.
If not successful, we do the same as the previous function, but instead for 1 error instead of multiple.

Unfortunately we cannot store `pest_vm::Vm` in `CompiledGrammar`, because `Vm` does not implement `Clone` nor `Eq`.
Therefore, we have to create a new `Vm` every time we parse, which has a small performance overhead, but that is fine for this example.

To check whether this code does what we want, we'll write a test for it (yes, you can add tests to examples in Rust!).
Add to `pie/examples/parser_dev/parse.rs`:

```rust,
{{#include a_6_test.rs:2:}}
```

We test grammar compilation failure and success, and parse failure and success.
Run this test with `cargo test --example parser_dev -- --show-output`, which also shows what the returned `String`s look like.

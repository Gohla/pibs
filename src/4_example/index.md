# Example: Interactive Parser Development

To demonstrate what can be done with the programmatic incremental build system we just created, we will create a simple "interactive parser development" example.
In this example, we can interactively develop a grammar for a new (programming) language, and test that grammar against several example files written in the new language.

We will use [pest](https://pest.rs/) as the parser framework, because it is written in Rust and can be easily embedded into an application.
Pest uses Parsing Expression Grammars (PEGs) which are easy to understand, which is also good for this example.

For the GUI, we will use [Ratatui](https://ratatui.rs/), which is a cross-platform terminal GUI framework, along with [tui-textarea](https://github.com/rhysd/tui-textarea) for a text editor widget.
We could use a more featured GUI framework like [egui](https://github.com/emilk/egui), but for this example we'll keep it simple and runnable in a terminal.

We will continue as follows:

1) Implement compilation of pest grammars and parsing of text with the compiled grammar.
2) Create tasks for grammar compilation and parsing.
3) Parse CLI arguments and run these tasks in a non-interactive setting.
4) Create a terminal GUI for interactive parser development.

## Compiling grammars and parsing

First we will implement compilation of pest grammars, and parsing text with a compiled grammar.
A [pest grammar](https://pest.rs/book/grammars/peg.html) contains named rules that describe how to parse something.
For example, `number = { ASCII_DIGIT+ }` means that a `number` is parsed by parsing 1 or more `ASCII_DIGIT`, with `ASCII_DIGIT` being a builtin rule that parses ASCII numbers 0-9.

Add the following dev-dependencies to `pie/Cargo.toml`:

```diff2html linebyline
{{#include ../gen/4_example/a_1_Cargo.toml.diff}}
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
{{#include ../gen/4_example/a_3_main_parse_mod.rs.diff}}
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
{{#include ../gen/4_example/a_5_parse.rs.diff}}
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

## Tasks

Now we'll implement tasks for compiling a grammar and parsing.
Add `task` as a public module to `pie/examples/parser_dev/main.rs`:

```diff2html linebyline
{{#include ../gen/4_example/b_1_main_task_mod.rs.diff}}
```

Create the `pie/examples/parser_dev/task.rs` file and add to it:

```rust,
{{#include b_2_tasks_outputs.rs}}
```

We create a `Tasks` enum with:

- A `CompileGrammar` variant for compiling a grammar from a file.
- A `Parse` variant that uses the compiled grammar returned from another task to parse a program in a file, starting parsing with a specific rule given by name.

`compile_grammar` and `parse` are convenience functions for creating these variants.
We derive `Clone`, `Eq`, `Hash` and `Debug` as these are required for tasks.

We create an `Outputs` enum for storing the results of these tasks, and derive the required traits.

Since both tasks will require a file, and we're using `String`s as errors, we will implement a convenience function for this.
Add to `pie/examples/parser_dev/task.rs`:

```rust,
{{#include b_3_require_file.rs:2:}}
```

`require_file_to_string` is like `context.require_file`, but converts all errors to `String`.

Now we implement `Task` for `Tasks`.
Add to `pie/examples/parser_dev/task.rs`:

```rust,
{{#include b_4_task.rs:2:}}
```

The output is `Result<Outputs, String>`: either an `Outputs` if the task succeeds, or a `String` if not.
In `execute` we match our variant and either compile a grammar or parse, which are mostly straightforward.
In the `Parse` variant, we require the compile grammar task, but don't propagate its errors and instead return `Ok(Outputs::Parsed(None))`.
We do this to prevent duplicate errors.
If we propagated the error, the grammar compilation error would be duplicated into every parse task.

Confirm the code compiles with `cargo build --example parser_dev`.
We won't test this code as we'll use these tasks in the `main` function next.

## Parse CLI arguments

We have tasks for compiling grammars and parsing files, but we need to pass file paths and a rule name into these tasks.
We will pass this data to the program via command-line arguments.
To parse command-line arguments, we will use [clap](https://docs.rs/clap/latest/clap/), which is an awesome library for easily parsing command-line arguments.
Add clap as a dependency to `pie/Cargo.toml`:

```diff2html linebyline
{{#include ../gen/4_example/c_1_Cargo.toml.diff}}
```

We're using the `derive` feature of clap to automatically derive a full-featured argument parser from a struct.
Modify `pie/examples/parser_dev/main.rs`:

```diff2html
{{#include ../gen/4_example/c_2_cli.rs.diff}}
```

The `Args` struct contains exactly the data we need: the path to the grammar file, the name of the rule to start parsing with, and paths to program files to parse.
We derive an argument parser for `Args` with `#[derive(Parser)]`.
Then we parse command-line arguments in `main` with `Args::parse()`.

Test this program with `cargo run --example parser_dev -- --help`, which should result in usage help for the program.
Note that the names, ordering, and doc-comments of the fields are used to generate this help.
You can test out several more commands:

- `cargo run --example parser_dev --`
- `cargo run --example parser_dev -- foo`
- `cargo run --example parser_dev -- foo bar`
- `cargo run --example parser_dev -- foo bar baz qux`

Now let's use these arguments to actually compile the grammar and parse example program files.
Modify `pie/examples/parser_dev/main.rs`:

```diff2html
{{#include ../gen/4_example/c_3_compile_parse.rs.diff}}
```

In `compile_grammar_and_parse`, we create a new `Pie` instance that writes the build log to stderr, and create a new build session.
Then, we require a compile grammar task using the `grammar_file_path` from `Args`, and write any errors to the `errors` `String`.
We then require a parse task for every path in `args.program_file_paths`, using the previously created `compile_grammar_task` and `args.rule_name`.
Successes are printed to stdout and errors are written to `errors`.
Finally, we print `errors` to stdout if there are any.

To test this out, we need a grammar and some test files. Create `grammar.pest`:

```
{{#include c_4_grammar.pest}}
```

```admonish info title="Pest Grammars"
It's not important for this example to understand pest grammars, but I will explain the basics of this grammar.
Feel free to learn and experiment more if you are interested.

This grammar parses numbers with the `num` rule.
The `main` rule ensures that there is no additional text before and after a `num` rule, using [`SOI` (start of input) `EOI` (end of input)](https://pest.rs/book/grammars/syntax.html#start-and-end-of-input), and using the [`~` operator to sequence](https://pest.rs/book/grammars/syntax.html#sequence) these rules.
We set the [`WHITESPACE` builtin rule](https://pest.rs/book/grammars/syntax.html#implicit-whitespace) to `{ " " | "\t" | "\n" | "\r" }` so that spaces, tabs, newlines, and carriage return characters are implicitly allowed between returns.
The `_` operator before `{` indicates that it is a [silent rule](https://pest.rs/book/grammars/syntax.html#silent) that does not contribute to the parse result.
```

Create `test_1.txt` with:

```
{{#include c_4_test_1.txt}}
```

And create `test_2.txt` with:

```
{{#include c_4_test_2.txt}}
```

Run the program with `cargo run --example parser_dev -- grammar.pest main test_1.txt test_2.txt`.
This should result in a build log showing that the grammar is successfully compiled, that one file is successfully parsed, and that one file has a parse error.

```admonish tip title="Hiding the Build Log"
If you are using a bash-like shell on a UNIX-like OS, you can hide the build log by redirecting stderr to `/dev/null` with: `cargo run --example parser_dev -- grammar.pest main test_1.txt test_2.txt 2>/dev/null`.
Otherwise, you can hide the build log by replacing `WritingTracker::with_stderr()` with `NoopTracker`.
```

```admonish note title="No Incrementality?" collapsible=true
Unfortunately, there is no incrementality between different runs of the example, because the `Store` is not persisted.
The `Store` only exists in-memory while the program is running, and is then thrown away.
Thus, there cannot be any incrementality.

To get incrementality, we need to serialize the `Store` before the program exits, and deserialize it when the program starts.
This is possible and not actually that hard, I just never got around to implementing it in this tutorial.
If you want some pointers to implement serialization for your PIE implementation, read on.

The de-facto standard (and awesome) serialization library in Rust in [serde](https://serde.rs/).
See the [PIE in Rust repository at the `pre_type_refactor` tag](https://github.com/Gohla/pie/blob/pre_type_refactor/pie/) for a version of PIE with serde serialization.
For example, the [`Store`](https://github.com/Gohla/pie/blob/pre_type_refactor/pie/src/store.rs#L14-L17) struct has annotations for deriving `serde::Deserialize` and `serde::Serialize`.
These attributes are somewhat convoluted due to serialization being optional, and due to the `H` generic type parameter which should not be included into serialization bounds.

You should derive `serde::Deserialize` and `serde::Serialize` for all required types in the PIE library, but also all tasks, and all task outputs.
The `pie_graph` library support serialization when the `serde` feature is enabled, which is enabled by default.
Then, see [this serialization integration test](https://github.com/Gohla/pie/blob/pre_type_refactor/pie/tests/serde.rs).
```

Feel free to experiment with the grammar, example files, etc. before continuing!

## Editor

```admonish warning title="Under Construction"
This subsection is under construction.
```

[//]: # (add dev-deps)

[//]: # ()
[//]: # (create `pie/examples/parser_dev/editor.rs`)

[//]: # ()
[//]: # (`Editor` `new` `draw_and_process_event` `run`)

[//]: # ()
[//]: # (add editor arg to `Cli` and run editor instead)

[//]: # ()
[//]: # (run)

[//]: # ()
[//]: # (`Buffer`)

[//]: # ()
[//]: # (create buffers in `new`)

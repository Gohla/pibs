# CLI for Incremental Batch Builds

We have tasks for compiling grammars and parsing files, but we need to pass file paths and a rule name into these tasks.
We will pass this data to the program via command-line arguments.
To parse command-line arguments, we will use [clap](https://docs.rs/clap/latest/clap/), which is an awesome library for easily parsing command-line arguments.
Add clap as a dependency to `pie/Cargo.toml`:

```diff2html linebyline
{{#include ../../gen/4_example/3_cli/c_1_Cargo.toml.diff}}
```

We're using the `derive` feature of clap to automatically derive a full-featured argument parser from a struct.
Modify `pie/examples/parser_dev/main.rs`:

```diff2html
{{#include ../../gen/4_example/3_cli/c_2_cli.rs.diff}}
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
{{#include ../../gen/4_example/3_cli/c_3_compile_parse.rs.diff}}
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
You don't need to fully understand pest grammars to finish this example.
However, I will explain the basics of this grammar here.
Feel free to learn and experiment more if you are interested.

Grammars are [lists of rules](https://pest.rs/book/grammars/syntax.html#syntax-of-pest-grammars), such as `num` and `main`.
This grammar parses numbers with the `num` rule, matching 1 or more `ASCII_DIGIT` with [repetition](https://pest.rs/book/grammars/syntax.html#repetition).

The `main` rule ensures that there is no additional text before and after a `num` rule, using [`SOI` (start of input) `EOI` (end of input)](https://pest.rs/book/grammars/syntax.html#start-and-end-of-input), and using the [`~` operator to sequence](https://pest.rs/book/grammars/syntax.html#sequence) these rules.

We set the [`WHITESPACE` builtin rule](https://pest.rs/book/grammars/syntax.html#implicit-whitespace) to `{ " " | "\t" | "\n" | "\r" }` so that spaces, tabs, newlines, and carriage return characters are implicitly allowed between sequenced rules.
The `@` operator before `{` indicates that it is an [atomic rule](https://pest.rs/book/grammars/syntax.html#atomic), disallowing implicit whitespace.
We want this on the `num` rule so that we can't add spaces in between digits of a number (try removing it and see!)

The `_` operator before `{` indicates that it is a [silent rule](https://pest.rs/book/grammars/syntax.html#silent) that does not contribute to the parse result.
This is important when processing the parse result into an [Abstract Syntax Tree (AST)](https://pest.rs/book/examples/json.html#ast-generation).
In this example we just print the parse result, so silent rules are not really needed, but I included it for completeness.
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

Unfortunately, there is no incrementality between different runs of the example, because the `Pie` `Store` is not persisted.
The `Store` only exists in-memory while the program is running, and is then thrown away.
Thus, there cannot be any incrementality.
To get incrementality, we need to serialize the `Store` before the program exits, and deserialize it when the program starts.
This is possible and not actually that hard, I just never got around to explaining it in this tutorial.
See the [Side Note: Serialization](#side-note-serialization) section at the end for info on how this can be implemented.

```admonish tip title="Hiding the Build Log"
If you are using a bash-like shell on a UNIX-like OS, you can hide the build log by redirecting stderr to `/dev/null` with: `cargo run --example parser_dev -- grammar.pest main test_1.txt test_2.txt 2>/dev/null`.
Otherwise, you can hide the build log by replacing `WritingTracker::with_stderr()` with `NoopTracker`.
```

Feel free to experiment a bit with the grammar, example files, etc. before continuing.
We will develop an interactive editor next however, which will make experimentation easier!

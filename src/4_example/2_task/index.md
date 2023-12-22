# Task Implementation

Now we'll implement tasks for compiling a grammar and parsing.
Add `task` as a public module to `pie/examples/parser_dev/main.rs`:

```diff2html linebyline
{{#include ../../gen/4_example/2_task/b_1_main_task_mod.rs.diff}}
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


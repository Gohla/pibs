# Interactive Parser Development

Now we'll create an interactive version of this grammar compilation and parsing pipeline, using [Ratatui](https://ratatui.rs/) to create a terminal GUI.
Since we need to edit text files, we'll use [tui-textarea](https://github.com/rhysd/tui-textarea), which is a text editor widget for Ratatui.
Ratatui works with multiple [backends](https://ratatui.rs/concepts/backends/), with [crossterm](https://crates.io/crates/crossterm) being the default backend since it is cross-platform.
Add these libraries as a dependency to `pie/Cargo.toml`:

```diff2html linebyline
{{#include ../../gen/4_example/4_interactive/d_1_Cargo.toml.diff}}
```

We continue as follows:

1) Set up the scaffolding for a Ratatui application.
2) Create a text editor `Buffer` using tui-textarea to edit the grammar and example program files.
3) Draw and update those text editor `Buffer`s, and keep track of the active buffer.
4) Save `Buffer`s back to files and run the `CompileGrammar` and `Parse` tasks to provide feedback on the grammar and example programs.
5) Show the build log in the application.

## Ratatui Scaffolding

We will put the editor in a separate module, and start out with the basic scaffolding of a Ratatui "Hello World" application.
Add `editor` as a public module to `pie/examples/parser_dev/main.rs`:

```diff2html linebyline
{{#include ../../gen/4_example/4_interactive/d_2_main_editor_mod.rs.diff}}
```

Create the `pie/examples/parser_dev/editor.rs` file and add the following to it:

```rust,
{{#include d_3_editor.rs}}
```

The `Editor` struct will hold the state of the editor application, which is currently empty, but we'll add fields to it later.
Likewise, the `new` function doesn't do a lot right now, but it is scaffolding for when we add state.
It returns a `Result` because it can fail in the future.

The `run` method sets up the terminal for GUI rendering, draws the GUI and processes events in a loop until stopped, and then undoes our changes to the terminal.
It is set up in such a way that undoing our changes to the terminal happens regardless if there is an error or not (although panics would still skip that code and leave the terminal in a bad state).
This is a [standard program loop for Ratatui](https://ratatui.rs/tutorial/hello-world/index.html).

```admonish tip title="Rust Help: Returning From Loops" collapsible=true
A [`loop` indicates an infinite loop](https://doc.rust-lang.org/book/ch03-05-control-flow.html#repeating-code-with-loop).
You can [return a value from such loops with `break`](https://doc.rust-lang.org/book/ch03-05-control-flow.html#returning-values-from-loops).
```

The `draw_and_process_event` method first draws the GUI, currently just a hello world message, and then processes events such as key presses.
Currently, this skips key releases because we are only interested in presses, and returns `Ok(false)` if escape is pressed, causing the `loop` to be `break`ed out.

Now we need to go back to our command-line argument parsing and add a flag indicating that we want to start up an interactive editor.
Modify `pie/examples/parser_dev/main.rs`:

```diff2html
{{#include ../../gen/4_example/4_interactive/d_4_main_cli.rs.diff}}
```

We add a new `Cli` struct with an `edit` field that is settable by a short (`-e`) or long (`--edit`) flag, and flatten `Args` into it.
Using this new `Cli` struct here keeps `Args` clean, since the existing code does not need to know about the `edit` flag.
Instead of using a flag, you could also define a [separate command](https://docs.rs/clap/latest/clap/_derive/_tutorial/chapter_0/index.html) for editing.

In `main`, we parse `Cli` instead, check whether `cli.edit` is set, and create and run the editor if it is.
Otherwise, we do a batch build.

Try out the code with `cargo run --example parser_dev -- test.pest main test_1.test test_2.test -e` in a terminal, which should open up a separate screen with a hello world text.
Press escape to exit out of the application.

If the program ever panics, your terminal will be left in a bad state.
In that case, you'll have to reset your terminal back to a good state, or restart your terminal.

## Text Editor `Buffer`

The goal of this application is to develop a grammar alongside example programs of that grammar, getting feedback whether the grammar is correct, but also getting feedback whether the example programs can be parsed with the grammar.
Therefore, we will need to draw multiple text editors along with space for feedback, and be able to swap between active editors.
This will be the responsibility of the `Buffer` struct which we will create in a separate module.
Add the `buffer` module to `pie/examples/parser_dev/editor.rs`:

```diff2html
{{#include ../../gen/4_example/4_interactive/e_1_editor_buffer_mod.rs.diff}}
```

Then create the `pie/examples/parser_dev/editor/buffer.rs` file and add to it:

```rust,
{{#include e_2_buffer.rs}}
```

A `Buffer` is a text editor for a text file at a certain `path`.
It keeps track of a text editor with `TextArea<'static>`, `feedback` text, and whether the text was `modified` in relation to the file.
`new` creates a `Buffer` and is fallible due to reading a file.

The `draw` method draws/renders the buffer (using the Ratatui `frame`) into `area`, with `active` signifying that this buffer is active and should be highlighted differently.
The first part sets the style of the editor, mainly highlighting an active editor by using `Color::Gray` as the block style.
Default styles indicate that no additional styling is done, basically inheriting the style from a parent widget (i.e., a block), or using the style from your terminal.
The second part creates a [block](https://ratatui.rs/how-to/widgets/block.html) that renders a border around the text editor and renders a title on the upper border.
The third part splits up the available space into space for the text editor (80%), and space for the feedback text (at least 7 lines), and renders the text editor and feedback text into those spaces.
The layout can of course be tweaked, but it works for this example.

`process_event` lets the text editor process input events, and updates whether the text has been modified.
`save_if_modified` saves the text to file, but only if modified.
`path` gets the file path of the buffer.
`feedback_mut` returns a mutable borrow to the feedback text, enabling modification of the feedback text.

It is up to the user of `Buffer` to keep track of the active buffer, sending `active: true` to the `draw` method of that buffer, and calling `process_event` on the active buffer.
That's exactly what we're going to implement next.

### Drawing and Updating `Buffer`s

We'll create `Buffers` in `Editor` and keep track of the active buffer.
To keep this example simple, we'll create buffers only for the grammar file and example program files given as command-line arguments.
If you want more or less example files, you'll have to exit the application, add those example files to the command-line arguments, and then start the application again.
 
Modify `pie/examples/parser_dev/editor.rs`:
       
```diff2html
{{#include ../../gen/4_example/4_interactive/e_3_editor_buffers.rs.diff}}
```

`Editor` now has a list of `buffers` via `Vec<Buffer>` and keeps track of the active tracker via `active_buffer` which is an index into `buffers`.
In `new`, we create buffers based on the grammar and program file paths in `args`.
The buffers `Vec` is created in such a way that the first buffer is always the grammar buffer, with the rest being example program buffers.
The grammar buffer always exists because `args.grammar_file_path` is mandatory, but there can be 0 or more example program buffers.

`draw_and_process_event` now splits up the available space.
First vertically: as much space as possible is reserved for buffers, with at least 1 line being reserved for a help line at the bottom.
Then horizontally: half of the horizontal space is reserved for a grammar buffer, and the other half for program buffers.
The vertical space for program buffers (`program_buffer_areas`) is further divided: evenly split between all program buffers.

Then, the buffers are drawn in the corresponding spaces with `active` only being `true` if we are drawing the active buffer, based on the `active_buffer` index.  

In the event processing code, we match the Control+T shortcut and increase the `active_buffer` index.
We wrap back to 0 when the `active_buffer` index would overflow, using a modulo (%) operator, ensuring that `active_buffer` is always a correct index into the `buffers` `Vec`.
Finally, if none of the other shortcuts match, we send the event to the active buffer.

Try out the code again with `cargo run --example parser_dev -- test.pest main test_1.test test_2.test -e` in a terminal.
This should open up the application with a grammar buffer on the left, and two program buffers on the right.
Use Control+T to swap between buffers, and escape to exit.

## Saving `Buffer`s and Providing Feedback

Next up is saving the buffers, running the compile grammar and parse tasks, and show feedback from those tasks in the feedback space of buffers.
Modify `pie/examples/parser_dev/editor.rs`:
       
```diff2html
{{#include ../../gen/4_example/4_interactive/f_editor_update.rs.diff}}
```

The biggest addition as at the bottom: the `save_and_update_buffers` method.
This method first clears the feedback text for all buffers, and saves all buffers (if `save` is `true`).
Then we create a new PIE session and require the compile grammar task and parse tasks, similar to `compile_grammar_and_parse` in the main file.
Here we instead `writeln!` the results to the feedback text of buffers.

We store the `rule_name` in `Editor` as that is needed to create parse tasks, and store a `Pie` instance so that we can create new PIE sessions to require tasks.

When the Control+S shortcut is pressed, we call `save_and_update_buffers` with `save` set to `true`.
We also call `save_and_update_buffers` in `Editor::new` to provide feedback when the application starts out, but with `save` set to false, so we don't immediately save all files.
Finally, we update the help line to include the Control+S shortcut.

Try out the code again with `cargo run --example parser_dev -- test.pest main test_1.test test_2.test -e` in a terminal.
Now you should be able to make changes to the grammar and/or example programs, press Control+S to save modified files, and get feedback on grammar compilation and parsing example programs.
If you like, you can go through the [pest parser book](https://pest.rs/book/) and experiment with/develop a parser.

## Showing the Build Log

We'll add one more feature to the editor: showing the build log.
We can do this by writing the build log to an in-memory text buffer, and by drawing that text buffer.
Modify `pie/examples/parser_dev/editor.rs`:
       
```diff2html
{{#include ../../gen/4_example/4_interactive/g_editor_build_log.rs.diff}}
```

In `new` we now create the `Pie` instance with a writing tracker: `WritingTracker::new(Cursor::new(Vec::new()))`.
This writing tracker writes to a [`Cursor`](https://doc.rust-lang.org/std/io/struct.Cursor.html), specifically `Cursor<Vec<u8>>` for which [`Write` is implemented](https://doc.rust-lang.org/src/std/io/cursor.rs.html#570-591).
We modify the type of the `pie` field to include the tracker type to reflect this: `WritingTracker<Cursor<Vec<u8>>>`.
Build logs will then be written to the `Vec<u8>` inside the `Cursor`.

To draw the build log in between the buffers and help line, we first modify the layout split into `root_areas`: buffers now take up 70% of vertical space, and add a new constraint for the build log which takes 30% of vertical space.

We access the in-memory buffer via `&self.pie.tracker().writer().get_ref()`, convert this to a string via [`String::from_utf8_lossy`](https://doc.rust-lang.org/std/string/struct.String.html#method.from_utf8_lossy), and convert that to [Ratatui `Text`](https://docs.rs/ratatui/latest/ratatui/text/struct.Text.html) which can be passed to [`Paragraph::new`](https://docs.rs/ratatui/latest/ratatui/widgets/struct.Paragraph.html#method.new) and also gives us line information for scrolling the build log.
The scroll calculation is explained in the comments.
We then draw the build log as a `Paragraph`.

Finally, we update the area for the help line from `root_areas[1]` to `root_areas[2]`, as adding the layout constraint shifted the index up.

Try out the code again with `cargo run --example parser_dev -- test.pest main test_1.test test_2.test -e` in a terminal.
Pressing Control+S causes tasks to be required, which is shown in the build log.
Try modifying a single file to see what tasks PIE executes, or what the effect of an error in the grammar has.

And with that, we're done with the interactive parser development example 🎉🎉🎉!

## Conclusion

In this example, we developed tasks for compiling a grammar and parsing files with that grammar, and then used those tasks to implement both a batch build, and an interactive parser development environment.

In the introduction, we [motivated](../0_intro/index.md#motivation) programmatic incremental build systems with the key properties of: programmatic, incremental, correct, automatic, and multipurpose.
Did these properties help with the implementation of this example application?

- Programmatic: due to the build script -- that is: the compile grammar and parse tasks -- being written in the same programming language as the application, it was extremely simple to integrate. We also didn't have to learn a separate language, we could just apply our knowledge of Rust!
- Incremental: PIE incrementalized the build for us, so we didn't have to implement incrementality. This saves a lot of development effort as implemented incrementality is complicated. 
  - The batch build is unfortunately not incremental due to not having implemented serialization in this tutorial, but this is not a fundamental limitation. See [Side Note: Serialization](#side-note-serialization) for info on how to solve this.
- Correct: PIE ensures the build is correct, so we don't have to worry about glitches or inconsistent data, again saving development effort that would otherwise be spent on ensuring incrementality is correct.
  - For a real application, we should write tests to increase the confidence that our build is correct, because PIE checks for correctness at runtime.
- Automatic: we didn't manually implement incrementality, but only specified the dependencies: from compile grammar/parse task to a file, and from parse tasks to compile grammar tasks.
- Multipurpose: we reused the same tasks for both a batch build and for use in an interactive environment, without any modifications. Again, this saves development time.

So yes, I think that programmatic incremental build systems -- and in particular PIE -- help a lot when developing applications that require incremental batch builds or interactive pipelines, and especially when both are required.
The main benefit is reduced development effort, due to not having to solve the problem of correct incrementality, due to easy integration, and due to only needing to know and use a single programming language.

Larger applications with more features and complications that need incrementality would require an even bigger implementation effort.
Therefore, larger applications could benefit even more from using PIE.
Of course, you cannot really extrapolate that from this small example.
However, I have applied PIE to a larger application: the Spoofax Language Workbench, and found similar benefits. 
More info on this [can be found in the appendix](../a_appendix/1_pie.md#implementations).

You should of course decide for yourself whether a programmatic incremental build system really helped with implementing this example.
Every problem is different, and requires separate consideration as to what tools best solve a particular problem.

This is currently the end of the guided programming tutorial.
In the appendix chapters, we discuss PIE implementations and publications, related work, and future work.

```admonish example title="Download source code" collapsible=true
You can [download the source files up to this point](../../gen/4_example/4_interactive/source.zip).
```

## Side Note: Serialization

To get incrementality between different runs (i.e., processes) of the program, we need to serialize the `Store` before the program exits, and deserialize the `Store` when the program starts.

The de-facto standard (and awesome) serialization library in Rust in [serde](https://serde.rs/).
See the [PIE in Rust repository at the `pre_type_refactor` tag](https://github.com/Gohla/pie/blob/pre_type_refactor/pie/) for a version of PIE with serde serialization.
For example, the [`Store`](https://github.com/Gohla/pie/blob/pre_type_refactor/pie/src/store.rs#L14-L17) struct has annotations for deriving `serde::Deserialize` and `serde::Serialize`.
These attributes are somewhat convoluted due to serialization being optional, and due to the `H` generic type parameter which should not be included into serialization bounds.

You should derive `serde::Deserialize` and `serde::Serialize` for all required types in the PIE library, but also all tasks, and all task outputs.
The `pie_graph` library support serialization when the `serde` feature is enabled, which is enabled by default.
Then, see [this serialization integration test](https://github.com/Gohla/pie/blob/pre_type_refactor/pie/tests/serde.rs).

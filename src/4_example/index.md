# Project: Interactive Parser Development

To demonstrate what can be done with the programmatic incremental build system we just created, we will develop a "parser development" build script and interactive editor as a project.
In this project, we can develop a grammar for a new (programming) language, and test that grammar against several example files written in the new language.

It will have both a batch mode and an interactive mode.
In the batch mode, the grammar is checked and compiled, the example program files are parsed with the grammar, and the results are printed to the terminal.
The interactive mode will start up an interactive editor in which we can develop and test the grammar interactively.
We will develop tasks to perform grammar compilation and parsing, and incrementally execute them with PIE.
Both batch and interactive mode will use the same tasks!

We will use [pest](https://pest.rs/) as the parser framework, because it is written in Rust and can be easily embedded into an application.
Pest uses Parsing Expression Grammars (PEGs) which are easy to understand, which is also good for this project.

For the GUI, we will use [Ratatui](https://ratatui.rs/), which is a cross-platform terminal GUI framework, along with [tui-textarea](https://github.com/rhysd/tui-textarea) for a text editor widget.
We could use a more featured GUI framework like [egui](https://github.com/emilk/egui), but for this project we'll keep it simple and runnable in a terminal.

As a little teaser, this is what the interactive mode looks like:

<script src="https://asciinema.org/a/VfP8uiZ0MSs5QgzY0BhIxKp6L.js" id="asciicast-VfP8uiZ0MSs5QgzY0BhIxKp6L" async data-autoplay="true" data-loop="true" data-speed="1.25" data-idleTimeLimit="1" data-theme="solarized-dark"></script>
[//]: # (![Interactive mode demo]&#40;demo.gif&#41;)

We will continue as follows:

1) Implement compilation of pest grammars and parsing of text with the compiled grammar.
2) Create tasks for grammar compilation and parsing.
3) Parse CLI arguments and run these tasks in a non-interactive setting.
4) Create a terminal GUI for interactive parser development.

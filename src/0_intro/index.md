# Build your own Programmatic Incremental Build System

This is a programming tutorial where you will build your own _programmatic incremental build system_, which is a mix between an incremental build system and an incremental computation system.
Programmatic incremental build systems enable programmers to write expressive build scripts and interactive programs in a regular programming language, with the system taking care of correct incrementality once and for all, freeing programmers from having to manually implement complicated and error-prone incrementality every time.

The primary goal of this tutorial is to provide understanding of programmatic incremental build systems through implementation and experimentation.

In this programming tutorial you will write [Rust](https://www.rust-lang.org/) code, but you _don't_ need to be a Rust expert to follow it.
A secondary goal of this tutorial is to teach more about Rust through implementation and experimentation, given that you already have some programming experience (in another language) and are willing to learn. 
Therefore, all Rust code is available, and I try to explain and link to the relevant Rust book chapters as much as possible.

This is of course not a full tutorial or book on Rust.
For that, I can recommend the excellent [The Rust Programming Language](https://doc.rust-lang.org/book/) book.
However, if you like to learn through examples and experimentation, or already know Rust basics and want to practice, this might be a fun programming tutorial for you!

We will first motivate programmatic incremental build systems in more detail.

## Motivation

A programmatic incremental build system is a mix between an incremental build system and an incremental computation system, with the following key properties:

- _Programmatic_: Build scripts are regular programs written in a programming language, where parts of the program implement an API from the build system. This enables programmers to write incremental builds scripts and interactive programs with the full expressiveness of the programming language.
- _Incremental_: Builds are truly incremental -- only the parts of a build that are affected by changes are executed.
- _Correct_: Builds are fully correct -- all parts of the build that are affected by changes are executed. Builds are free of glitches: only up-to-date (consistent) data is observed.
- _Automatic_: The system takes care of incrementality and correctness. Programmers _do not_ have to manually implement incrementality. Instead, they only have to explicitly _declare dependencies_.

To show the benefits of a build system with these key properties, below is a simplified version of the build script for compiling a formal grammar and parsing text with that compiled grammar, which is the build script you will implement in the [final project chapter](../4_example/index.md).
This simplified version removes details that are not important for understanding programmatic incremental build systems at this moment.

```admonish info
Don't worry if you do not (fully) understand this code, the tutorial will guide you more with programming and understanding this kind of code.
This example is primarily here to motivate programmatic incremental build systems, as it is hard to do so without it.
```

```rust
pub enum ParseTasks {
  CompileGrammar { grammar_file: PathBuf },
  Parse { compile_grammar_task: Box<ParseTasks>, text_file: PathBuf, rule_name: String }
}

pub enum Outputs {
  CompiledGrammar(CompiledGrammar),
  Parsed(String)
}

impl Task for ParseTasks {
  fn execute<C: Context>(&self, context: &mut C) -> Result<Outputs, Error> {
    match self {
      ParseTasks::CompileGrammar { grammar_file } => {
        let grammar_text = context.require_file(grammar_file)?;
        let compiled_grammar = CompiledGrammar::new(&grammar_text)?;
        Ok(Outputs::CompiledGrammar(compiled_grammar))
      }
      ParseTasks::Parse { compile_grammar_task, text_file, rule_name } => {
        let compiled_grammar = context.require_task(compile_grammar_task)?;
        let text = context.require_file(text_file)?;
        let output = compiled_grammar.parse(&text, rule_name)?;
        Ok(Outputs::Parsed(output))
      }
    }
  }
}

fn main() {
  let compile_grammar_task = Box::new(ParseTasks::CompileGrammar {
    grammar_file: PathBuf::from("grammar.pest")
  });
  let parse_1_task = ParseTasks::Parse {
    compile_grammar_task: compile_grammar_task.clone(),
    text_file: PathBuf::from("test_1.txt"),
    rule_name: "main"
  };
  let parse_2_task = ParseTasks::Parse {
    compile_grammar_task: compile_grammar_task.clone(),
    text_file: PathBuf::from("test_2.txt"),
    rule_name: "main"
  };
  
  let mut context = IncrementalBuildContext::default();
  let output_1 = context.require_task(&parse_1_task).unwrap();
  println("{output_1:?}");
  let output_2 = context.require_task(&parse_2_task).unwrap();
  println("{output_2:?}");
}
```

This is in essence just a normal (pure) Rust program: it has enums, a trait implementation for one of those enums, and a `main` function.
However, this program is also a build script because `ParseTasks` implements the `Task` trait, which is the core trait defining the unit of computation in a programmatic incremental build system.
Because `ParseTasks` is an enum, there are two kinds of tasks: a `CompileGrammar` task that compiles a grammar, and a `Parse` task that parses a text file using the compiled grammar.

##### Tasks

A _task_ is kind of like a closure: a function along with its inputs that can be executed.
For example, `CompileGrammar` carries `grammar_file_path` which is the file path of the grammar that it will compile.
When we `execute` a `CompileGrammar` task, it reads the text of the grammar from the file, compiles that text into a grammar, and returns a compiled grammar.

Tasks differ from closures however, in that tasks are _incremental_.

##### Incremental File Dependencies

We want the `CompileGrammar` task to be incremental, such that this task is only re-executed when the contents of the `grammar_file` file changes.
Therefore, `execute` has a `context` parameter which is an _incremental build context_ that tasks use to tell the build system about dependencies.

For example, `CompileGrammar` tells the build system that it _requires_ the `grammar_file` file with `context.require_file(grammar_file)`, creating a _file read dependency_ to that file.
It is then the responsibility of the incremental build system to only execute this task if the file contents have changed.

##### Dynamic Dependencies

Note that this file dependency is created _while the task is executing_!
We call these _dynamic dependencies_, as opposed to static dependencies.
Dynamic dependencies enable the _programmatic_ part of programmatic incremental build systems, because dependencies are made while your program is running, and can thus depend on values computed earlier in your program.

##### Incremental Task Dependencies

Dynamic dependencies are also created _between tasks_.
For example, `Parse` carries `compile_grammar_task` which is an instance of the `CompileGrammar` task to compile a grammar.
When we `execute` a `Parse` task, it tells the build system that it depends on the compile grammar task with `context.require_task(compile_grammar_task)`.

This also asks the build system to return the most up-to-date (consistent) output of that task.
It is then the responsibility of the incremental build system to _check_ whether the task is _consistent_, and to _re-execute_ it only if it is _inconsistent_.
In essence, the build system will take these steps:

- If `compile_grammar_task` was never executed before, the build system executes it, caches the compiled grammar, and returns the compiled grammar.
- Otherwise, to check if the compile grammar task is consistent, we need to check its dependencies: the file dependency to `grammar_file`.
  - If the contents of the `grammar_file` file has changed, the task is inconsistent and the build system re-executes it, caches the new compiled grammar, and returns it.
  - Otherwise, the task is consistent and the build system simply returns the cached compiled grammar.

The `Parse` task then has access to the `compiled_grammar`, reads the text file to parse with `require_file`, and finally parses the `text` with `compiled_grammar.parse`.

##### Using Tasks

Because this is just a regular Rust program, we can use the tasks in the same program with a `main` function.
The `main` function creates instances of these tasks, creates an `IncrementalBuildContext`, and asks the build system to return the up-to-date outputs for two tasks with `context.require_task`.

This `main` function is performing an incremental batch build.
However, you can also use these same tasks to build an _interactive application_.
That is too much code to discuss here in this introduction, but the [final project chapter](../4_example/index.md) shows a video of an interactive application that you can build using these tasks.

##### Conclusion

This is the essence of programmatic incremental build systems.
In this tutorial, we will define the `Task` trait and implement the `IncrementalBuildContext` over the course of several chapters.

However, before we start doing that, I want to first zoom back out and discuss the benefits (and drawbacks) of programmatic incremental build systems.
If you already feel motivated enough, you can [skip to here](#pie-a-programmatic-incremental-build-system-in-rust).

### Benefits

The primary motivation for programmatic incremental build systems is that you can _program_ your incremental builds and interactive applications in a regular programming language, instead of having to write it in a separate (declarative) build script language, and this has several benefits:

- You can re-use your knowledge of the programming language, instead of having to learn a new build script language.
- You can use tools of the programming language, such as the compiler that provides (good) error messages, an IDE that helps you read and write code, a debugger for understanding the program, unit and integration testing for improving code reliability, benchmarking for improving performance, etc.
- You can modularize your build script using facilities of the programming language, enabling you to reuse your build script as a library or to use modules created by others in your build script. You can also use regular modules of the programming language and integrate them into build scripts, and vice versa.

The other important benefit is that incrementality and correctness are taken care of by the build system.
Therefore, you don't have to manually implement incrementality in a correct way, which is complicated and error-prone to implement.

You do have to specify the exact dependencies of tasks to files and other tasks, as seen in the example, but this is easier than implementing incrementality.
Due to the dependencies being dynamic, you can use regular programming language constructs like calling a function to figure out what file to depend on, `if` to create conditional dependencies, `while` to create multiple dependencies, and so forth.

Exactly specifying the dependencies in this way has another important benefit: the dynamic dependencies of a task _perfectly describe when the task should be re-executed_, enabling the build system to be fully incremental and correct.
This is in contrast to build system with static dependencies -- dependencies that cannot use runtime values, typically using literal file names or patterns -- where dependencies often have to be over-approximated (not fully incremental) or under-approximated (not correct) due to not being able to exactly specify dependencies.

Some build systems use _multiple stages_ to emulate a limited form of dynamic dependencies.
For example, dynamic dependencies in [Make](https://www.gnu.org/software/make/) requires staging: first dynamically generate new makefiles with correct dependencies, and then recursively execute them.
[Gradle](https://gradle.org/) has a two-staged build process: first configure the task graph, then incrementally execute it, but no new dependencies nor tasks can be created during execution.
This is an improvement over static dependencies, but requires you to think about what to do in each stage, requires maintenance of each stage, and limits what you can do in each stage.

A final benefit of dynamic dependencies is that they do away with staging because there is only a single stage: the execution of your build script, and you can create dynamic dependencies in this single stage.
This increases expressiveness, makes build scripts easier to read and write, and reduces maintenance overhead. 

### Drawbacks

Of course, programmatic incremental build systems also have some drawbacks.
These drawbacks become more clear during the tutorial, but I want to list them here to be up-front about it:

- The build system is more complicated, but hopefully this tutorial can help mitigate some of that by understanding the key ideas through implementation and experimentation.
- Some correctness properties are checked while building. Therefore, you need to test your builds to try to catch these issues before they reach users. However, I think that testing builds is something you should do regardless of the build system, to be more confident about the correctness of your build.
- More tracking is required at runtime compared to non-programmatic build systems. However, in our experience, the overhead is not excessive unless you try to do very fine-grained incrementalization. For fine-grained incrementalization, [incremental computing](https://en.wikipedia.org/wiki/Incremental_computing) approaches are more well suited.

## PIE: a Programmatic Incremental Build System in Rust

We have developed [PIE, a Rust library](https://github.com/Gohla/pie) implementing a programmatic incremental build system adhering to the key properties listed above.
It is still under development, and has not been published to crates.io yet, but it is already usable 
If you are interested in experimenting with a programmatic incremental build system, do check it out!

In this tutorial we will implement a subset of PIE.
We simplify the internals in order to minimize distractions as much as possible, but still go over all the key ideas and concepts that make programmatic incremental build systems tick.

However, the _idea_ of programmatic incremental build systems is not limited to PIE or the Rust language.
You can implement a programmatic incremental build systems in any general-purpose programming language, or adapt the idea to better fit your preferences and/or requirements.
In fact, we first implemented [PIE in Java](https://github.com/metaborg/pie), with [PIE in Rust](https://github.com/Gohla/pie) being the second iteration, mostly simplifying internals to make it easier to explain.

For a more thorough discussion on PIE, see the [PIE Implementations & Publications appendix chapter](../a_appendix/1_pie.md), and the [Related Work appendix chapter](../a_appendix/2_related_work.md).

## Feedback & Contributing

This tutorial is open source, hosted at <https://github.com/Gohla/pibs>.
If you find an error in the code or text of this tutorial, or want to report other kinds of problems, you can report it on the [issue tracker](https://github.com/Gohla/pibs/issues).
Small fixes can be sent as a pull request by pressing the edit button in the top-right corner.

Let's continue with the tutorial.
The next section covers installing Rust and setting up a fresh Rust project.

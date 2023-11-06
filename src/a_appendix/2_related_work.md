# Related Work

There are several other programmatic incremental build systems and works published about them.
This subsection discusses them.
For additional related work discussion, check the related work sections of [chapter 7 (page 104)](https://gkonat.github.io/assets/dissertation/konat_dissertation.pdf#page=126) and [chapter 8 (page 126)](https://gkonat.github.io/assets/dissertation/konat_dissertation.pdf#page=148) of my dissertation. 

## Pluto

PIE is based on [Pluto](https://www.pl.informatik.uni-mainz.de/files/2019/04/pluto-incremental-build.pdf), a programmatic incremental build system developed by Sebastian Erdweg et al.
This is not a coincidence, as Sebastian Erdweg was my PhD promotor, and we developed and wrote the "Scalable Incremental Building with Dynamic Task Dependencies" paper together.

The [Pluto paper](https://www.pl.informatik.uni-mainz.de/files/2019/04/pluto-incremental-build.pdf) provides a more formal proof of incrementality and correctness for the top-down build algorithm, which provides confidence that this algorithm works correctly, but also explains the intricate details of the algorithm very well.
Note that Pluto uses "builder" instead of "task".
In fact, a Pluto builder is more like an incremental function that _does not carry its input_, whereas a PIE task is more like an incremental closure that includes its input.
 
PIE uses almost the same top-down build algorithm as Pluto, but there are some technical changes that make PIE more convenient to use.
In Pluto, tasks are responsible for storing their output and dependencies, called "build units", which are typically stored in files.
In PIE, the library handles this for you.
The downside is that PIE requires a mapping from a `Task` (using its `Eq` and `Hash` impls) to its dependencies and output (which is what the `Store` does), and some modifications had to be done to the consistency checking routines.
The upside is that tasks don't have to manage these build unit files, and the central `Store` can efficiently manage the entire dependency graph.
Especially this central dependency graph management is useful for the bottom-up build algorithm, as we can use [dynamic topological sort algorithms for directed acyclic graphs](http://www.doc.ic.ac.uk/~phjk/Publications/DynamicTopoSortAlg-JEA-07.pdf).

## Other Incremental Build Systems with Dynamic Dependencies

[Build Systems à la Carte](https://www.microsoft.com/en-us/research/uploads/prod/2018/03/build-systems.pdf) shows a systematic and executable framework (in Haskell) for developing and comparing build systems. It compares the impact of design decisions such as what persistent build information to store, the scheduler to use, static/dynamic dependencies, whether it is minimal, supports early cutoff, and whether it supports distributed (cloud) builds. 
Even though the Haskell code might be a bit confusing if you're not used to functional programming, it is a great paper that discusses many aspects of programmatic incremental build systems and how to implement them.

### Shake

[Shake](https://shakebuild.com/) is an incremental build system implemented in Haskell, described in detail in the [Shake Before Building](https://ndmitchell.com/downloads/paper-shake_before_building-10_sep_2012.pdf) paper.
The main difference in the model between Shake and PIE is that Shake follows a more target-based approach as seen in Make, where targets are build tasks that provide the files of the target.
Therefore, the output (provided) files of a build task need to be known up-front.
The upside of this approach is that build scripts are easier to read and write and easier to parallelize.
However, the main downside is that it is not possible to express build tasks where the names of provided files are only known after executing the task.
For example, compiling a Java class with inner classes results in a class file for every inner class with a name based on the outer and inner class, which is not known up-front.

Implementation wise, Shake supports explicit parallelism, whereas PIE cannot (at the time of writing).
Parallel builds in PIE are tricky because two build tasks executing in parallel could require/provide (read/write) to the same file, which can result in data races.
Shake avoids this issue by requiring provided files to be specified as targets up-front, speeding up builds through explicit parallelism.
In PIE, this might be solvable with a protocol where tasks first call a `Context` method to tell PIE about the files that will be provided, or the directory in which files will be provided, so PIE can limit parallelism on those files and directories.
Tasks that do not know this up-front cannot be executed in parallel, but can still be executed normally.

### Rattle

[Rattle](https://github.com/ndmitchell/rattle) is a build system focussing on easily turning build scripts into incremental and parallel build scripts without requiring dependency annotations, described in detail in the [Build Scripts with Perfect Dependencies](https://ndmitchell.com/downloads/paper-build_scripts_with_perfect_dependencies-18_nov_2020.pdf) paper. 
To make this possible, Rattle has a very different model compared to PIE.

Rattle build scripts consist of (terminal/command-line) commands such as `gcc -c main.c`, and simple control logic/manipulation to work with the results of commands, such as if checks, for loops, or changing the file extension in a path.
Therefore, future commands can use values of previous commands, and use control logic to selectively or iteratively execute commands.
Commands create dynamic file dependencies, both reading (require) and writing (provide), which are automatically detected with dependency tracing on the OS level.
There are no explicit dependencies between commands, but implicit dependencies arise when a command reads a file that another command writes for example.

Rattle incrementally executes the commands of a build script, skipping commands for which no files have changed.
The control logic/manipulation around the commands is not incrementalized.
Rattle build scripts can be explicitly parallelized, but Rattle also implicitly parallelizes builds by speculatively executing future commands.
If speculation results in a hazard, such as a command reading a file and then a command writing to that file -- equivalent to a hidden dependency in PIE -- then the build is inconsistent and must be restarted without speculative parallelism.

#### Core difference

The best way I can explain the core difference is that Rattle builds a _single build script_ which is a _stream of commands_ with _file dependencies_; whereas in PIE, every build task is in essence _its own build script_ that _produces an output value_, with file dependencies but also _dependencies between build tasks_.
Both models have merit!

The primary advantage of the Rattle model is that existing build scripts, such as Make scripts or even just Bash scripts, can be easily converted to Rattle build scripts by converting the commands and control logic/manipulation into Rattle.
No file dependencies have to be specified since they are automatically detected with file dependency tracing.
Then, Rattle can parallelize and incrementally execute the build script.
Therefore, Rattle is great for incrementalizing and parallelizing existing Make/Bash/similar build scripts with very low effort.

While it is possible to incrementalize these kind of builds in PIE, the effort will be higher due to having to split commands into task, and having to report the file dependencies to PIE.
If PIE had access to reliable cross-platform automated file dependency tracing, we could reduce this effort by building a "command task" that executes arbitrary terminal/command-line commands.
However, reliable cross-platform file dependency tracking does not exist (to my knowledge, at the time of writing).
The library that Rattle uses, [Fsatrace](https://github.com/jacereda/fsatrace), has limitations such as not detecting reads/writes to directories, and having to disable system integrity protection on macOS.
Therefore, Rattle also (as mentioned in the paper, frustratingly) inherits the limitations of this library.

Compared to Rattle, the primary advantages of programmatic incremental build systems (i.e., the PIE model) are: 
- PIE can _rebuild a subset of the build script_, instead of only the entire build script.
- The entire build is incrementalized (using tasks as a boundary), not just commands.
- Tasks can return any value of the programming language, not just strings.
- Tasks are modular, and can be shared using the mechanism of the programming language.

[//]: # (TODO: motivate these points better)

[//]: # (TODO: cache output values; PIE tasks can create values that are expensive to calculate, so we want to cache them. In Rattle this has to be written to a file.)

These properties are a necessity for use in interactive environments, such as a code editors, IDEs, or other using-facing interactive applications.
Therefore, the PIE model is more suited towards incrementalization in interactive environment, but can still be used to do incremental batch builds.

#### Implicit Parallelism (Speculative Execution)

Rattle supports both implicit and explicit parallelization, whereas PIE does not at the time of writing.
Explicit parallelism was already discussed in the Shake section.

After a first build, Rattle knows which commands have been executed and can perform implicit parallelization by speculatively executing future commands.
If a hazard occurs, the build is restarted without speculation (other recovery mechanisms are also mentioned in the paper), although the evaluation shows that this is rare, and even then the builds are still fast due to incrementality and explicit parallelism.

After the initial build, PIE also has full knowledge of the build script.
In fact, we know more about the build script due to tracking both the file dependencies _and the dependencies between tasks_.
However, just like Rattle, PIE doesn't know whether the tasks that were required last time, will be the tasks that are required this time.
In principle, 0 tasks that were required last time can be required the next time.
Therefore, if we would do speculative execution of future commands, we could run into similar hazard: hidden dependencies and overlapping provided files.

However, I think that restarting the build without speculative execution, when a hazard is detected, is incorrect in PIE.
This is because PIE keeps track of the entire dependency graph, including task output values, which would not be correct after a hazard.
Restarting the build could then produce a different result, because PIE uses the previously created dependency graph for incrementality.
In Rattle this is correct because it only keeps track of file dependencies of commands.

So I am currently not sure if and how we could do implicit parallelism in PIE.

#### Self-Tracking

Self-tracking is the ability of an incremental build system to correctly react to _changes in the build script_.
If a part of the build script changes, that part should be re-executed.

Rattle supports self-tracking without special support for it, because Rattle makes no assumption about the build script, and re-executes the build script every time (while skipping commands that have not been affected).
Therefore, build script changes are handled automatically.

PIE supports self-tracking by creating a dependency to the source code or binary file of a task.
However, this requires support from the programming language to find the source or binary file corresponding to a task.
In the Java implementation of PIE, we can use [class loaders to get the (binary) class files for tasks and related files](https://github.com/metaborg/spoofax-pie/blob/develop/lwb/metalang/stratego/stratego/src/main/java/mb/str/task/spoofax/StrategoParseWrapper.java).
In the [Rust implementation of PIE](https://github.com/Gohla/pie), we have not yet implemented self-tracking.
In Rust, we could implement self-tracking by writing a [procedural macro](https://doc.rust-lang.org/beta/reference/procedural-macros.html) that can be applied to `Task` implementations to embed a self-tracking dependency (probably a hash over the `Task` `impl`) into the `Task::execute` method.

However, since PIE is fully programmatic, tasks can use arbitrary code.
To be fully correct, we'd need to over-approximate: check whether the binary of the program has changed and consider all tasks inconsistent if the binary has changed.
In practice, the approach from the Java implementation of PIE works well, alongside a version number that gets updated when code used by tasks changes semantics in a significant way.

#### Cloud Builds

Rattle could support "cloud builds" where the output files of a command are stored on a server, using the hashed inputs (command string and read files) of the command as a key.
Subsequent builds that run command with matching hashes could then just download the output files and put them in the right spot.
It is unclear if Rattle actually does this, but they discuss it (and several problems in practice) in the paper.

PIE does not currently support this, but could support it in a similar way (with the same practical problems).
In essence, the `Store` as implemented in this tutorial is such a key-value store, except that it is locally stored.
We also cache task outputs, but they could be stored in a similar way.

Whether this is a good idea depends on the task.
For tasks that are expensive to execute, querying a server and getting the data from the server can be faster than executing the task.
For tasks that are cheap to execute, just executing it can be faster.

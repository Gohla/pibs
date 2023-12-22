# PIE Implementations & Publications

## Implementations

In this tutorial, you implemented a large part of the PIE, the programmatic incremental build system that I developed during my PhD and Postdoc.
There are currently two versions of PIE:

- [PIE in Rust](https://github.com/Gohla/pie), a superset of what you have been developing in this tutorial.
  - The largest differences between PIE in this tutorial and the PIE library are:
    - [Support for arbitrary task](https://github.com/Gohla/pie/blob/main/pie/src/lib.rs#L72) and [resource types](https://github.com/Gohla/pie/blob/main/pie/src/lib.rs#L74-L97), achieved by using [trait objects](https://github.com/Gohla/pie/blob/main/pie/src/trait_object/mod.rs) to provide dynamic dispatch. 
    - [Resource abstraction](https://github.com/Gohla/pie/blob/main/pie/src/lib.rs#L117-L130) enables resources other than files. Resources are global mutable state where the state is not handled by the PIE library (as opposed to task inputs and outputs), but _read and write access to_ that state _is_ handled by PIE. [Files (as `PathBuf`)](https://github.com/Gohla/pie/blob/main/pie/src/resource/file.rs) are a resource, but so is a [hashmap](https://github.com/Gohla/pie/blob/main/pie/src/resource/map.rs).
    - Terminology differences. The PIE library uses _read_ and _write_ for resource dependencies instead of _require_ and _provide_. This allows us to use _require_ only for tasks, and _read_ and _write_ only for resources. It uses _checkers_ instead of _stampers_.
  - The motivation for developing a PIE library in Rust was to test whether the idea of a programmatic incremental build system really is programming-language agnostic, as a target for developing this tutorial, and to get a higher-performance implementation compared to the Java implementation of PIE.
  - In my opinion, implementing PIE in Rust as part of this tutorial is a much nicer experience than implementing it in Java, due to the more powerful type system and great tooling provided by Cargo. However, supporting multiple task types, which we didn't do in this tutorial, is a bit of a pain due to requiring trait objects, which can be really complicated to work with in certain cases. In Java, everything is a like a trait object, and you get many of these things for free, at the cost of garbage collection and performance of course.
- [PIE in Java](https://github.com/metaborg/pie). The motivation for using Java was so that we could use PIE to correctly incrementalize the [Spoofax Language Workbench](https://spoofax.dev/), a set of tools and interactive development environment (IDE) for developing programming languages. In Spoofax, you develop a programming language by _defining_ the aspects of your language in _domain-specific meta-languages_, such as SDF3 for syntax definition, and Statix for type system and name binding definitions. 
 
  Applying PIE to Spoofax culminated in [Spoofax 3](https://spoofax.dev/spoofax-pie/develop/) (sometimes also called Spoofax-PIE), a new version of Spoofax that uses PIE for all tasks such as generating parsers, running parsers on files to create ASTs, running highlighters on those ASTs to provide syntax highlighting for code editors, etc. Because all tasks are PIE tasks, we can do correct and incremental batch builds of language definitions, but also live development of those language definitions in an IDE, using PIE to get feedback such as inline errors and syntax highlighting as fast as possible.

## Publications

We wrote two papers about programmatic incremental build systems and PIE, for which updated versions are in my dissertation:
- [Chapter 7, page 83: PIE: A Domain-Specific Language for Interactive Software Development Pipelines](https://gkonat.github.io/assets/dissertation/konat_dissertation.pdf#page=105). 
 
  This describes a domain-specific language (DSL) for programmatic incremental build systems, and introduces the PIE library in Kotlin. This implementation was later changed to a pure Java library to reduce the number of dependencies. 
- [Chapter 8, page 109: Scalable Incremental Building with Dynamic Task Dependencies](https://gkonat.github.io/assets/dissertation/konat_dissertation.pdf#page=131).

  This describes a hybrid incremental build algorithm that builds from the bottom-up, only switching to top-down building when necessary. Bottom-up builds are more efficient with changes that have a small effect (i.e., most changes), due to only _checking the part of the dependency graph affected by changes_. Therefore, this algorithm _scales down to small changes while scaling up to large dependency graphs_. 

  Unfortunately, we did not implement (hybrid) bottom-up building in this tutorial due to a lack of time. However, the [PIE in Rust](https://github.com/Gohla/pie) library has a [bottom-up context implementation](https://github.com/Gohla/pie/blob/master/pie/src/context/bottom_up.rs) which you can check out. Due to similarities between the top-down and bottom-up context, some common functionality was [extracted into an extension trait](https://github.com/Gohla/pie/blob/master/pie/src/context/mod.rs).

We published a summary/abstract paper as well:

- [Precise, Efficient, and Expressive Incremental Build Scripts with PIE](https://gkonat.github.io/assets/publication/pie-ic19.pdf).

Two master students graduated on extensions to PIE:

- [Roelof Sol: Task Observability in Change Driven Incremental Build Systems with Dynamic Dependencies](https://repository.tudelft.nl/islandora/object/uuid%3A3bd052ee-b8a0-4687-85d0-ca6df0b07d0d). A problem with bottom-up builds is that tasks stay in the dependency graph forever, even if they are no longer needed. Even though those tasks are not executed (because they are not needed), they do need to be _checked_ and increase the size of the dependency graph which in turn has overhead for several graph operations. To solve that problem, we introduce _task observability_.

  A task is observable if and only if it is _explicitly observed_ by the user of the build system through directly requiring (`Session::require`) the task, or if it is _implicitly observed_ by a require task dependency from another task. Otherwise, the task is _unobserved_. The build system updates the observability status of tasks while the build is executing.

  Unobserved tasks are _never checked_, removing the checking overhead. Unobserved tasks can be removed from the dependency graph in a "garbage collection" pass, removing graph operation overhead. Removing unobserved tasks is flexible: during the garbage collection pass you can decide to keep a task in the dependency graph if you think it will become observed again, to keep its cached output. You can also remove the provided (intermediate or output) files of an unobserved task to clean up disk space, which is correct due to the absence of hidden dependencies!

  Currently, observability is implemented in the Java implementation of PIE, but not yet in the Rust implementation of PIE.
- [Ivo Wilms: Extending the DSL for PIE](https://repository.tudelft.nl/islandora/object/uuid%3A567a7faf-1460-4348-8344-4746a18fb0b1). This improves and solves many problems in the original PIE DSL implementation. It introduces a module system, compatibility with dependency injection, and generics with subtyping into the DSL. Generics and subtyping have a proper type system implementation in the [Statix](https://spoofax.dev/references/statix/) meta-DSL.

One paper was published about using PIE:

- [Constructing Hybrid Incremental Compilers for Cross-Module Extensibility with an Internal Build System](https://programming-journal.org/2020/4/16/). This paper introduces a compiler design approach for reusing parts of a non-incremental to build an incremental compiler, using PIE to perform the incrementalization. The approach is applied to [Stratego](https://spoofax.dev/references/stratego/), a term transformation meta-DSL with several cross-cutting features that make incremental compilation hard. The result is the Stratego 2 compiler that is split up into multiple PIE tasks to do incremental parsing (per-file), incremental name analysis, and incremental compilation. Stratego 2 was also extended with [gradual typing](https://www.jeffsmits.net/assets/articles/sle20-paper4.pdf) at a later stage, where the gradual typing was also performed in PIE tasks.

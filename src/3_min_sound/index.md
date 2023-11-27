# Testing Incrementality and Correctness

So far, we have not fully tested whether our build system is incremental and correct.
We have reasoned that it is incremental and correct, implemented an example that seems to be incremental and correct, and have unit tests covering most individual components of the build system.
However, we have not yet performed _integration testing_, where all components of the build system are integrated and tested together.
Furthermore, we haven't really defined incremental and correct.
In this chapter, we will define those more concretely, do integration testing to test whether our build system really is incremental and correct, and (spoilers!) fix uncovered incrementality and correctness issues.

#### Incremental and Correct

In essence, a build system is _incremental_ (also called _minimal_) if it performs the _least amount of work possible_ to bring the system in a consistent state again after a change.
More concretely, an incremental build system _executes at most all inconsistent tasks_.
If it executes more tasks than necessary, it is not fully incremental.
A trivial way to be incremental is to never execute anything, but that is of course not correct.

On the other hand, a build system is _correct_ (also called [_sound_](https://en.wikipedia.org/wiki/Soundness)) if it performs _all work required_ to bring the system in a consistent state again after a change.
More concretely, a correct build system _executes at least all inconsistent tasks_.
If it executes fewer tasks than necessary, it is not correct.
A trivial way to be correct is to execute everything, but that in turn is not incremental.

Combining these definitions: a correct incremental build system _executes exactly all inconsistent tasks_.

Whether a task is inconsistent or not, is characterized by its _dependencies_.
A task is inconsistent when any of its dependencies are inconsistent, and consequently only consistent when all its dependencies are consistent.
A file dependency is inconsistent if its file stamp changes.
A task dependency is inconsistent if, after recursively checking the task, its output stamp changes.
An inconsistent task is made consistent by executing it, because executing it makes all its dependencies consistent!

_New tasks_ are tasks that have not yet been executed (no cached output), and are deemed inconsistent, and thus must be executed.
Once executed, they have had a chance to create dependencies, and are no longer new: their consistency then depends on the consistency of their dependencies.

```admonish info title="Tasks Without Dependencies" collapsible=true
Tasks without dependencies (that are not new) are forever deemed consistent, and never have to be re-executed.
This is rare in practice, but can be useful for one-time expensive calculations.
```

[//]: # (The recursive nature of checking task dependencies ensures that indirect changes can affect tasks and cause them to be correctly executed.)
By defining incremental and correct in terms of dependencies (through consistency), a task author forgetting to create a dependency or not choosing the correct stamper, does not change whether our build system is incremental and correct.
PIE works under the assumption that task authors correctly list all dependencies that mark their task as affected by a change when it actually is. 

```admonish info title="Preventing Task Authoring Mistakes" collapsible=true
It is of course possible to make mistakes when authoring tasks, for example by creating a dependency to the wrong file, or by forgetting to create a file dependency.
Unfortunately, there is no easy way to solve this.

We will be writing a build event tracking system later, for which we will make an implementation that writes the entire build log to standard output.
This build log can help debug mistakes by precisely showing what the build system is doing.

A technique to catch file dependency mistakes is by sandboxing the filesystem to only have access to files that have been required.
For example, Bazel can perform [sandboxing](https://bazel.build/docs/sandboxing), but it is not fully cross-platform, and still allows reading files from absolute paths.
If a cross-platform and bulletproof sandboxing library exists, it could help catch file dependency mistakes in programmatic incremental build systems.

Finally, the ultimate technique to catch file dependency mistakes is by automatically creating these dependencies using filesystem tracing, instead of having the task author make them.
For example, the [Rattle](https://github.com/ndmitchell/rattle) build system uses [fsatrace](https://github.com/jacereda/fsatrace) to automatically create file dependencies, freeing task authors from having to think about file dependencies
However, filesystem tracing is also not fully cross-platform and bulletproof, so it cannot always be used.
Again, if a cross-platform and bulletproof filesystem tracing library exists, it would be extremely useful for programmatic incremental build systems.
```

#### The Ever-Changing Filesystem

One issue with this definition is that we do not control the filesystem: changes to the filesystem can happen at any time during the build.
Therefore, we would need to constantly check file dependencies for consistency, and we can never be sure that a task is really consistent!
That makes incremental builds infeasible.

To solve that problem, we will introduce the concept of a _build session_ in which we only check tasks for consistency once.
Once a task has been executed or checked, we don't check it anymore that session, solving the problem of constantly having to check file dependencies.
A new session has to created to check those tasks again.
Therefore, sessions are typically short-lived, and are created whenever file changes should be detected again.

#### Integration Testing

In this chapter, we will show incrementality and correctness by integration testing.
However, this requires quite some setup, as testing incrementality requires checking whether tasks are executed or not.
Therefore, we will create an infrastructure for _tracking build events_ which we will use to test incrementality.

Then we will spend several sections writing integration tests to find issues, and fix them.

```admonish question title="Proving Incrementality and Correctness?" collapsible=true
While proving incrementality and correctness would be a very interesting exercise, I am not at all an expert in formal proofs in proof assistants such as [Coq](https://coq.inria.fr/), [Agda](https://wiki.portal.chalmers.se/agda/pmwiki.php), etc.
If that is something that interests you, do pursue it and get in touch!
```

We will continue as follows:

1) Introduce sessions and change the API to work with sessions: `Session` type for performing builds in a session, and the `Pie` type as the entry point that manages sessions. We do this first as it introduces API changes that would be annoying to deal with later.
2) Create infrastructure to track build events for testing and debugging purposes. Create the `Tracker` trait, and implement a `WritingTracker` for debugging and `EventTracker` for testing.
3) Create integration tests that test incrementality and correctness.
4) Find a bug where superfluous dependencies are being created, and fix it.
5) Find a soundness hole where multiple tasks write to the same file. Fix it by tracking file write dependencies separately from read dependencies, and catch these mistakes with dynamic verification.
6) Find a soundness hole where a task reads from a file before another task writes to it. Fix it by catching these mistakes with dynamic verification.
7) Find a soundness hole where cyclic task execution can still occur. Fix it by changing how task dependencies are stored.

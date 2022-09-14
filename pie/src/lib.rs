use std::any::Any;
use std::collections::hash_map::RandomState;
use std::collections::HashSet;
use std::error::Error;
use std::fmt::Debug;
use std::fs::File;
use std::hash::{BuildHasher, Hash};
use std::path::PathBuf;

use crate::runner::TopDownRunner;
use crate::store::{Store, TaskNode};
use crate::tracker::{NoopTracker, Tracker};
use crate::trait_object::DynTask;

pub mod prelude;
pub mod dependency;
pub mod runner;
pub mod store;
pub mod tracker;
pub mod task;
pub mod trait_object;

/// The unit of computation in a programmatic incremental build system.
pub trait Task: Eq + Hash + Clone + Any + Debug {
  /// The type of output this task produces when executed. Must implement [`Eq`], [`Clone`], and either not contain any 
  /// references, or only `'static` references.
  type Output: Output;
  /// Execute the task, with `context` providing a means to specify dependencies, producing an instance of 
  /// `Self::Output`.
  fn execute<C: Context>(&self, context: &mut C) -> Self::Output;
  #[inline]
  fn as_dyn(&self) -> &dyn DynTask {
    self as &dyn DynTask
  }
  #[inline]
  fn as_dyn_clone(&self) -> Box<dyn DynTask> {
    dyn_clone::clone_box(self.as_dyn())
  }
}


/// Trait alias for task outputs.
pub trait Output: Eq + Clone + Any + Debug {}

impl<T: Eq + Clone + Any + Debug> Output for T {}


/// Incremental context, mediating between tasks and executors, enabling tasks to dynamically create dependencies that 
/// executors check for consistency and use in incremental execution.
pub trait Context {
  /// Requires given `task`, creating a dependency to it, and returning its up-to-date output.
  fn require_task<T: Task>(&mut self, task: &T) -> T::Output;
  /// Requires file at given `path`, creating a read-dependency to the file by reading its content or metadata at the 
  /// time this function is called, and returning the opened file. Call this method *before reading from the file*.
  fn require_file(&mut self, path: &PathBuf) -> Result<File, std::io::Error>;
  /// Provides file at given `path`, creating a write-dependency to it by writing to its content or changing its
  /// metadata at the time this function is called. Call this method *after writing to the file*. This method does not 
  /// return the opened file, as it must be called *after writing to the file*.
  fn provide_file(&mut self, path: &PathBuf) -> Result<(), std::io::Error>;
}


/// Main entry point into the PIE build system.
#[derive(Debug)]
pub struct Pie<A = NoopTracker, H = RandomState> {
  store: Store<H>,
  tracker: A,
}

impl Default for Pie {
  #[inline]
  fn default() -> Self { Self { store: Store::default(), tracker: NoopTracker::default() } }
}

impl Pie {
  /// Creates a new [`Pie`] instance.
  #[inline]
  pub fn new() -> Self { Self::default() }
}

impl<A: Tracker + Default> Pie<A> {
  /// Creates a new [`Pie`] instance with given `tracker`.
  #[inline]
  pub fn with_tracker(tracker: A) -> Self { Self { store: Store::default(), tracker } }
}

impl<A: Tracker + Default, H: BuildHasher + Default> Pie<A, H> {
  /// Creates a new build session. Only one session may be active at once, enforced via mutable (exclusive) borrow.
  #[inline]
  pub fn new_session(&mut self) -> Session<A, H> { Session::new(self) }
  /// Runs `f` inside a new session.
  #[inline]
  pub fn run_in_session<R>(&mut self, f: impl FnOnce(Session<A, H>) -> R) -> R {
    let session = self.new_session();
    f(session)
  }

  /// Gets the [`Tracker`] instance.
  #[inline]
  pub fn tracker(&self) -> &A { &self.tracker }
  /// Gets the mutable [`Tracker`] instance.
  #[inline]
  pub fn tracker_mut(&mut self) -> &mut A { &mut self.tracker }
}


/// A session in which builds are executed. Every task is only executed once each session.
#[derive(Debug)]
pub struct Session<'p, A, H> {
  store: &'p mut Store<H>,
  tracker: &'p mut A,
  visited: HashSet<TaskNode, H>,
  dependency_check_errors: Vec<Box<dyn Error>>,
}

impl<'p, A: Tracker + Default, H: BuildHasher + Default> Session<'p, A, H> {
  #[inline]
  fn new(pie: &'p mut Pie<A, H>) -> Self {
    Self {
      store: &mut pie.store,
      tracker: &mut pie.tracker,
      visited: HashSet::default(),
      dependency_check_errors: Vec::default(),
    }
  }

  /// Requires given `task`, returning its up-to-date output.
  #[inline]
  pub fn require<T: Task>(&mut self, task: &T) -> T::Output {
    let mut runner = TopDownRunner::new(self);
    runner.require(task)
  }

  /// Gets the [`Tracker`] instance.
  #[inline]
  pub fn tracker(&self) -> &A { &self.tracker }
  /// Gets the mutable [`Tracker`] instance.
  #[inline]
  pub fn tracker_mut(&mut self) -> &mut A { &mut self.tracker }
  /// Gets a slice over all errors produced during dependency checks.
  #[inline]
  pub fn dependency_check_errors(&self) -> &[Box<dyn Error>] { &self.dependency_check_errors }
}

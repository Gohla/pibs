use std::any::Any;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

use dyn_clone::DynClone;

use crate::Context;

/// The unit of computation in the incremental build system.
pub trait Task: Eq + Hash + Clone + DynTask + Debug + 'static {
  /// The type of output this task produces when executed. Must implement `[Eq]`, `[Clone]`, and either not contain any 
  /// references, or only `'static` references.
  type Output: Eq + Clone + DynOutput + Debug + 'static;
  /// Execute the task, with `[context]` providing a means to specify dependencies, producing `[Self::Output]`.
  fn execute<C: Context>(&self, context: &mut C) -> Self::Output;

  #[inline]
  fn as_dyn(&self) -> &dyn DynTask { self as &dyn DynTask }
  #[inline]
  fn as_dyn_mut(&mut self) -> &mut dyn DynTask { self as &mut dyn DynTask }
  #[inline]
  fn clone_box_dyn(&self) -> Box<dyn DynTask> { self.as_dyn().clone() }
}

/// Object-safe version of [`Task`], enabling tasks to be used as trait objects.
pub trait DynTask: DynClone + Any + Debug + 'static {
  fn dyn_eq(&self, other: &dyn Any) -> bool;
  fn dyn_hash(&self, state: &mut dyn Hasher);
  fn as_any(&self) -> &dyn Any;
}

/// Alias trait for task outputs.
pub trait Output: Eq + Clone + Debug + 'static {}

impl<T: Eq + Clone + Debug + 'static> Output for T {}

/// Object-safe version of [`Output`].
pub trait DynOutput: DynClone + Any + Debug + 'static {
  fn dyn_eq(&self, other: &dyn Any) -> bool;
  fn as_any(&self) -> &dyn Any;
}


// DynTask implementations

// Implement DynTask for all `Task`s.
impl<T: Task> DynTask for T {
  #[inline]
  fn dyn_eq(&self, other: &dyn Any) -> bool {
    if let Some(other) = other.downcast_ref::<Self>() {
      self == other
    } else {
      false
    }
  }
  #[inline]
  fn dyn_hash(&self, mut state: &mut dyn Hasher) { self.hash(&mut state); }
  #[inline]
  fn as_any(&self) -> &dyn Any { self }
}

// Implement PartialEq/Eq/Hash/Clone for `dyn DynTask` or `Box<dyn DynTask>`
impl PartialEq for dyn DynTask {
  #[inline]
  fn eq(&self, other: &dyn DynTask) -> bool { self.dyn_eq(other.as_any()) }
}

impl Eq for dyn DynTask {}

impl Hash for dyn DynTask {
  #[inline]
  fn hash<H: Hasher>(&self, state: &mut H) { self.dyn_hash(state); }
}

impl Clone for Box<dyn DynTask> {
  fn clone(&self) -> Self {
    dyn_clone::clone_box(&**self)
  }
}

// Extension trait to enable calling `clone` on `dyn DynTask`.
pub trait DynTaskExt {
  fn clone(&self) -> Box<Self>;
}

impl DynTaskExt for dyn DynTask {
  fn clone(&self) -> Box<Self> {
    dyn_clone::clone_box(self)
  }
}


// DynOutput implementations

// Implement DynOutput for all `Output`s.
impl<T: Output> DynOutput for T {
  #[inline]
  fn dyn_eq(&self, other: &dyn Any) -> bool {
    if let Some(other) = other.downcast_ref::<Self>() {
      self == other
    } else {
      false
    }
  }
  #[inline]
  fn as_any(&self) -> &dyn Any { self }
}

// Implement PartialEq/Eq/Hash/Clone for `dyn DynOutput` or `Box<dyn DynOutput>`
impl PartialEq for dyn DynOutput {
  #[inline]
  fn eq(&self, other: &dyn DynOutput) -> bool { self.dyn_eq(other.as_any()) }
}

impl Eq for dyn DynOutput {}

impl Clone for Box<dyn DynOutput> {
  fn clone(&self) -> Self {
    dyn_clone::clone_box(&**self)
  }
}

// Extension trait to enable calling `clone` on `dyn DynOutput`.
pub trait DynOutputExt {
  fn clone(&self) -> Box<Self>;
}

impl DynOutputExt for dyn DynOutput {
  fn clone(&self) -> Box<Self> {
    dyn_clone::clone_box(self)
  }
}


/// Task that does nothing and returns `()`.
#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct NoopTask {}

impl Task for NoopTask {
  type Output = ();
  #[inline]
  fn execute<C: Context>(&self, _context: &mut C) -> Self::Output { () }
}

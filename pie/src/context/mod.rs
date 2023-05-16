use std::fs::File;
use std::hash::BuildHasher;
use std::path::Path;

use crate::{Session, Task};
use crate::dependency::{FileDependency, TaskDependency};
use crate::stamp::{FileStamper, OutputStamper};
use crate::store::TaskNode;
use crate::tracker::Tracker;

pub(crate) mod non_incremental;
pub(crate) mod bottom_up;
pub(crate) mod top_down;

struct ContextShared<'p, 's, T, O, A, H> {
  pub(crate) session: &'s mut Session<'p, T, O, A, H>,
  pub(crate) task_execution_stack: Vec<TaskNode>,
}

impl<'p, 's, T: Task, A: Tracker<T>, H: BuildHasher + Default> ContextShared<'p, 's, T, T::Output, A, H> {
  fn new(session: &'s mut Session<'p, T, T::Output, A, H>) -> Self {
    Self {
      session,
      task_execution_stack: Default::default(),
    }
  }

  fn require_file_with_stamper(&mut self, path: impl AsRef<Path>, stamper: FileStamper) -> Result<Option<File>, std::io::Error> {
    let path = path.as_ref();
    let (dependency, file) = FileDependency::new_with_file(path, stamper)?;
    self.session.tracker.require_file(&dependency);
    let node = self.session.store.get_or_create_file_node(path);
    if let Some(current_requiring_task_node) = self.task_execution_stack.last() {
      if let Some(providing_task_node) = self.session.store.get_task_providing_file(&node) {
        if !self.session.store.contains_transitive_task_dependency(current_requiring_task_node, &providing_task_node) {
          let current_requiring_task = self.session.store.get_task(current_requiring_task_node);
          let providing_task = self.session.store.get_task(&providing_task_node);
          panic!("Hidden dependency; file '{}' is required by the current task '{:?}' without a dependency to providing task: {:?}", path.display(), current_requiring_task, providing_task);
        }
      }
      self.session.store.add_file_require_dependency(current_requiring_task_node, &node, dependency);
    }
    Ok(file)
  }

  fn provide_file_with_stamper(&mut self, path: impl AsRef<Path>, stamper: FileStamper) -> Result<(), std::io::Error> {
    let path = path.as_ref();
    let dependency = FileDependency::new(path, stamper).map_err(|e| e.kind())?;
    self.session.tracker.provide_file(&dependency);
    let node = self.session.store.get_or_create_file_node(path);
    if let Some(current_providing_task_node) = self.task_execution_stack.last() {
      if let Some(previous_providing_task_node) = self.session.store.get_task_providing_file(&node) {
        let current_providing_task = self.session.store.get_task(current_providing_task_node);
        let previous_providing_task = self.session.store.get_task(&previous_providing_task_node);
        panic!("Overlapping provided file; file '{}' is provided by the current task '{:?}' that was previously provided by task: {:?}", path.display(), current_providing_task, previous_providing_task);
      }
      for (requiring_task_node, _) in self.session.store.get_tasks_requiring_file(&node) {
        if !self.session.store.contains_transitive_task_dependency(&requiring_task_node, current_providing_task_node) {
          let current_providing_task = self.session.store.get_task(current_providing_task_node);
          let requiring_task = self.session.store.get_task(&requiring_task_node);
          panic!("Hidden dependency; file '{}' is provided by the current task '{:?}' without a dependency from requiring task '{:?}' to the current providing task", path.display(), current_providing_task, requiring_task);
        }
      }
      self.session.store.add_file_provide_dependency(current_providing_task_node, &node, dependency);
    }
    Ok(())
  }

  /// Reserve a task require dependency, detecting cycles before we execute, preventing infinite recursion/loops.
  fn reserve_task_require_dependency(&mut self, task: &T, node: &TaskNode) {
    if let Some(current_task_node) = self.task_execution_stack.last() {
      if let Err(pie_graph::Error::CycleDetected) = self.session.store.reserve_task_require_dependency(current_task_node, node) {
        let current_task = self.session.store.get_task(current_task_node);
        let task_stack: Vec<_> = self.task_execution_stack.iter().map(|task_node| self.session.store.get_task(task_node)).collect();
        panic!("Cyclic task dependency; current task '{:?}' is requiring task '{:?}' which was already required. Task stack: {:?}", current_task, task, task_stack);
      }
    }
  }

  /// Update the reserved task require dependency with an actual task dependency.
  fn update_reserved_task_require_dependency(&mut self, task: T, node: &TaskNode, output: T::Output, stamper: OutputStamper) {
    if let Some(current_task_node) = self.task_execution_stack.last() {
      let dependency = TaskDependency::new(task, stamper, output);
      self.session.store.update_reserved_task_require_dependency(current_task_node, node, dependency);
    }
  }

  fn pre_execute(&mut self, task: &T, node: TaskNode) {
    self.task_execution_stack.push(node);
    self.session.tracker.execute_task_start(task);
  }

  fn post_execute(&mut self, task: &T, node: TaskNode, output: &T::Output) {
    self.session.tracker.execute_task_end(task, output);
    self.task_execution_stack.pop();
    self.session.store.set_task_output(&node, output.clone());
  }

  #[inline]
  fn default_output_stamper(&self) -> OutputStamper { OutputStamper::Equals }
  #[inline]
  fn default_require_file_stamper(&self) -> FileStamper { FileStamper::Modified }
  #[inline]
  fn default_provide_file_stamper(&self) -> FileStamper { FileStamper::Modified }
}

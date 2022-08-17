use std::error::Error;
use std::fs::File;
use std::path::PathBuf;

use incremental_topo::Node;

use crate::{Context, DynTask, FileDependency, Task, TaskDependency};
use crate::runner::store::{Store, TaskNode};

/// Incremental runner that checks dependencies recursively in a top-down manner.
pub struct TopDownRunner {
  store: Store<Self>,
  task_execution_stack: Vec<TaskNode>,
  dependency_check_errors: Vec<Box<dyn Error>>,
}

impl TopDownRunner {
  /// Creates a new `[TopDownRunner]`.
  pub fn new() -> Self {
    Self {
      store: Store::new(),
      task_execution_stack: Vec::new(),
      dependency_check_errors: Vec::new(),
    }
  }

  /// Requires given `[task]`, returning its up-to-date output, or an error indicating failure to check consistency of 
  /// one or more dependencies.
  pub fn require_initial<T: Task>(&mut self, task: &T) -> Result<T::Output, (T::Output, &[Box<dyn Error>])> {
    self.task_execution_stack.clear();
    self.dependency_check_errors.clear();
    let output = self.require_task::<T>(task);
    if self.dependency_check_errors.is_empty() {
      Ok(output)
    } else {
      Err((output, &self.dependency_check_errors))
    }
  }
}

impl Context for TopDownRunner {
  fn require_task<T: Task>(&mut self, task: &T) -> T::Output {
    let task_node = self.store.get_or_create_node_by_task(Box::new(task.clone()) as Box<dyn DynTask>);
    if self.should_execute_task(task_node) { // Execute the task, cache and return up-to-date output.
      self.store.reset_task(&task_node);
      // Check for cyclic dependency
      if let Some(current_task_node) = self.task_execution_stack.last() {
        if let Err(incremental_topo::Error::CycleDetected) = self.store.add_task_dependency_edge(*current_task_node, task_node) {
          let current_task = self.store.get_task_by_node(current_task_node);
          panic!("Cyclic task dependency; task {:?} required task {:?} which was already required. Task stack: {:?}", current_task, task, self.task_execution_stack);
        }
      }
      // Execute task
      self.task_execution_stack.push(task_node);
      let output = task.execute(self);
      self.task_execution_stack.pop();
      // Store dependency and output.
      if let Some(current_task_node) = self.task_execution_stack.last() {
        self.store.add_to_dependencies_of_task(*current_task_node, Box::new(TaskDependency::new(task.clone(), output.clone())));
      }
      self.store.set_task_output(task.clone(), output.clone());
      output
    } else { // Return already up-to-date output.
      // Unwrap OK: if we should not execute the task, it must have been executed before, and therefore it has an output.
      let output = self.store.get_task_output::<T>(task).unwrap().clone();
      output
    }
  }

  fn require_file(&mut self, path: &PathBuf) -> Result<File, std::io::Error> {
    let file_node = self.store.get_or_create_file_node(path);
    let dependency = FileDependency::new(path.clone()).map_err(|e| e.kind())?;
    let opened = dependency.open();
    if let Some(current_task_node) = self.task_execution_stack.last() {
      if let Some(providing_task) = self.store.get_providing_task(&file_node) {
        if !self.store.contains_transitive_task_dependency(current_task_node, providing_task) {
          panic!("Hidden dependency; file {:?} is provided by task {:?} without a dependency from the current task {:?} to the provider", path, providing_task, current_task_node);
        }
      }
      self.store.add_file_require_dependency(*current_task_node, file_node, dependency);
    }
    opened
  }

  fn provide_file(&mut self, path: &PathBuf) -> Result<(), std::io::Error> {
    let file_node = self.store.get_or_create_file_node(path);
    let dependency = FileDependency::new(path.clone()).map_err(|e| e.kind())?;
    if let Some(current_task_node) = self.task_execution_stack.last() {
      if let Some(providing_task) = self.store.get_providing_task(&file_node) {
        panic!("Overlapping provided file; file {:?} is already provided by task {:?}", path, providing_task);
      }
      if let Some(requiring_tasks) = self.store.get_requiring_tasks(&file_node) {
        for requiring_task in requiring_tasks {
          if !self.store.contains_transitive_task_dependency(requiring_task, current_task_node) {
            panic!("Hidden dependency; file {:?} is provided by the current task {:?} without a dependency from task {:?} that requires the file to the current task", path, current_task_node, requiring_task);
          }
        }
      }
      self.store.add_file_provide_dependency(*current_task_node, file_node, dependency);
    }
    Ok(())
  }
}

impl TopDownRunner {
  fn should_execute_task(&mut self, task_node: Node) -> bool {
    // Remove task dependencies so that we get ownership over the list of dependencies. If the task does not need to be
    // executed, we will restore the dependencies again.
    let task_dependencies = self.store.remove_dependencies_of_task(&task_node);
    if let Some(task_dependencies) = task_dependencies {
      // Task has been executed before, check whether all its dependencies are still consistent. If one or more are not,
      // we need to execute the task.
      for task_dependency in &task_dependencies {
        match task_dependency.is_consistent(self) {
          Ok(false) => return true, // Not consistent -> should execute task.
          Err(e) => { // Error -> store error and assume not consistent -> should execute task.
            self.dependency_check_errors.push(e);
            return true;
          }
          _ => {} // Continue to check other dependencies.
        }
      }
      // Task is consistent and does not need to be executed. Restore the previous dependencies.
      self.store.set_dependencies_of_task(task_node, task_dependencies); // OPTO: removing and inserting into a HashMap due to ownership requirements.
      false
    } else {
      // Task has not been executed before, therefore we need to execute it.
      true
    }
  }
}

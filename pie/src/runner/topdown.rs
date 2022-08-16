use std::error::Error;
use std::fs::File;
use std::path::PathBuf;

use incremental_topo::Node;

use crate::{Context, DynTask, FileDependency, Task, TaskDependency};
use crate::runner::store::Store;

/// Incremental runner that checks dependencies recursively in a top-down manner.
pub struct TopDownRunner {
  store: Store<Self>,
  task_execution_stack: Vec<Node>,
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
    if let Some(current_task_node) = self.task_execution_stack.last() {
      // TODO: can we detect a cycle by first removing dependencies, and checking if that is already in the stack?
      if let Err(incremental_topo::Error::CycleDetected) = self.store.graph.add_dependency(current_task_node, task_node) {
        let current_task = self.store.get_task_by_node(current_task_node);
        panic!("Cyclic task dependency; task {:?} required task {:?} which was already required. Task stack: {:?}", current_task, task, self.task_execution_stack);
      }
    }
    if self.should_execute_task(task_node) {
      // TODO: remove from task_to_required_files/file_to_requiring_tasks/task_to_provided_file/file_to_providing_task and update graph
      // TODO: should also delete dependencies from the graph!

      self.task_execution_stack.push(task_node);
      let output = task.execute(self);
      self.task_execution_stack.pop();
      if let Some(current_task_node) = self.task_execution_stack.last() {
        self.store.add_to_task_dependencies(*current_task_node, Box::new(TaskDependency::new(task.clone(), output.clone())));
      }
      self.store.set_task_output(task.clone(), output.clone());
      output
    } else {
      // Assume: if we should not execute the task, it must have been executed before, and therefore it has an output.
      let output = self.store.get_task_output::<T>(task).unwrap().clone();
      output
    }
  }

  fn require_file(&mut self, path: &PathBuf) -> Result<File, std::io::Error> {
    let file_node = self.store.get_or_create_file_node(path);
    // TODO: hidden dependency detection
    let dependency = FileDependency::new(path.clone()).map_err(|e| e.kind())?;
    let opened = dependency.open();
    if let Some(current_task_node) = self.task_execution_stack.last() {
      let current_task_node = *current_task_node;
      self.store.graph.add_dependency(current_task_node, file_node).ok(); // Ignore error OK: cycles cannot occur from task to file dependencies, as files do not have dependencies.
      self.store.task_to_required_files.entry(current_task_node).or_insert_with(|| Vec::with_capacity(1)).push(file_node);
      self.store.file_to_requiring_tasks.entry(file_node).or_insert_with(|| Vec::with_capacity(1)).push(current_task_node);
      self.store.add_to_task_dependencies(current_task_node, Box::new(dependency));
    }
    opened
  }

  fn provide_file(&mut self, path: &PathBuf) -> Result<(), std::io::Error> {
    let file_node = self.store.get_or_create_file_node(path);
    // TODO: hidden dependency detection
    // TODO: overlapping provided file detection
    let dependency = FileDependency::new(path.clone()).map_err(|e| e.kind())?;
    if let Some(current_task_node) = self.task_execution_stack.last() {
      let current_task_node = *current_task_node;
      self.store.graph.add_dependency(current_task_node, file_node).ok(); // Ignore error OK: cycles cannot occur from task to file dependencies, as files do not have dependencies.
      self.store.file_to_providing_task.insert(file_node, current_task_node);
      self.store.add_to_task_dependencies(current_task_node, Box::new(dependency));
    }
    Ok(())
  }
}

impl TopDownRunner {
  fn should_execute_task(&mut self, task_node: Node) -> bool {
    // Remove task dependencies so that we get ownership over the list of dependencies. If the task does not need to be
    // executed, we will restore the dependencies again.
    let task_dependencies = self.store.remove_task_dependencies(&task_node);
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
          _ => {}, // Continue to check other dependencies.
        }
      }
      // Task is consistent and does not need to be executed. Restore the previous dependencies.
      self.store.set_task_dependencies(task_node, task_dependencies); // OPTO: removing and inserting into a HashMap due to ownership requirements.
      false
    } else {
      // Task has not been executed before, therefore we need to execute it.
      true
    }
  }
}

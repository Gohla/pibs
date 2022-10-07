use std::collections::HashMap;
use std::hash::BuildHasher;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use pie_graph::{DAG, Node};

use crate::dependency::FileDependency;
use crate::Task;
use crate::trait_object::{DynDependency, DynOutput, DynTask};

pub type TaskNode = Node;
pub type FileNode = Node;

#[derive(Serialize, Deserialize, Debug)]
pub struct Store<H> {
  #[serde(bound(
  serialize = "H: BuildHasher + Default, DAG<NodeData, ParentData, ChildData, H>: serde::Serialize",
  deserialize = "H: BuildHasher + Default, DAG<NodeData, ParentData, ChildData, H>: serde::Deserialize<'de>"
  ))] // Set bounds such that `H` does not have to be (de)serializable
  graph: DAG<NodeData, ParentData, ChildData, H>,
  #[serde(bound(
  serialize = "H: BuildHasher + Default, HashMap<Box<dyn DynTask>, TaskNode, H>: serde::Serialize",
  deserialize = "H: BuildHasher + Default, HashMap<Box<dyn DynTask>, TaskNode, H>: serde::Deserialize<'de>"
  ))] // Set bounds such that `H` does not have to be (de)serializable
  #[serde(skip)]
  task_to_node: HashMap<Box<dyn DynTask>, TaskNode, H>,
  #[serde(bound(
  serialize = "H: BuildHasher + Default, HashMap<PathBuf, FileNode, H>: serde::Serialize",
  deserialize = "H: BuildHasher + Default, HashMap<PathBuf, FileNode, H>: serde::Deserialize<'de>"
  ))] // Set bounds such that `H` does not have to be (de)serializable
  #[serde(skip)]
  file_to_node: HashMap<PathBuf, FileNode, H>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) enum NodeData {
  Task {
    #[serde(with = "task_serde")]
    task: Box<dyn DynTask>,
    #[serde(skip)]
    dependencies: Option<Vec<Box<dyn DynDependency>>>,
    #[serde(skip)]
    output: Option<Box<dyn DynOutput>>,
  },
  File(PathBuf),
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Debug)]
pub(crate) enum ParentData {
  FileRequiringTask,
  FileProvidingTask,
  TaskRequiringTask,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Debug)]
pub(crate) enum ChildData {
  RequireFile,
  ProvideFile,
  RequireTask,
}

impl<H: BuildHasher + Default> Default for Store<H> {
  #[inline]
  fn default() -> Self {
    Self {
      graph: DAG::with_default_hasher(),
      task_to_node: HashMap::default(),
      file_to_node: HashMap::default(),
    }
  }
}

impl<H: BuildHasher + Default> Store<H> {
  #[inline]
  pub fn get_or_create_node_by_task(&mut self, task: Box<dyn DynTask>) -> TaskNode {
    if let Some(node) = self.task_to_node.get(&task) {
      *node
    } else {
      let node = self.graph.add_node(NodeData::Task {
        task: task.clone(),
        dependencies: None,
        output: None,
      });
      self.task_to_node.insert(task, node);
      node
    }
  }
  #[inline]
  pub fn get_task_by_node(&self, task_node: &TaskNode) -> Option<&Box<dyn DynTask>> {
    self.graph.get_node_data(task_node).and_then(|d| match d {
      NodeData::Task { task, .. } => Some(task),
      _ => None
    })
  }

  #[inline]
  pub fn task_by_node(&self, task_node: &TaskNode) -> &Box<dyn DynTask> {
    self.get_task_by_node(task_node).unwrap()
  }


  #[inline]
  pub fn get_or_create_file_node(&mut self, path: &PathBuf) -> FileNode {
    // TODO: normalize path?
    if let Some(file_node) = self.file_to_node.get(path) {
      *file_node
    } else {
      let node = self.graph.add_node(NodeData::File(path.clone()));
      self.file_to_node.insert(path.clone(), node);
      node
    }
  }


  #[inline]
  pub fn add_task_dependency_edge(&mut self, depender_task_node: TaskNode, dependee_task_node: TaskNode) -> Result<bool, pie_graph::Error> {
    self.graph.add_dependency(depender_task_node, dependee_task_node, ParentData::TaskRequiringTask, ChildData::RequireTask)
  }


  #[inline]
  pub fn remove_dependencies_of_task(&mut self, task_node: &TaskNode) -> Option<Vec<Box<dyn DynDependency>>> {
    if let Some(NodeData::Task { dependencies, .. }) = self.graph.get_node_data_mut(task_node) {
      std::mem::take(dependencies)
    } else {
      None
    }
  }
  #[inline]
  pub fn set_dependencies_of_task(&mut self, task_node: TaskNode, new_dependencies: Vec<Box<dyn DynDependency>>) {
    if let Some(NodeData::Task { ref mut dependencies, .. }) = self.graph.get_node_data_mut(task_node) {
      std::mem::swap(dependencies, &mut Some(new_dependencies));
    }
  }
  #[inline]
  pub fn add_to_dependencies_of_task(&mut self, task_node: TaskNode, dependency: Box<dyn DynDependency>) {
    if let Some(NodeData::Task { ref mut dependencies, .. }) = self.graph.get_node_data_mut(task_node) {
      if let Some(dependencies) = dependencies {
        dependencies.push(dependency);
      } else {
        *dependencies = Some(vec![dependency]);
      }
    }
  }


  #[inline]
  pub fn task_has_output(&self, task_node: TaskNode) -> bool {
    if let Some(NodeData::Task { output: Some(_), .. }) = self.graph.get_node_data(task_node) {
      true
    } else {
      false
    }
  }
  #[inline]
  pub fn get_task_output<T: Task>(&self, task_node: TaskNode) -> Option<&T::Output> {
    if let Some(NodeData::Task { output: Some(output), .. }) = self.graph.get_node_data(task_node) {
      T::downcast_ref_output(output)
    } else {
      None
    }
  }
  #[inline]
  pub fn set_task_output<T: Task>(&mut self, task_node: TaskNode, new_output: T::Output) {
    if let Some(NodeData::Task { output, .. }) = self.graph.get_node_data_mut(task_node) {
      if let Some(output) = output {
        if let Some(output) = T::downcast_mut_output(output) {
          *output = new_output; // Replace the value inside the box.
        } else { // Stored output is not of the correct type any more, replace it with a new boxed output.
          *output = Box::new(new_output);
        }
      } else { // No output was stored yet, create a new boxed output.
        *output = Some(Box::new(new_output));
      }
    }
  }


  #[inline]
  pub fn add_file_require_dependency(&mut self, depender_task_node: TaskNode, dependee_file_node: FileNode, dependency: FileDependency) {
    self.graph.add_dependency(depender_task_node, dependee_file_node, ParentData::FileRequiringTask, ChildData::RequireFile).ok(); // Ignore error OK: cycles cannot occur from task to file dependencies, as files do not have dependencies.
    self.add_to_dependencies_of_task(depender_task_node, Box::new(dependency));
  }
  #[inline]
  pub fn add_file_provide_dependency(&mut self, depender_task_node: TaskNode, dependee_file_node: FileNode, dependency: FileDependency) {
    self.graph.add_dependency(depender_task_node, dependee_file_node, ParentData::FileProvidingTask, ChildData::ProvideFile).ok(); // Ignore error OK: cycles cannot occur from task to file dependencies, as files do not have dependencies.
    self.add_to_dependencies_of_task(depender_task_node, Box::new(dependency));
  }


  #[inline]
  pub fn reset_task(&mut self, task_node: &TaskNode) {
    for dependee in self.graph.get_outgoing_dependency_nodes(task_node).cloned().collect::<Vec<_>>() { // OPTO: reuse allocation
      self.graph.remove_dependency(task_node, dependee);
    }
  }


  #[inline]
  pub fn get_providing_task_node(&self, file_node: &FileNode) -> Option<&TaskNode> {
    self.graph.get_incoming_dependencies(file_node).filter_map(|(n, pe)| if pe == &ParentData::FileProvidingTask { Some(n) } else { None }).next()
  }
  #[inline]
  pub fn get_requiring_task_nodes<'a>(&'a self, file_node: &'a FileNode) -> impl Iterator<Item=&TaskNode> + '_ {
    self.graph.get_incoming_dependencies(file_node).filter_map(|(n, pe)| if pe == &ParentData::FileRequiringTask { Some(n) } else { None })
  }
  #[inline]
  pub fn contains_transitive_task_dependency(&self, depender_task_node: &TaskNode, dependee_task_node: &TaskNode) -> bool {
    self.graph.contains_transitive_dependency(depender_task_node, dependee_task_node)
  }
}

mod task_serde {
  use serde::{Deserializer, Serializer};

  use pie_tagged_serde::{deserialize_tagged, serialize_tagged};

  use crate::DynTask;

  pub(crate) fn serialize<S: Serializer>(task: &Box<dyn DynTask>, serializer: S) -> Result<S::Ok, S::Error> {
    serialize_tagged(task.as_ref(), serializer)
  }

  pub(crate) fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Box<dyn DynTask>, D::Error> {
    deserialize_tagged(deserializer)
  }
}


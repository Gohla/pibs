use std::fmt::Debug;
use std::fs::File;
use std::io::{Read, Stdout, Write};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tempfile::TempDir;

use pie::{Context, Task};
use pie::stamp::FileStamper;
use pie::tracker::CompositeTracker;
use pie::tracker::event::EventTracker;
use pie::tracker::writing::WritingTracker;

// Helper functions

pub type Tracker<T> = CompositeTracker<EventTracker<T>, WritingTracker<Stdout, T>>;

pub fn create_tracker<T: Task>() -> Tracker<T> {
  CompositeTracker(EventTracker::default(), WritingTracker::new_stdout_writer())
}

pub type Pie<T> = pie::Pie<T, Tracker<T>>;

pub fn create_pie<T: Task>() -> Pie<T> {
  Pie::with_tracker(create_tracker())
}

pub fn temp_dir() -> TempDir {
  tempfile::tempdir().expect("failed to create temporary directory")
}


// Helper traits

pub trait CheckErrorExt<T> {
  fn check(self) -> T;
}

impl<T: Debug> CheckErrorExt<T> for Result<T, std::io::Error> {
  fn check(self) -> T {
    self.expect("failed io operation on file")
  }
}

impl<T: Debug> CheckErrorExt<T> for Result<T, std::io::ErrorKind> {
  fn check(self) -> T {
    self.expect("failed io operation on file")
  }
}

impl<T: Debug> CheckErrorExt<T> for Result<T, ()> {
  fn check(self) -> T {
    self.expect("something failed")
  }
}

impl CheckErrorExt<()> for CommonOutput {
  fn check(self) -> () {
    match self {
      CommonOutput::ReadStringFromFile(r) => { r.check(); }
      CommonOutput::WriteStringToFile(r) => { r.check(); }
      CommonOutput::ListDirectory(r) => { r.check(); }
      CommonOutput::ToLowerCase(r) => { r.check(); }
      CommonOutput::ToUpperCase(r) => { r.check(); }
      _ => {}
    };
    ()
  }
}


// Pseudo-tasks

// Read string from file task

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize, Debug)]
pub struct ReadStringFromFile(pub PathBuf, pub FileStamper);

impl ReadStringFromFile {
  fn execute<T: Task, C: Context<T>>(&self, context: &mut C) -> Result<String, ()> {
    let mut string = String::new();
    if let Some(mut file) = context.require_file_with_stamper(&self.0, self.1).map_err(|_| ())? {
      file.read_to_string(&mut string).map_err(|_| ())?;
    }
    Ok(string)
  }
}

// Read indirect string from file task

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize, Debug)]
pub struct ReadIndirectStringFromFile(pub PathBuf, pub FileStamper);

impl ReadIndirectStringFromFile {
  fn execute<T: Task, C: Context<T>>(&self, context: &mut C) -> Result<String, ()> {
    let mut string = String::new();
    if let Some(mut file) = context.require_file_with_stamper(&self.0, self.1).map_err(|_| ())? {
      let mut indirect_path = String::new();
      file.read_to_string(&mut indirect_path).map_err(|_| ())?;
      let indirect_path = PathBuf::from(indirect_path);
      if let Some(mut file) = context.require_file_with_stamper(&indirect_path, self.1).map_err(|_| ())? {
        file.read_to_string(&mut string).map_err(|_| ())?;
      }
    }
    Ok(string)
  }
}

// Write string to file task

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize, Debug)]
pub struct WriteStringToFile(pub Box<CommonTask>, pub PathBuf, pub FileStamper);

impl WriteStringToFile {
  fn execute<C: Context<CommonTask>>(&self, context: &mut C) -> Result<(), ()> {
    let string = context.require_task(self.0.as_ref()).into_string()?;
    let mut file = File::create(&self.1).map_err(|_| ())?;
    file.write_all(string.as_bytes()).map_err(|_| ())?;
    context.provide_file_with_stamper(&self.1, self.2).map_err(|_| ())?;
    Ok(())
  }
}

// List directory

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize, Debug)]
pub struct ListDirectory(pub PathBuf, pub FileStamper);

impl ListDirectory {
  fn execute<T: Task, C: Context<T>>(&self, context: &mut C) -> Result<String, ()> {
    context.require_file_with_stamper(&self.0, self.1).map_err(|_| ())?;
    let paths = std::fs::read_dir(&self.0).map_err(|_| ())?;
    let paths: String = paths
      .into_iter()
      .map(|p| p.unwrap().path().to_string_lossy().to_string())
      .fold(String::new(), |a, b| a + &b + "\n");
    Ok(paths)
  }
}

// Make string lowercase

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize, Debug)]
pub struct ToLowerCase(pub Box<CommonTask>);

impl ToLowerCase {
  fn execute<C: Context<CommonTask>>(&self, context: &mut C) -> Result<String, ()> {
    let string = context.require_task(self.0.as_ref()).into_string()?;
    Ok(string.to_lowercase())
  }
}

// Make string uppercase

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize, Debug)]
pub struct ToUpperCase(pub Box<CommonTask>);

impl ToUpperCase {
  fn execute<C: Context<CommonTask>>(&self, context: &mut C) -> Result<String, ()> {
    let string = context.require_task(self.0.as_ref()).into_string()?;
    Ok(string.to_uppercase())
  }
}

// Require a task when a file exists

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize, Debug)]
pub struct RequireTaskOnFileExists(pub Box<CommonTask>, pub PathBuf);

impl RequireTaskOnFileExists {
  fn execute<C: Context<CommonTask>>(&self, context: &mut C) -> Result<(), ()> {
    if let Some(_) = context.require_file_with_stamper(&self.1, FileStamper::Exists).map_err(|_| ())? {
      context.require_task(&self.0).into_result()?;
    }
    Ok(())
  }
}

// Sequence

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize, Debug)]
pub struct Sequence(pub Vec<Box<CommonTask>>);

impl Sequence {
  fn execute<C: Context<CommonTask>>(&self, context: &mut C) -> Result<(), ()> {
    for task in &self.0 {
      context.require_task(task).into_result()?;
    }
    Ok(())
  }
}


// Common task

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize, Debug)]
pub enum CommonTask {
  StringConstant(String),
  ReadStringFromFile(ReadStringFromFile),
  ReadIndirectStringFromFile(ReadIndirectStringFromFile),
  WriteStringToFile(WriteStringToFile),
  ListDirectory(ListDirectory),
  ToLowerCase(ToLowerCase),
  ToUpperCase(ToUpperCase),
  RequireTaskOnFileExists(RequireTaskOnFileExists),
  Sequence(Sequence),
  RequireSelf,
  RequireCycleA,
  RequireCycleB,
}

#[allow(clippy::wrong_self_convention)]
#[allow(dead_code)]
impl CommonTask {
  pub fn string_constant(string: impl Into<String>) -> Self {
    Self::StringConstant(string.into())
  }
  pub fn read_string_from_file(path: impl Into<PathBuf>, stamper: FileStamper) -> Self {
    Self::ReadStringFromFile(ReadStringFromFile(path.into(), stamper))
  }
  pub fn read_indirect_string_from_file(path: impl Into<PathBuf>, stamper: FileStamper) -> Self {
    Self::ReadIndirectStringFromFile(ReadIndirectStringFromFile(path.into(), stamper))
  }
  pub fn write_string_to_file(string_provider: impl Into<Box<CommonTask>>, path: impl Into<PathBuf>, stamper: FileStamper) -> Self {
    Self::WriteStringToFile(WriteStringToFile(string_provider.into(), path.into(), stamper))
  }
  pub fn write_constant_string_to_file(string: impl Into<String>, path: impl Into<PathBuf>, stamper: FileStamper) -> Self {
    Self::WriteStringToFile(WriteStringToFile(Box::new(CommonTask::string_constant(string)), path.into(), stamper))
  }
  pub fn list_directory(path: impl Into<PathBuf>, stamper: FileStamper) -> Self {
    Self::ListDirectory(ListDirectory(path.into(), stamper))
  }
  pub fn to_lower_case(string_provider: impl Into<Box<CommonTask>>) -> Self {
    Self::ToLowerCase(ToLowerCase(string_provider.into()))
  }
  pub fn to_lower_case_constant(string: impl Into<String>) -> Self {
    Self::ToLowerCase(ToLowerCase(Box::new(Self::string_constant(string))))
  }
  pub fn to_upper_case(string_provider: impl Into<Box<CommonTask>>) -> Self {
    Self::ToUpperCase(ToUpperCase(string_provider.into()))
  }
  pub fn require_task_on_file_exists(task: impl Into<Box<CommonTask>>, path: impl Into<PathBuf>) -> Self {
    Self::RequireTaskOnFileExists(RequireTaskOnFileExists(task.into(), path.into()))
  }
  pub fn sequence(tasks: impl Into<Vec<CommonTask>>) -> Self {
    let tasks: Vec<Box<CommonTask>> = tasks.into().into_iter().map(|t| Box::new(t)).collect();
    Self::Sequence(Sequence(tasks))
  }

  pub fn require_self() -> Self {
    Self::RequireSelf
  }
  pub fn require_cycle_a() -> Self {
    Self::RequireCycleA
  }
  pub fn require_cycle_b() -> Self {
    Self::RequireCycleB
  }
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize, Debug)]
pub enum CommonOutput {
  StringConstant(String),
  ReadStringFromFile(Result<String, ()>),
  WriteStringToFile(Result<(), ()>),
  ListDirectory(Result<String, ()>),
  ToLowerCase(Result<String, ()>),
  ToUpperCase(Result<String, ()>),
  RequireTaskOnFileExists(Result<(), ()>),
  Sequence(Result<(), ()>),
}

#[allow(clippy::wrong_self_convention)]
#[allow(dead_code)]
impl CommonOutput {
  pub fn string_constant(string: impl Into<String>) -> Self { Self::StringConstant(string.into()) }
  pub fn read_string_from_file(result: Result<String, ()>) -> Self { Self::ReadStringFromFile(result) }
  pub fn read_string_from_file_ok(string: impl Into<String>) -> Self { Self::read_string_from_file(Ok(string.into())) }
  pub fn write_string_to_file(result: Result<(), ()>) -> Self { Self::WriteStringToFile(result) }
  pub fn write_string_to_file_ok() -> Self { Self::WriteStringToFile(Ok(())) }
  pub fn list_directory(result: Result<String, ()>) -> Self { Self::ListDirectory(result) }
  pub fn list_directory_ok(string: impl Into<String>) -> Self { Self::list_directory(Ok(string.into())) }
  pub fn to_lower_case(result: impl Into<Result<String, ()>>) -> Self { Self::ToLowerCase(result.into()) }
  pub fn to_lower_case_ok(string: impl Into<String>) -> Self { Self::ToLowerCase(Ok(string.into())) }
  pub fn to_upper_case(result: impl Into<Result<String, ()>>) -> Self { Self::ToUpperCase(result.into()) }
  pub fn to_upper_case_ok(string: impl Into<String>) -> Self { Self::ToUpperCase(Ok(string.into())) }
  pub fn require_task_on_file_exists(result: Result<(), ()>) -> Self { Self::RequireTaskOnFileExists(result) }
  pub fn require_task_on_file_exists_ok() -> Self { Self::RequireTaskOnFileExists(Ok(())) }
  pub fn sequence(result: Result<(), ()>) -> Self { Self::Sequence(result) }
  pub fn sequence_ok() -> Self { Self::Sequence(Ok(())) }

  pub fn into_string(self) -> Result<String, ()> {
    use CommonOutput::*;
    let string = match self {
      StringConstant(s) => s,
      ReadStringFromFile(r) => r?,
      ListDirectory(r) => r?,
      ToLowerCase(r) => r?,
      ToUpperCase(r) => r?,
      o => panic!("Output {:?} does not contain a string", o),
    };
    Ok(string)
  }
  pub fn into_result(self) -> Result<(), ()> {
    use CommonOutput::*;
    match self {
      StringConstant(_) => Ok(()),
      ReadStringFromFile(r) => r.map(|_| ()),
      WriteStringToFile(r) => r,
      ListDirectory(r) => r.map(|_| ()),
      ToLowerCase(r) => r.map(|_| ()),
      ToUpperCase(r) => r.map(|_| ()),
      RequireTaskOnFileExists(r) => r,
      Sequence(r) => r,
    }
  }
}

impl Task for CommonTask {
  type Output = CommonOutput;

  fn execute<C: Context<Self>>(&self, context: &mut C) -> Self::Output {
    match self {
      CommonTask::StringConstant(s) => CommonOutput::StringConstant(s.clone()),
      CommonTask::ReadStringFromFile(task) => CommonOutput::ReadStringFromFile(task.execute(context)),
      CommonTask::ReadIndirectStringFromFile(task) => CommonOutput::ReadStringFromFile(task.execute(context)),
      CommonTask::WriteStringToFile(task) => CommonOutput::WriteStringToFile(task.execute(context)),
      CommonTask::ListDirectory(task) => CommonOutput::ListDirectory(task.execute(context)),
      CommonTask::ToLowerCase(task) => CommonOutput::ToLowerCase(task.execute(context)),
      CommonTask::ToUpperCase(task) => CommonOutput::ToUpperCase(task.execute(context)),
      CommonTask::RequireTaskOnFileExists(task) => CommonOutput::RequireTaskOnFileExists(task.execute(context)),
      CommonTask::Sequence(task) => CommonOutput::Sequence(task.execute(context)),
      CommonTask::RequireSelf => context.require_task(&CommonTask::RequireSelf),
      CommonTask::RequireCycleA => context.require_task(&CommonTask::RequireCycleB),
      CommonTask::RequireCycleB => context.require_task(&CommonTask::RequireCycleA),
    }
  }
}


use std::fs;

use ron::{Deserializer, Serializer};
use ron::ser::PrettyConfig;
use rstest::{fixture, rstest};
use tempfile::TempDir;

use ::pie::stamp::FileStamper;

use crate::common::{CheckErrorExt, CommonTask, Pie};

mod common;

#[fixture]
fn pie() -> Pie<CommonTask> { common::create_pie() }

#[fixture]
fn temp_dir() -> TempDir { common::temp_dir() }


#[rstest]
fn test_serde_roundtrip_one_task(mut pie: Pie<CommonTask>, temp_dir: TempDir) {
  let path = temp_dir.path().join("test.txt");
  fs::write(&path, "HELLO WORLD!").check();

  let task = CommonTask::combine_a(&path, FileStamper::Modified);

  pie.run_in_session(|mut session| {
    session.require(&task);

    let tracker = &mut session.tracker_mut().0;
    tracker.clear();
  });

  let mut buffer = Vec::new();
  let mut serializer = Serializer::new(&mut buffer, Some(PrettyConfig::default()))
    .unwrap_or_else(|e| panic!("Creating serializer failed: {:?}", e));
  pie.serialize(&mut serializer)
    .unwrap_or_else(|e| panic!("Serialization failed: {:?}", e));
  println!("{}", String::from_utf8(buffer.clone()).expect("Ron should be utf-8"));

  let mut deserializer = Deserializer::from_bytes(&buffer)
    .unwrap_or_else(|e| panic!("Creating deserializer failed: {:?}", e));
  let mut pie = pie.deserialize(&mut deserializer)
    .unwrap_or_else(|e| panic!("Deserialization failed: {:?}", e));

  pie.run_in_session(|mut session| {
    session.require(&task);

    let tracker = &mut session.tracker_mut().0;
    assert!(tracker.contains_no_execute_start());
    tracker.clear();
  });
}

#[test]
fn test_non_hidden_dependency() -> Result<(), io::Error> {
  let mut pie = test_pie();
  let temp_dir = create_temp_dir()?;

  let file = temp_dir.path().join("in_out.txt");
  write(&file, "Hello, World!")?;

  let input_file = temp_dir.path().join("in.txt");
  write(&input_file, "Hi There!")?;
  let read_input = ReadFile(input_file.clone(), FileStamper::Modified, None);
  let write = WriteFile(Box::new(read_input.clone()), file.clone(), FileStamper::Modified);
  let read = ReadFile(file.clone(), FileStamper::Modified, Some(Box::new(write.clone())));

  // Require `read`, which requires `write` to update the provided file. All tasks are executed because they are new.
  let output = pie.require_then_assert(&read, |tracker| {
    assert!(tracker.one_execute_of(&read));
    assert!(tracker.one_execute_of(&write));
    assert!(tracker.one_execute_of(&read_input));
  })?;
  // `read` should output what `write` wrote, which is what `read_input` read from `input_file`.
  assert_eq!(output.as_str(), "Hi There!");

  // Remove `file` and confirm the provided file is re-generated.
  std::fs::remove_file(&file)?;
  assert!(!file.exists());
  let output = pie.require_then_assert(&read, |tracker| {
    // `write` should execute to re-generate the provided file.
    assert!(tracker.one_execute_of(&write));
    // `read_input` is not executed because its file dependency to `input_file` is consistent.
    assert!(!tracker.any_execute_of(&read_input));
    // `read` is executed because its `file` dependency is inconsistent, due to it having a new modified date. If we use
    // a file hash stamper, we can prevent this re-execution.
    assert!(tracker.one_execute_of(&read));
  })?;
  assert!(file.exists());
  assert_eq!(output.as_str(), "Hi There!");

  Ok(())
}

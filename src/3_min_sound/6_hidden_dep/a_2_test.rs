

// Hidden dependency tests

#[test]
fn test_hidden_dependency() -> Result<(), io::Error> {
  let mut pie = test_pie();
  let temp_dir = create_temp_dir()?;

  let file = temp_dir.path().join("in_out.txt");
  write(&file, "Hello, World!")?;

  let read = ReadFile(file.clone(), FileStamper::Modified);

  let input_file = temp_dir.path().join("in.txt");
  write(&input_file, "Hi there")?;
  let read_for_write = ReadFile(input_file.clone(), FileStamper::Modified);
  let write = WriteFile(Box::new(read_for_write.clone()), file.clone(), FileStamper::Modified);

  // Require `write` and `read`, assert they are executed because they are new.
  pie.require_then_assert_one_execute(&write)?;
  assert_eq!(read_to_string(&file)?, "Hi there");
  let output = pie.require_then_assert_one_execute(&read)?;
  assert_eq!(output.as_str(), "Hi there");

  // Although there is a hidden dependency here (`read` doesn't require `write`), we happened to have required `write`
  // and `read` in the correct order, so there is no inconsistency yet. The output of `read` is `"Hi there"` which is
  // correct.

  // Change `input_file` such that `read_for_write` becomes inconsistent, making `write` inconsistent.
  write_until_modified(&input_file, "Hello There!")?;

  // Require `read` and assert that it has not been executed, because all its dependencies are still consistent.
  let output = pie.require_then_assert_no_execute(&read)?;
  assert_eq!(output.as_str(), "Hi there");
  // Require `write` and assert that it is executed, because its dependency to `read_for_write` is inconsistent.
  pie.require_then_assert_one_execute(&write)?;
  assert_eq!(read_to_string(&file)?, "Hello There!");

  // This is incorrect, as `read` was deemed consistent with output `"Hi there"`, even though `write` was inconsistent
  // and needed to first write `"Hello There!"` to `file`. This inconsistency occurs because the dependency from `read`
  // to `write` was hidden by `file`! This inconsistent behaviour is undesirable.

  // Note: this is asserting the current behaviour, not the desired behaviour, which is to disallow this!
  // Note: we required the tasks in separate sessions (`require_then_assert_one_execute` starts a new session), but
  //       requiring these tasks in a single session will have the same outcome.

  Ok(())
}

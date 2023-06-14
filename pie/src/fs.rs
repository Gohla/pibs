use std::{fs, io};
use std::fs::{File, Metadata};
use std::path::Path;

/// Gets the metadata for given `path`, returning:
/// - `Ok(Some(metadata))` if a file or directory exists at given path,
/// - `Ok(None)` if no file or directory exists at given path,
/// - `Err(e)` if there was an error getting the metadata for given path.
pub fn metadata(path: impl AsRef<Path>) -> Result<Option<Metadata>, io::Error> {
  match fs::metadata(path) {
    Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
    Err(e) => Err(e),
    Ok(m) => Ok(Some(m))
  }
}

/// Attempt to open file at given `path`, returning:
/// - `Ok(Some(file))` if a file exists at given path, 
/// - `Ok(None)` if no file exists at given path (but a directory could exist at given path),
/// - `Err(e)` if there was an error getting the metadata for given path, or if there was an error opening the file.
///
/// This function is necessary due to Windows returning an error when attempting to open a directory.
pub fn open_if_file(path: impl AsRef<Path>) -> Result<Option<File>, io::Error> {
  let file = match metadata(&path)? {
    Some(metadata) if metadata.is_file() => Some(File::open(&path)?),
    _ => None,
  };
  Ok(file)
}


#[cfg(test)]
mod test {
  use tempfile::{NamedTempFile, TempDir};

  use super::*;

  #[test]
  fn test_metadata_ok() {
    let file = create_temp_file();
    let metadata = metadata(file.path());
    assert!(metadata.is_ok());
    let metadata = metadata.unwrap();
    assert!(metadata.is_some());
    let metadata = metadata.unwrap();
    assert!(metadata.is_file());
  }

  #[test]
  fn test_metadata_none() {
    let file = create_temp_file();
    let path = file.into_temp_path();
    std::fs::remove_file(&path).expect("failed to delete temporary file");
    let metadata = metadata(&path);
    assert!(metadata.is_ok());
    let metadata = metadata.unwrap();
    assert!(metadata.is_none());
  }

  #[test]
  fn test_open_if_file() {
    let path = create_temp_file().into_temp_path();
    let file = open_if_file(&path);
    assert!(file.is_ok());
    let file = file.unwrap();
    assert!(file.is_some());
  }

  #[test]
  fn test_open_if_file_non_existent() {
    let path = create_temp_file().into_temp_path();
    std::fs::remove_file(&path).expect("failed to delete temporary file");
    let file = open_if_file(&path);
    assert!(file.is_ok());
    let file = file.unwrap();
    assert!(file.is_none());
  }

  #[test]
  fn test_open_if_file_on_directory() {
    let dir = TempDir::new().expect("failed to create temporary directory");
    let file = open_if_file(dir.path());
    assert!(file.is_ok());
    let file = file.unwrap();
    assert!(file.is_none());
  }

  fn create_temp_file() -> NamedTempFile {
    NamedTempFile::new().expect("failed to create temporary file")
  }
}

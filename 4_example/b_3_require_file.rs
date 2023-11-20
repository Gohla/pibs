
fn require_file_to_string<C: Context<Tasks>>(context: &mut C, path: impl AsRef<Path>) -> Result<String, String> {
  let path = path.as_ref();
  let mut file = context.require_file(path)
    .map_err(|e| format!("Opening file '{}' for reading failed: {}", path.display(), e))?
    .ok_or_else(|| format!("File '{}' does not exist", path.display()))?;
  let mut text = String::new();
  file.read_to_string(&mut text)
    .map_err(|e| format!("Reading file '{}' failed: {}", path.display(), e))?;
  Ok(text)
}

use std::path::PathBuf;

use clap::Parser;

pub mod parse;
pub mod task;

#[derive(Parser)]
pub struct Args {
  /// Path to the pest grammar file.
  grammar_file_path: PathBuf,
  /// Rule name (from the pest grammar file) used to parse program files.
  rule_name: String,
  /// Paths to program files to parse with the pest grammar.
  program_file_paths: Vec<PathBuf>,
}

fn main() {
  let args = Args::parse();
}

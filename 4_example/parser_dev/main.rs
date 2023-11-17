use std::fmt::Write;
use std::io;
use std::path::PathBuf;

use clap::Parser;

use pie::Pie;
use pie::tracker::writing::WritingTracker;

use crate::editor::Editor;
use crate::task::{Outputs, Tasks};

pub mod parse;
pub mod task;
pub mod editor;

#[derive(Parser)]
struct Cli {
  #[command(flatten)]
  args: Args,
  /// Start a live parser development editor.
  #[arg(short, long)]
  edit: bool,
}

#[derive(clap::Args)]
pub struct Args {
  /// Path to the pest grammar file.
  grammar_file_path: PathBuf,
  /// Rule name (from the pest grammar file) used to parse program files.
  rule_name: String,
  /// Paths to program files to parse with the pest grammar.
  program_file_paths: Vec<PathBuf>
}

fn main() -> Result<(), io::Error> {
  let cli = Cli::parse();
  if cli.edit {
    let mut editor = Editor::new(cli.args)?;
    editor.run()
  } else {
    let mut pie = Pie::with_tracker(WritingTracker::with_stderr());
    let mut session = pie.new_session();

    let args = cli.args;
    let compile_grammar_task = Tasks::CompileGrammar(args.grammar_file_path.clone());
    let mut errors = String::new();
    if let Err(error) = session.require(&compile_grammar_task) {
      let _ = writeln!(errors, "{}", error);
    }

    let compile_grammar_task = Box::new(compile_grammar_task);
    for path in args.program_file_paths {
      match session.require(&Tasks::Parse(compile_grammar_task.clone(), path.clone(), args.rule_name.clone())) {
        Err(error) => { let _ = writeln!(errors, "{}", error); }
        Ok(Outputs::Parsed(Some(output))) => println!("Parsing '{}' succeeded: {}", path.display(), output),
        _ => {}
      }
    }

    if !errors.is_empty() {
      println!("Errors:\n{}", errors);
    }

    Ok(())
  }
}








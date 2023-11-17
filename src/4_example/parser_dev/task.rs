use std::io::Read;
use std::path::{Path, PathBuf};

use pie::{Context, Task};
use crate::parse::CompiledGrammar;

/// Tasks for compiling a pest grammar and parsing files with it.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum Tasks {
  CompileGrammar(PathBuf),
  Parse(Box<Tasks>, PathBuf, String)
}

/// Outputs for [`Tasks`].
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Outputs {
  CompiledGrammar(CompiledGrammar),
  Parsed(Option<String>)
}

impl Task for Tasks {
  type Output = Result<Outputs, String>;

  fn execute<C: Context<Self>>(&self, context: &mut C) -> Self::Output {
    match self {
      Tasks::CompileGrammar(grammar_file_path) => {
        let grammar_text = require_file_to_string(context, grammar_file_path)?;
        let compiled_grammar = CompiledGrammar::new(&grammar_text, grammar_file_path.to_string_lossy())?;
        Ok(Outputs::CompiledGrammar(compiled_grammar))
      }
      Tasks::Parse(compile_grammar_task, program_file_path, rule_name) => {
        let Ok(Outputs::CompiledGrammar(compiled_grammar)) = context.require_task(compile_grammar_task.as_ref()) else {
          // Return `None` if compiling grammar failed. Don't propagate the error, otherwise the error would be
          // duplicated for all `Parse` tasks.
          return Ok(Outputs::Parsed(None));
        };
        let program_text = require_file_to_string(context, program_file_path)?;
        let output = compiled_grammar.parse(&program_text, rule_name, program_file_path.to_string_lossy())?;
        Ok(Outputs::Parsed(Some(output)))
      }
    }
  }
}

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

use std::io::Read;
use std::path::{Path, PathBuf};

use pie::{Context, Task};

use crate::parse::CompiledGrammar;

/// Tasks for compiling a grammar and parsing files with it.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum Tasks {
  CompileGrammar { grammar_file_path: PathBuf },
  Parse { compiled_grammar_task: Box<Tasks>, program_file_path: PathBuf, rule_name: String }
}

impl Tasks {
  /// Create a [`Self::CompileGrammar`] task that compiles the grammar in file `grammar_file_path`.
  pub fn compile_grammar(grammar_file_path: impl Into<PathBuf>) -> Self {
    Self::CompileGrammar { grammar_file_path: grammar_file_path.into() }
  }

  /// Create a [`Self::Parse`] task that uses the compiled grammar returned by requiring `compiled_grammar_task` to
  /// parse the program in file `program_file_path`, starting parsing with `rule_name`.
  pub fn parse(
    compiled_grammar_task: &Tasks,
    program_file_path: impl Into<PathBuf>,
    rule_name: impl Into<String>
  ) -> Self {
    Self::Parse {
      compiled_grammar_task: Box::new(compiled_grammar_task.clone()),
      program_file_path: program_file_path.into(),
      rule_name: rule_name.into()
    }
  }
}

/// Outputs for [`Tasks`].
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Outputs {
  CompiledGrammar(CompiledGrammar),
  Parsed(Option<String>)
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

impl Task for Tasks {
  type Output = Result<Outputs, String>;

  fn execute<C: Context<Self>>(&self, context: &mut C) -> Self::Output {
    match self {
      Tasks::CompileGrammar { grammar_file_path } => {
        let grammar_text = require_file_to_string(context, grammar_file_path)?;
        let compiled_grammar = CompiledGrammar::new(&grammar_text, Some(grammar_file_path.to_string_lossy().as_ref()))?;
        Ok(Outputs::CompiledGrammar(compiled_grammar))
      }
      Tasks::Parse { compiled_grammar_task, program_file_path, rule_name } => {
        let Ok(Outputs::CompiledGrammar(compiled_grammar)) = context.require_task(compiled_grammar_task.as_ref()) else {
          // Return `None` if compiling grammar failed. Don't propagate the error, otherwise the error would be
          // duplicated for all `Parse` tasks.
          return Ok(Outputs::Parsed(None));
        };
        let program_text = require_file_to_string(context, program_file_path)?;
        let output = compiled_grammar.parse(&program_text, rule_name, Some(program_file_path.to_string_lossy().as_ref()))?;
        Ok(Outputs::Parsed(Some(output)))
      }
    }
  }
}

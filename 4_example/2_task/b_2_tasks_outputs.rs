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

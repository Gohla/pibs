
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

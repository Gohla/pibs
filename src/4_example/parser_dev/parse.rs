use std::collections::HashSet;
use std::fmt::Write;

/// Parse programs with a compiled pest grammar.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct CompiledGrammar {
  rules: Vec<pest_meta::optimizer::OptimizedRule>,
  rule_names: HashSet<String>,
}

impl CompiledGrammar {
  /// Compile the pest grammar from `grammar_text`, using `path` to annotate errors. Returns a [`Self`] instance.
  ///
  /// # Errors
  ///
  /// Returns `Err(error_string)` when compiling the grammar fails.
  pub fn new(grammar_text: &str, path: Option<&str>) -> Result<Self, String> {
    match pest_meta::parse_and_optimize(grammar_text) {
      Ok((builtin_rules, rules)) => {
        let mut rule_names = HashSet::with_capacity(builtin_rules.len() + rules.len());
        rule_names.extend(builtin_rules.iter().map(|s| s.to_string()));
        rule_names.extend(rules.iter().map(|s| s.name.clone()));
        Ok(Self { rules, rule_names })
      },
      Err(errors) => {
        let mut error_string = String::new();
        for mut error in errors {
          if let Some(path) = path.as_ref() {
            error = error.with_path(path);
          }
          error = error.renamed_rules(pest_meta::parser::rename_meta_rule);
          let _ = writeln!(error_string, "{}", error); // Ignore error: writing to String cannot fail.
        }
        Err(error_string)
      }
    }
  }

  /// Parse `program_text` with rule `rule_name` using this compiled grammar, using `path` to annotate errors. Returns
  /// parsed pairs formatted as a string.
  ///
  /// # Errors
  ///
  /// Returns `Err(error_string)` when parsing fails.
  pub fn parse(&self, program_text: &str, rule_name: &str, path: Option<&str>) -> Result<String, String> {
    if !self.rule_names.contains(rule_name) {
      let message = format!("rule '{}' was not found", rule_name);
      return Err(message);
    }
    // Note: can't store `Vm` in `CompiledGrammar` because `Vm` is not `Clone` nor `Eq`.
    let vm = pest_vm::Vm::new(self.rules.clone());
    match vm.parse(rule_name, program_text) {
      Ok(pairs) => Ok(format!("{}", pairs)),
      Err(mut error) => {
        if let Some(path) = path {
          error = error.with_path(path);
        }
        error = error.renamed_rules(|r| r.to_string());
        let error_string = format!("{}", error);
        Err(error_string)
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_compile_parse() -> Result<(), String> {
    // Grammar compilation failure.
    let result = CompiledGrammar::new("asd = { fgh } qwe = { rty }", None);
    assert!(result.is_err());
    println!("{}", result.unwrap_err());

    // Grammar that parses numbers.
    let compiled_grammar = CompiledGrammar::new("num = { ASCII_DIGIT+ }", None)?;
    println!("{:?}", compiled_grammar);

    // Parse failure
    let result = compiled_grammar.parse("a", "num", None);
    assert!(result.is_err());
    println!("{}", result.unwrap_err());
    // Parse failure due to non-existent rule.
    let result = compiled_grammar.parse("1", "asd", None);
    assert!(result.is_err());
    println!("{}", result.unwrap_err());
    // Parse success
    let result = compiled_grammar.parse("1", "num", None);
    assert!(result.is_ok());
    println!("{}", result.unwrap());

    Ok(())
  }
}

use std::collections::HashSet;
use std::fmt::Write;

/// Parse programs with a compiled pest grammar.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct CompiledGrammar {
  rules: Vec<pest_meta::optimizer::OptimizedRule>,
  rule_names: HashSet<String>,
}

impl CompiledGrammar {
  /// Compile the pest grammar from `grammar_text`, using `path` to annotate errors. Returns a [`CompiledGrammar`]
  /// instance.
  ///
  /// # Errors
  ///
  /// Returns `Err(error_string)` when compiling the grammar fails.
  pub fn new(grammar_text: &str, path: impl AsRef<str>) -> Result<CompiledGrammar, String> {
    match pest_meta::parse_and_optimize(grammar_text) {
      Ok((builtin_rules, rules)) => {
        let mut rule_names = HashSet::with_capacity(builtin_rules.len() + rules.len());
        rule_names.extend(builtin_rules.iter().map(|s| s.to_string()));
        rule_names.extend(rules.iter().map(|s| s.name.clone()));
        Ok(Self { rules, rule_names })
      },
      Err(errors) => {
        let mut error_string = String::new();
        for error in errors {
          let error = error
            .with_path(path.as_ref())
            .renamed_rules(pest_meta::parser::rename_meta_rule);
          let _ = writeln!(error_string, "{}", error);
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
  pub fn parse(&self, program_text: &str, rule_name: &str, path: impl AsRef<str>) -> Result<String, String> {
    if !self.rule_names.contains(rule_name) {
      let message = format!("rule '{}' was not found", rule_name);
      return Err(message);
    }
    // Note: can't store `Vm` in `CompiledGrammar` because `Vm` is not `Clone` nor `Eq`.
    let vm = pest_vm::Vm::new(self.rules.clone());
    match vm.parse(rule_name, program_text) {
      Ok(pairs) => Ok(format!("{}", pairs)),
      Err(error) => {
        let error = error
          .with_path(path.as_ref())
          .renamed_rules(|r| r.to_string());
        let error_string = format!("{}", error);
        Err(error_string)
      }
    }
  }
}

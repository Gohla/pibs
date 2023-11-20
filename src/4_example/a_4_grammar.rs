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
}

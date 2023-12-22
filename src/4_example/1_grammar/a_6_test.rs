
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

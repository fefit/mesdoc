use gotdom::parser::rule::*;
fn main() {
  let rule: Rule = ":nth-child({spaces}{index}{spaces(0)})".into();
  print!("rule {:?}", rule);
  let rule: Rule = "[{{spaces}{attr_key}{regexp!#abc#}]".into();
  print!("rule {:?}", rule);
}

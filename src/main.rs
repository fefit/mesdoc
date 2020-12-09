use gotdom::selector::rule::{self, Rule};
fn main() {
  rule::init();
  let mut rule: Rule = ":nth-child({spaces}{index}{spaces})".into();
  rule.exec(":nth-child(1)");
  print!("rule {:?}", rule);

  // let rule: Rule = "[{{spaces}}{attr_key}{regexp#abc#}]".into();
  // print!("rule {:?}", rule);
}

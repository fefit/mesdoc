use gotdom::selector::rule::{self, Rule};
fn main() {
  rule::init();
  let mut rule: Rule = "{identity?}:nth-child({spaces}{nth}{spaces})".into();
  rule.exec("div:nth-child(-2n+1)");

  // let rule: Rule = "[{{spaces}}{attr_key}{regexp#abc#}]".into();
  // print!("rule {:?}", rule);
}

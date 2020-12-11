use gotdom::selector::rule::{self, Rule};
fn main() {
  rule::init();
  let mut rule: Rule = "{identity?}:nth-child({spaces}{nth}{spaces})".into();

  // let rule: Rule = "[{{spaces}}{attr_key}{regexp#abc#}]".into();
  // print!("rule {:?}", rule);
}

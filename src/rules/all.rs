use crate::selector::rule::{RuleDefItem, RuleItem};
pub fn init(rules: &mut Vec<RuleItem>) {
  let rule = RuleDefItem("*", 0, vec![], Box::new(|nodes, _params| Ok(nodes)));
  rules.push(rule.into());
}

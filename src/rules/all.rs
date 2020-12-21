use crate::selector::rule::{RuleDefItem, RuleItem};
pub fn init(rules: &mut Vec<RuleItem>) {
  let rule: RuleItem = RuleDefItem(
    "*",
    0,
    vec![],
    Box::new(|nodes, _params| Ok(nodes.cloned())),
  )
  .into();
  rules.push(rule);
}

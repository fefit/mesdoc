use crate::selector::rule::{Rule, RuleItem};
fn add_empty(rules: &mut Vec<RuleItem>) {
  // empty
  let rule: RuleItem = (":empty", vec![], Box::new(|nodes, params, count| Ok(nodes)));
  rules.push(rule);
}
fn add_first_child(rules: &mut Vec<RuleItem>) {
  // empty
  let rule: RuleItem = (
    ":first-child",
    vec![],
    Box::new(|nodes, params, count| Ok(nodes)),
  );
  rules.push(rule);
}
fn add_last_child(rules: &mut Vec<RuleItem>) {
  // empty
  let rule: RuleItem = (
    ":last-child",
    vec![],
    Box::new(|nodes, params, count| Ok(nodes)),
  );
  rules.push(rule);
}
fn add_first_of_type(rules: &mut Vec<RuleItem>) {
  // empty
  let rule: RuleItem = (
    ":first-of-type",
    vec![],
    Box::new(|nodes, params, count| Ok(nodes)),
  );
  rules.push(rule);
}
pub fn init(rules: &mut Vec<RuleItem>) {
  add_empty(rules);
  add_first_child(rules);
  add_last_child(rules);
  add_first_of_type(rules);
}

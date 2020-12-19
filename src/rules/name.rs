use crate::selector::interface::NodeList;
use crate::selector::rule::{Rule, RuleDefItem, RuleItem};
pub fn init(rules: &mut Vec<RuleItem>) {
  let rule = RuleDefItem(
    "{identity}",
    100,
    vec![("identity", 0)],
    Box::new(|nodes, params| {
      let name = Rule::param(&params, "identity").expect("The 'id' selector is not correct");
      let mut result: NodeList = NodeList::new();
      for node in nodes {
        if node.tag_name() == name {
          result.push(node.cloned());
        }
      }
      Ok(result)
    }),
  );
  rules.push(rule.into());
}

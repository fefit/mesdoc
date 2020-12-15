use crate::selector::interface::NodeList;
use crate::selector::rule::{Rule, RuleItem};
pub fn init(rules: &mut Vec<RuleItem>) {
  let rule: RuleItem = (
    "{identity}",
    vec![("identity", 0)],
    Box::new(|nodes, params, _count| {
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
  rules.push(rule);
}

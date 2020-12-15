use crate::selector::interface::{AttrValue, NodeList};
use crate::selector::rule::{Rule, RuleItem};
pub fn init(rules: &mut Vec<RuleItem>) {
  let rule: RuleItem = (
    "#{identity}",
    vec![("identity", 0)],
    Box::new(|nodes, params, _count| {
      let id = Rule::param(&params, "identity").expect("The 'id' selector is not correct");
      let mut result: NodeList = NodeList::new();
      for node in nodes {
        if let Some(AttrValue::Value(id_name)) = node.get_attribute("id") {
          if id_name == id {
            result.push(node.cloned());
            break;
          }
        }
      }
      Ok(result)
    }),
  );
  rules.push(rule);
}

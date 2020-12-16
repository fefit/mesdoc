use crate::selector::interface::{AttrValue, NodeList};
use crate::selector::rule::{Rule, RuleItem};
pub fn init(rules: &mut Vec<RuleItem>) {
  let rule: RuleItem = (
    ".{identity}",
    vec![("identity", 0)],
    Box::new(|nodes, params| {
      let class_name =
        Rule::param(&params, "identity").expect("The 'class' selector is not correct");
      let mut result: NodeList = NodeList::new();
      for node in nodes {
        if let Some(AttrValue::Value(classes)) = node.get_attribute("id") {
          let class_list = classes.split_ascii_whitespace();
          for cls in class_list {
            if cls == class_name {
              result.push(node.cloned());
              break;
            }
          }
        }
      }
      Ok(result)
    }),
  );
  rules.push(rule);
}

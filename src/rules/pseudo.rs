use crate::selector::interface::{NodeList, NodeType};
use crate::selector::rule::{Rule, RuleItem};
const PRIORITY: u32 = 10;
fn add_empty(rules: &mut Vec<RuleItem>) {
  // empty
  let rule: RuleItem = (
    ":empty",
    PRIORITY,
    vec![],
    Box::new(|nodes, _params| {
      let mut result = NodeList::new();
      for node in nodes {
        if node.node_type().is_element() {
          if let Ok(childs) = node.children() {
            let mut only_comments = true;
            for child in childs {
              match child.node_type() {
                NodeType::Comment => continue,
                _ => {
                  only_comments = false;
                  break;
                }
              }
            }
            if only_comments {
              result.push(node.cloned());
            }
          } else {
            result.push(node.cloned());
          }
        }
      }
      Ok(result)
    }),
  );
  rules.push(rule);
}
fn add_first_child(rules: &mut Vec<RuleItem>) {
  // first-child
  let rule: RuleItem = (
    ":first-child",
    PRIORITY,
    vec![],
    Box::new(|nodes, _params| {
      let mut result = NodeList::new();
      for node in nodes {
        if node.parent().is_ok() {
          if node.node_type().is_element() && node.index().unwrap() == 0 {
            result.push(node.cloned());
          }
        } else {
          result.push(node.cloned());
        }
      }
      Ok(result)
    }),
  );
  rules.push(rule);
}
fn add_last_child(rules: &mut Vec<RuleItem>) {
  // last_child
  let rule: RuleItem = (
    ":last-child",
    PRIORITY,
    vec![],
    Box::new(|nodes, _params| {
      let mut result = NodeList::new();
      for node in nodes {
        if let Ok(pnode) = node.parent() {
          if node.node_type().is_element() {
            let childs = pnode.children().unwrap();
            let mut total = childs.count();
            while total > 0 {
              total -= 1;
              let cur_node = childs.get(total).unwrap();
              if cur_node.node_type().is_element() {
                if node.is(cur_node) {
                  result.push(node.cloned());
                }
                break;
              }
            }
          }
        } else {
          result.push(node.cloned());
        }
      }
      Ok(result)
    }),
  );
  rules.push(rule);
}
fn add_first_of_type(rules: &mut Vec<RuleItem>) {
  // first of type
  let rule: RuleItem = (
    ":first-of-type",
    PRIORITY,
    vec![],
    Box::new(|nodes, params| Ok(nodes)),
  );
  rules.push(rule);
}
pub fn init(rules: &mut Vec<RuleItem>) {
  add_empty(rules);
  add_first_child(rules);
  add_last_child(rules);
  add_first_of_type(rules);
}

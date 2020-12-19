#![allow(clippy::or_fun_call)]
use crate::selector::interface::{AttrValue, NodeList};
use crate::selector::rule::{Rule, RuleDefItem, RuleItem};
pub fn init(rules: &mut Vec<RuleItem>) {
  let rule = RuleDefItem(
    r##"[{spaces}{attr_key}{spaces}{regexp#(?:([~|^$*]?)=\s*(?:"((?:\\?+.)*?)"|'((?:\\?+.)*?)'|([^\s'"<>/=`]+)))?#}{spaces}]"##,
    10,
    vec![("attr_key", 0), ("regexp", 0)],
    Box::new(|nodes, params| {
      let attr_key =
        Rule::param(&params, "attr_key").expect("The attribute selector's key is not correct");
      let attr_value = Rule::param(&params, ("regexp", 0, "2"))
        .or(Rule::param(&params, ("regexp", 0, "3")))
        .or(Rule::param(&params, ("regexp", 0, "4")));
      let handle: Box<dyn Fn(Option<AttrValue>) -> bool> = if let Some(attr_value) = attr_value {
        let mode = Rule::param(&params, ("regexp", 0, "1")).unwrap_or("");
        match mode {
          "^" => Box::new(move |val: Option<AttrValue>| match val {
            Some(AttrValue::Value(v)) => v.starts_with(attr_value),
            _ => false,
          }),
          "$" => Box::new(move |val: Option<AttrValue>| match val {
            Some(AttrValue::Value(v)) => v.ends_with(attr_value),
            _ => false,
          }),
          "*" => Box::new(move |val: Option<AttrValue>| match val {
            Some(AttrValue::Value(v)) => v.contains(attr_value),
            _ => false,
          }),
          "|" => Box::new(move |val: Option<AttrValue>| match val {
            Some(AttrValue::Value(v)) => {
              if v.contains(attr_value) {
                return true;
              }
              let attr_value: String = String::from(attr_value) + "-";
              v.contains(&attr_value)
            }
            _ => false,
          }),
          "~" => Box::new(move |val: Option<AttrValue>| match val {
            Some(AttrValue::Value(v)) => {
              let split_v = v.split_ascii_whitespace();
              for v in split_v {
                if v == attr_value {
                  return true;
                }
              }
              false
            }
            _ => false,
          }),
          _ => Box::new(move |val: Option<AttrValue>| match val {
            Some(AttrValue::Value(v)) => v == attr_value,
            _ => false,
          }),
        }
      } else {
        Box::new(|val: Option<AttrValue>| val.is_some())
      };
      let mut result: NodeList = NodeList::new();
      for node in nodes {
        let cur_value = node.get_attribute(attr_key);
        if handle(cur_value) {
          result.push(node.cloned());
        }
      }
      Ok(result)
    }),
  );
  rules.push(rule.into());
}

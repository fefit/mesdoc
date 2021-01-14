#![allow(clippy::or_fun_call)]
use crate::selector::rule::{Rule, RuleDefItem, RuleItem};
use crate::selector::{
	interface::{IAttrValue, NodeList, Result},
	rule::RuleMatchedData,
};
pub fn init(rules: &mut Vec<RuleItem>) {
	let rule = RuleDefItem(
		r##"[{spaces}{attr_key}{spaces}{regexp#(?:([~|^$*!]?)=\s*(?:"((?:\\?+.)*?)"|'((?:\\?+.)*?)'|([^\s'"<>/=`]+)))?#}{spaces}]"##,
		10,
		vec![("attr_key", 0), ("regexp", 0)],
		Box::new(|nodes: &NodeList, params: &RuleMatchedData| -> Result {
			let attr_key =
				Rule::param(&params, "attr_key").expect("The attribute selector's key is not correct");
			let attr_value = Rule::param(&params, ("regexp", 0, "2"))
				.or(Rule::param(&params, ("regexp", 0, "3")))
				.or(Rule::param(&params, ("regexp", 0, "4")));
			let handle: Box<dyn Fn(Option<IAttrValue>) -> bool> = if let Some(attr_value) = attr_value {
				let mode = Rule::param(&params, ("regexp", 0, "1")).unwrap_or("");
				match mode {
					"^" => Box::new(move |val: Option<IAttrValue>| match val {
						Some(IAttrValue::Value(v, _)) => v.starts_with(attr_value),
						_ => false,
					}),
					"$" => Box::new(move |val: Option<IAttrValue>| match val {
						Some(IAttrValue::Value(v, _)) => v.ends_with(attr_value),
						_ => false,
					}),
					"*" => Box::new(move |val: Option<IAttrValue>| match val {
						Some(IAttrValue::Value(v, _)) => v.contains(attr_value),
						_ => false,
					}),
					"|" => Box::new(move |val: Option<IAttrValue>| match val {
						Some(IAttrValue::Value(v, _)) => {
							if v.contains(attr_value) {
								return true;
							}
							let attr_value: String = format!("{}-", attr_value);
							v.contains(&attr_value)
						}
						_ => false,
					}),
					"~" => Box::new(move |val: Option<IAttrValue>| match val {
						Some(IAttrValue::Value(v, _)) => {
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
					"!" => Box::new(move |val: Option<IAttrValue>| match val {
						Some(IAttrValue::Value(v, _)) => attr_value != v,
						_ => false,
					}),
					_ => Box::new(move |val: Option<IAttrValue>| match val {
						Some(IAttrValue::Value(v, _)) => v == attr_value,
						_ => false,
					}),
				}
			} else {
				Box::new(|val: Option<IAttrValue>| val.is_some())
			};
			let mut result: NodeList = NodeList::new();
			for node in nodes.get_ref() {
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

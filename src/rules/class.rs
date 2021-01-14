use crate::selector::rule::{Rule, RuleDefItem, RuleItem};
use crate::selector::{
	interface::{IAttrValue, NodeList, Result},
	rule::RuleMatchedData,
};
pub fn init(rules: &mut Vec<RuleItem>) {
	let rule = RuleDefItem(
		".{identity}",
		1000,
		vec![("identity", 0)],
		Box::new(|nodes: &NodeList, params: &RuleMatchedData| -> Result {
			let class_name =
				Rule::param(&params, "identity").expect("The 'class' selector is not correct");
			let mut result: NodeList = NodeList::new();
			for node in nodes.get_ref() {
				if let Some(IAttrValue::Value(class_list, _)) = node.get_attribute("class") {
					let class_list = class_list.split_ascii_whitespace();
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
	rules.push(rule.into());
}

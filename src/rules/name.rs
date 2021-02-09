use crate::interface::Elements;
use crate::selector::rule::RuleMatchedData;
use crate::selector::rule::{Rule, RuleDefItem, RuleItem};
pub fn init(rules: &mut Vec<RuleItem>) {
	let rule = RuleDefItem(
		"name",
		"{identity}",
		100,
		vec![("identity", 0)],
		Box::new(|nodes: &Elements, params: &RuleMatchedData| -> Elements {
			let name =
				Rule::param(&params, "identity").expect("The 'name' selector must have a tag name");
			let mut result = Elements::new();
			for node in nodes.get_ref() {
				if node.tag_name() == name {
					result.push(node.cloned());
				}
			}
			result
		}),
	);
	rules.push(rule.into());
}

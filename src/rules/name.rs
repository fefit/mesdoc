use crate::interface::Elements;
use crate::selector::rule::RuleMatchedData;
use crate::selector::rule::{Rule, RuleDefItem, RuleItem};
pub fn init(rules: &mut Vec<RuleItem>) {
	let rule = RuleDefItem(
		"name",
		"{identity}",
		100,
		vec![("identity", 0)],
		Box::new(|eles: &Elements, params: &RuleMatchedData| -> Elements {
			let name = Rule::param(&params, "identity")
				.expect("The 'name' selector must have a tag name")
				.to_ascii_uppercase();
			let mut result = Elements::with_capacity(eles.length() / 2);
			for node in eles.get_ref() {
				if node.tag_name() == name {
					result.push(node.cloned());
				}
			}
			result
		}),
	);
	rules.push(rule.into());
}

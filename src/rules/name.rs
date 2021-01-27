use crate::interface::NodeList;
use crate::selector::rule::RuleMatchedData;
use crate::selector::rule::{Rule, RuleDefItem, RuleItem};
pub fn init(rules: &mut Vec<RuleItem>) {
	let rule = RuleDefItem(
		"name",
		"{identity}",
		100,
		vec![("identity", 0)],
		Box::new(|nodes: &NodeList, params: &RuleMatchedData| -> NodeList {
			let name = Rule::param(&params, "identity").expect("The 'id' selector is not correct");
			let mut result = NodeList::new();
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

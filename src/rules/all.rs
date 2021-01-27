use crate::interface::NodeList;
use crate::selector::rule::{RuleDefItem, RuleItem};
/// selector: `*`
pub fn init(rules: &mut Vec<RuleItem>) {
	let rule: RuleItem = RuleDefItem(
		"all",
		"*",
		0,
		vec![],
		Box::new(|nodes: &NodeList, _| -> NodeList { nodes.cloned() }),
	)
	.into();
	rules.push(rule);
}

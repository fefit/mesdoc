use crate::interface::Elements;
use crate::selector::rule::{RuleDefItem, RuleItem};
/// selector: `*`
pub fn init(rules: &mut Vec<RuleItem>) {
	let rule: RuleItem = RuleDefItem(
		"all",
		"*",
		0,
		vec![],
		Box::new(|nodes: &Elements, _| -> Elements { nodes.cloned() }),
	)
	.into();
	rules.push(rule);
}

use crate::selector::{
	interface::{NodeList, Result},
	rule::{RuleDefItem, RuleItem},
};
/// selector: `*`
pub fn init(rules: &mut Vec<RuleItem>) {
	let rule: RuleItem = RuleDefItem(
		"*",
		0,
		vec![],
		Box::new(|nodes: &NodeList, _| -> Result { Ok(nodes.cloned()) }),
	)
	.into();
	rules.push(rule);
}

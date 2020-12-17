use crate::selector::rule::RuleItem;
pub fn init(rules: &mut Vec<RuleItem>) {
	let rule: RuleItem = ("*", vec![], Box::new(|nodes, _params| Ok(nodes)));
	rules.push(rule);
}

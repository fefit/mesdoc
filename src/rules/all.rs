use crate::selector::rule::RuleItem;
pub fn init(rules: &mut Vec<RuleItem>) {
	let rule: RuleItem = ("*", 0, vec![], Box::new(|nodes, _params| Ok(nodes)));
	rules.push(rule);
}

use crate::selector::interface::NodeList;
use crate::selector::rule::{Rule, RuleItem};
pub fn init(rules: &mut Vec<RuleItem>) {
	let rule: RuleItem = (
		"#{identity}",
		10000,
		true,
		vec![("identity", 0)],
		Box::new(|nodes, params| {
			let id = Rule::param(&params, "identity").expect("The 'id' selector is not correct");
			let mut result: NodeList = NodeList::with_capacity(1);
			if nodes.length() > 0 {
				let first_node = nodes
					.get_ref()
					.get(0)
					.expect("The first node must exists because the length > 0");
				if let Ok(Some(root)) = first_node.owner_document() {
					if let Some(id_element) = root.get_element_by_id(id) {
						result.push(id_element.cloned());
					}
				}
			}
			Ok(result)
		}),
	);
	rules.push(rule);
}

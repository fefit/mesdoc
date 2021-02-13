use crate::interface::Elements;
use crate::selector::rule::RuleMatchedData;
use crate::selector::rule::{Rule, RuleItem};
pub fn init(rules: &mut Vec<RuleItem>) {
	let rule: RuleItem = RuleItem {
		name: "id",
		context: "#{identity}",
		rule: Rule {
			priority: 10000,
			in_cache: true,
			fields: vec![("identity", 0)],
			handle: Some(Box::new(
				|eles: &Elements, params: &RuleMatchedData| -> Elements {
					let id = Rule::param(&params, "identity").expect("The 'id' selector is not correct");
					let mut result = Elements::with_capacity(1);
					if eles.length() > 0 {
						let first_node = eles
							.get_ref()
							.get(0)
							.expect("The first node must exists because the length > 0");
						if let Some(root) = first_node.owner_document() {
							if let Some(id_element) = &root.get_element_by_id(id) {
								for ele in eles.get_ref() {
									if ele.is(id_element) {
										result.push(ele.cloned());
										break;
									}
								}
							}
						}
					}
					result
				},
			)),
			..Default::default()
		},
	};
	rules.push(rule);
}

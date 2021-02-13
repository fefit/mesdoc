use crate::selector::rule::RuleMatchedData;
use crate::selector::rule::{Rule, RuleItem};
use crate::{interface::Elements, selector::rule::RuleOptions};
pub fn init(rules: &mut Vec<RuleItem>) {
	let rule: RuleItem = RuleItem {
		name: "id",
		context: "#{identity}",
		rule: Rule {
			priority: 10000,
			in_cache: true,
			fields: vec![("identity", 0)],
			handle: Some(Box::new(
				|eles: &Elements, params: &RuleMatchedData, options: &RuleOptions| -> Elements {
					let id = Rule::param(&params, "identity").expect("The 'id' selector is not correct");
					let mut result = Elements::with_capacity(1);
					if eles.length() > 0 {
						let is_use_cache = !options.no_cache;
						if is_use_cache {
							let first_node = eles
								.get_ref()
								.get(0)
								.expect("The first node must exists because the length > 0");
							if let Some(root) = first_node.owner_document() {
								if let Some(id_element) = root.get_element_by_id(id) {
									result.push(id_element.cloned());
								}
							}
						} else {
							for ele in eles.get_ref() {
								if let Some(value) = &ele.get_attribute("id") {
									if value.is_str(id) {
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

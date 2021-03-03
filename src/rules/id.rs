use crate::selector::rule::{Matcher, MatcherData, MatcherHandle, Rule, RuleItem};
use crate::{constants::USE_CACHE_DATAKEY, interface::Elements};
pub fn init(rules: &mut Vec<RuleItem>) {
	let rule: RuleItem = RuleItem {
		name: "id",
		context: "#{identity}",
		rule: Rule {
			priority: 10000,
			in_cache: true,
			fields: vec![("identity", 0), USE_CACHE_DATAKEY],
			handle: Some(Box::new(|data: MatcherData| Matcher {
				handle: MatcherHandle::All(Box::new(|eles: &Elements| {
					let id = Rule::param(&data, "identity").expect("The 'id' selector is not correct");
					let use_cache = Rule::is_use_cache(&data);
					let mut result = Elements::with_capacity(1);
					if !eles.is_empty() {
						let first_ele = eles
							.get_ref()
							.get(0)
							.expect("The elements must have at least one element.");
						if let Some(doc) = &first_ele.owner_document() {
							if let Some(id_element) = &doc.get_element_by_id(id) {
								if use_cache {
									// just add, will checked if the element contains the id element
									result.push(id_element.cloned());
								} else {
									// filter methods, will filtered in elements
									for ele in eles.get_ref() {
										if ele.is(id_element) {
											result.push(ele.cloned());
											break;
										}
									}
								}
							}
						}
					}
					result
				})),
				data,
			})),
			..Default::default()
		},
	};
	rules.push(rule);
}

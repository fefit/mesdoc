use crate::interface::{Elements, IAttrValue};
use crate::selector::rule::RuleMatchedData;
use crate::selector::rule::{Rule, RuleDefItem, RuleItem};
use crate::utils::get_class_list;
pub fn init(rules: &mut Vec<RuleItem>) {
	let rule = RuleDefItem(
		"class",
		".{identity}",
		1000,
		vec![("identity", 0)],
		Box::new(|eles: &Elements, params: &RuleMatchedData| -> Elements {
			let class_name =
				Rule::param(&params, "identity").expect("The 'class' selector is not correct");
			let mut result = Elements::with_capacity(5);
			for node in eles.get_ref() {
				if let Some(IAttrValue::Value(names, _)) = node.get_attribute("class") {
					let class_list = get_class_list(&names);
					if class_list.contains(&class_name) {
						result.push(node.cloned());
					}
				}
			}
			result
		}),
	);
	rules.push(rule.into());
}

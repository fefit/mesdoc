use crate::interface::{BoxDynElement, IAttrValue};
use crate::selector::rule::{Matcher, MatcherData, MatcherHandle};
use crate::selector::rule::{Rule, RuleDefItem, RuleItem};
use crate::utils::get_class_list;
pub fn init(rules: &mut Vec<RuleItem>) {
	let rule = RuleDefItem(
		"class",
		".{identity}",
		1000,
		vec![("identity", 0)],
		Box::new(|data: MatcherData| Matcher {
			handle: MatcherHandle::One(Box::new(|ele: &BoxDynElement| -> bool {
				let class_name =
					Rule::param(&data, "identity").expect("The 'class' selector is not correct");
				if let Some(IAttrValue::Value(names, _)) = ele.get_attribute("class") {
					let class_list = get_class_list(&names);
					return class_list.contains(&class_name);
				}
				false
			})),
			data,
		}),
	);
	rules.push(rule.into());
}

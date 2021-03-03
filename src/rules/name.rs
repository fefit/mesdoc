use crate::interface::BoxDynElement;
use crate::selector::rule::{Matcher, MatcherData, MatcherHandle, Rule, RuleDefItem, RuleItem};
pub fn init(rules: &mut Vec<RuleItem>) {
	let rule = RuleDefItem(
		"name",
		"{identity}",
		100,
		vec![("identity", 0)],
		Box::new(|data: MatcherData| Matcher {
			handle: MatcherHandle::One(Box::new(|ele: &BoxDynElement| {
				let name = Rule::param(&data, "identity")
					.expect("The 'name' selector must have a tag name")
					.to_ascii_uppercase();
				return ele.tag_name() == name;
			})),
			data,
		}),
	);
	rules.push(rule.into());
}

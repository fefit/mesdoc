use crate::interface::BoxDynElement;
use crate::selector::rule::{Matcher, MatcherData, Rule, RuleDefItem, RuleItem};
pub fn init(rules: &mut Vec<RuleItem>) {
	let rule = RuleDefItem(
		"name",
		"{identity}",
		100,
		vec![("identity", 0)],
		Box::new(|data: MatcherData| {
			let name = Rule::param(&data, "identity")
				.expect("The 'name' selector must have a tag name")
				.to_ascii_uppercase();
			Matcher {
				one_handle: Some(Box::new(move |ele: &BoxDynElement, _| {
					return ele.tag_name() == name;
				})),
				..Default::default()
			}
		}),
	);
	rules.push(rule.into());
}

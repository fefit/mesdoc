use std::collections::HashMap;

use crate::interface::Elements;
use crate::selector::rule::{Matcher, MatcherHandle, RuleDefItem, RuleItem};
/// selector: `*`
pub fn init(rules: &mut Vec<RuleItem>) {
	let rule: RuleItem = RuleDefItem(
		"all",
		"*",
		0,
		vec![],
		Box::new(|_| Matcher {
			handle: MatcherHandle::All(Box::new(|eles: &Elements| eles.cloned())),
			data: HashMap::new(),
		}),
	)
	.into();
	rules.push(rule);
}

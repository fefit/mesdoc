use crate::selector::rule::Rule;
pub fn init(rules: &mut Vec<Rule>) {
	let rule = Rule::add(
		r##"[{spaces}{attr_key}${spaces}{regexp#([~|^$*]?)=\s*(?:(['"])((?:(?!\2).)*)\2|([^\s'"<>/=`]+))#}{spaces}]"##,
		vec![("attr_key", 0), ("regexp", 0)],
		Box::new(|nodes, params, count| {
			let attr_key = Rule::param(&params, "attr_key").expect("The 'class' selector is not correct");
			Ok(nodes)
		}),
	);
	rules.push(rule);
}

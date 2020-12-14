use crate::selector::rule::Rule;
pub fn init(rules: &mut Vec<Rule>) {
	let rule = Rule::add(
		".{identity}",
		vec![("identity", 0)],
		Box::new(|nodes, params, count| {
			let class_name =
				Rule::param(&params, "identity").expect("The 'class' selector is not correct");
			Ok(nodes)
		}),
	);
	rules.push(rule);
}

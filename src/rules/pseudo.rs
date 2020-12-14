use crate::selector::rule::Rule;
fn add_empty(rules: &mut Vec<Rule>) {
	// empty
	let rule = Rule::add(":empty", vec![], Box::new(|nodes, params, count| Ok(nodes)));
	rules.push(rule);
}
fn add_first_child(rules: &mut Vec<Rule>) {
	// empty
	let rule = Rule::add(
		":first-child",
		vec![],
		Box::new(|nodes, params, count| Ok(nodes)),
	);
	rules.push(rule);
}
fn add_last_child(rules: &mut Vec<Rule>) {
	// empty
	let rule = Rule::add(
		":last-child",
		vec![],
		Box::new(|nodes, params, count| Ok(nodes)),
	);
	rules.push(rule);
}
fn add_first_of_type(rules: &mut Vec<Rule>) {
	// empty
	let rule = Rule::add(
		":first-of-type",
		vec![],
		Box::new(|nodes, params, count| Ok(nodes)),
	);
	rules.push(rule);
}
pub fn init(rules: &mut Vec<Rule>) {
	add_empty(rules);
	add_first_child(rules);
	add_last_child(rules);
	add_first_of_type(rules);
}

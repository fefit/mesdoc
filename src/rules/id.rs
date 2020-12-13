use crate::selector::rule::{Handle, Rule};
pub fn load(rules: &mut Vec<Rule>) {
  let rule = Rule::add(
    "#{identity}",
    vec![("identity", 0)],
    Box::new(|nodes, params, count| {
			let id = Rule::param(params, "identity").expect("The 'id' is not correct");
		
    }),
  );
  rules.push(rule);
}

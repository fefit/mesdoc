use crate::selector::rule::Rule;
pub fn init(rules: &mut Vec<Rule>) {
  let rule = Rule::add(
    "#{identity}",
    vec![("identity", 0)],
    Box::new(|nodes, params, count| {
      let id = Rule::param(&params, "identity").expect("The 'id' selector is not correct");
      Ok(nodes)
    }),
  );
  rules.push(rule);
}

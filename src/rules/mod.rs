use crate::selector::rule::Rule;

pub mod attr;
pub mod class;
pub mod id;
pub mod name;
pub mod pseudo;
fn load<T: Fn(&mut Vec<Rule>)>(fns: Vec<T>) -> Vec<Rule> {
	let mut rules = Vec::with_capacity(20);
	for handle in fns {
		handle(&mut rules);
	}
	rules
}

pub fn load_rules() -> Vec<Rule> {
	load(vec![id::init])
}

pub mod interface;
pub mod pattern;
pub mod rule;

use lazy_static::lazy_static;
use pattern::Matched;
use rule::{Rule, RULES};
use std::sync::{Arc, Mutex};
lazy_static! {
	static ref SPLITTER: Mutex<Rule> = Mutex::new(Rule::from(r##"{regexp#(\s*[>,~+]\s*|\s+)#}"##));
}
#[derive(Debug, Clone, Copy)]
pub enum Combinator {
	// descendants
	ChildrenAll,
	// children
	Children,
	// reverse for child
	Parent,
	// reverse for childrens
	ParentAll,
	// next all siblings
	NextAll,
	// next sibling
	Next,
	// reverse for next siblings
	PrevAll,
	// reverse for next sibling
	Prev,
	// chain selectors
	Chain,
}

// change string to combinator
impl From<&str> for Combinator {
	fn from(comb: &str) -> Self {
		use Combinator::*;
		match comb {
			"" => ChildrenAll,
			">" => Children,
			"~" => NextAll,
			"+" => Next,
			_ => panic!("Not supported combinator string '{}'", comb),
		}
	}
}

impl Combinator {
	pub fn reverse(&self) -> Self {
		use Combinator::*;
		match self {
			ChildrenAll => ParentAll,
			Children => Parent,
			NextAll => PrevAll,
			Next => Prev,
			Chain => Chain,
			_ => panic!("Not supported combinator reverse for '{:?}'", self),
		}
	}
}

pub type SelectorSegment = (Arc<Rule>, Vec<Matched>, Combinator);
#[derive(Debug)]
pub struct Selector {
	groups: Vec<Vec<SelectorSegment>>,
}

impl Selector {
	fn new() -> Self {
		Selector {
			groups: Vec::with_capacity(1),
		}
	}
	fn add_group(&mut self) -> &mut Self {
		self.groups.push(Vec::with_capacity(2));
		self
	}
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum PrevInSelector {
	Begin,
	Splitter,
	Selector,
}

impl From<&str> for Selector {
	fn from(selector: &str) -> Self {
		let splitter = SPLITTER.lock().unwrap();
		let chars: Vec<char> = selector.chars().collect();
		let total_len = chars.len();
		let mut selector = Selector::new();
		if total_len > 0 {
			let mut index: usize = 0;
			let mut comb = Combinator::ChildrenAll;
			let mut group_index = 0;
			let mut prev_in = PrevInSelector::Begin;
			let mut last_in = prev_in;
			selector.add_group();
			while index < total_len - 1 {
				let next_chars = &chars[index..];
				// first check if combinator
				if let Some((matched, len)) = splitter.exec(next_chars) {
					let op = matched[0].chars.iter().collect::<String>();
					let op = op.trim();
					if prev_in == PrevInSelector::Splitter {
						// wrong multiple combinator
						panic!(
							"Wrong combinator '{}' at index {}",
							matched[0].chars.iter().collect::<String>(),
							index
						);
					}
					// find the match
					index += len;
					// set combinator
					if op == "," {
						if prev_in != PrevInSelector::Selector {
							panic!("Wrong empty selector before ',' at index  {}", index);
						}
						selector.add_group();
						group_index += 1;
						comb = Combinator::ChildrenAll;
					} else {
						comb = Combinator::from(op);
					}
					// set prev is splitter
					if op == "" {
						last_in = prev_in;
						prev_in = PrevInSelector::Splitter;
					} else {
						prev_in = PrevInSelector::Splitter;
						last_in = prev_in;
					}
					continue;
				}
				// then it must match a selector rule
				if prev_in == PrevInSelector::Selector {
					comb = Combinator::Chain;
				} else {
					prev_in = PrevInSelector::Selector;
					last_in = prev_in;
				}
				if let Ok(rules) = RULES.lock() {
					for r in rules.iter() {
						if let Some((matched, len)) = r.exec(next_chars) {
							// find the rule
							index += len;
							// push to selector
							let group_item = selector.groups.get_mut(group_index).unwrap();
							group_item.push((Arc::clone(r), matched, comb));
							break;
						}
					}
					continue;
				}
				// no splitter, no selector rule
				panic!(
					"Unrecognized selector '{}' at index {}",
					next_chars.iter().collect::<String>(),
					index
				);
			}
			if last_in != PrevInSelector::Selector {
				panic!("Wrong selector rule at last")
			}
		}
		selector
	}
}

pub mod interface;
pub mod pattern;
pub mod rule;

use lazy_static::lazy_static;
use pattern::{exec, Matched, Pattern};
use rule::{Rule, RULES};
use std::{
	collections::HashMap,
	sync::{Arc, Mutex},
};

lazy_static! {
	static ref SPLITTER: Mutex<Rule> = Mutex::new(Rule::from(r##"{regexp#(\s*[>,~+]\s*|\s+)#}"##));
	static ref ALL_RULE: Mutex<Option<Arc<Rule>>> = Mutex::new(None);
}
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
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
	// siblings
	Siblings,
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
#[derive(Debug, Default)]
pub struct QueryProcess {
	pub should_in: Option<SelectorGroupsItem>,
	pub query: SelectorGroupsItem,
}

#[derive(Debug, Default)]
pub struct Selector {
	pub process: Vec<QueryProcess>,
}

type SelectorGroupsItem = Vec<Vec<SelectorSegment>>;
type SelectorGroups = Vec<SelectorGroupsItem>;
impl Selector {
	pub fn new() -> Self {
		Selector {
			process: Vec::with_capacity(1),
		}
	}
	pub fn from_str(selector: &str, use_lookup: bool) -> Self {
		let chars: Vec<char> = selector.chars().collect();
		let total_len = chars.len();
		let mut selector = Selector::new();
		if total_len > 0 {
			let mut index: usize = 0;
			let mut comb = Combinator::ChildrenAll;
			let mut prev_in = PrevInSelector::Begin;
			let mut last_in = prev_in;
			let mut groups: SelectorGroups = Vec::new();
			let splitter = SPLITTER.lock().unwrap();
			let rules = RULES.lock().unwrap();
			Selector::add_group(&mut groups);
			while index < total_len {
				let next_chars = &chars[index..];
				// first check if combinator
				if let Some((matched, len, _)) = splitter.exec(next_chars) {
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
						Selector::add_group(&mut groups);
						comb = Combinator::ChildrenAll;
					} else {
						comb = Combinator::from(op);
					}
					// set prev is splitter
					if op.is_empty() {
						last_in = prev_in;
						prev_in = PrevInSelector::Splitter;
					} else {
						prev_in = PrevInSelector::Splitter;
						last_in = prev_in;
					}
					continue;
				}
				// then it must match a selector rule
				let mut is_new_item = true;
				if prev_in == PrevInSelector::Selector {
					comb = Combinator::Chain;
					is_new_item = false;
				} else {
					prev_in = PrevInSelector::Selector;
					last_in = prev_in;
				}
				let mut finded = false;
				for (_, r) in rules.iter() {
					if let Some((mut matched, len, queue_num)) = r.exec(next_chars) {
						// find the rule
						index += len;
						let queues = &r.queues;
						if queue_num == queues.len() {
							// push to selector
							Selector::add_group_item(&mut groups, (Arc::clone(r), matched, comb), is_new_item);
							finded = true;
						} else if queues[queue_num].is_nested() {
							// nested selector
							let (len, nested_matched) = Selector::parse_until(
								&chars[index..],
								&queues[queue_num + 1..],
								&rules,
								&splitter,
								0,
							);
							index += len;
							matched.extend(nested_matched);
							Selector::add_group_item(&mut groups, (Arc::clone(r), matched, comb), is_new_item);
							finded = true;
						}
						break;
					}
				}
				if !finded {
					// no splitter, no selector rule
					panic!(
						"Unrecognized selector '{}' at index {}",
						next_chars.iter().collect::<String>(),
						index
					);
				}
			}
			if last_in != PrevInSelector::Selector {
				panic!("Wrong selector rule at last")
			}
			// optimize groups to query process
			selector.optimize(groups, use_lookup);
		}
		selector
	}
	// add a selector group, splitted by ','
	fn add_group(groups: &mut SelectorGroups) {
		groups.push(Vec::with_capacity(2));
	}
	// add a selector group item
	fn add_group_item(groups: &mut SelectorGroups, item: SelectorSegment, is_new: bool) {
		let last_group = groups.last_mut().unwrap();
		if is_new {
			last_group.push(vec![item]);
		} else {
			last_group.last_mut().unwrap().push(item);
		}
	}
	// optimize the parse process
	fn optimize(&mut self, groups: SelectorGroups, use_lookup: bool) {
		let mut process: Vec<QueryProcess> = Vec::with_capacity(groups.len());
		for mut group in groups {
			// first optimize the chain selectors, the rule who's priority is bigger will apply first
			let mut max_index: usize = 0;
			let mut max_priority: u32 = 0;
			for (index, r) in group.iter_mut().enumerate() {
				let mut total_priority = 0;
				if r.len() > 1 {
					let chain_comb = r[0].2;
					r.sort_by(|a, b| b.0.priority.partial_cmp(&a.0.priority).unwrap());
					let mut now_first = &mut r[0];
					if now_first.2 != chain_comb {
						now_first.2 = chain_comb;
						total_priority += now_first.0.priority;
						for n in &mut r[1..] {
							n.2 = Combinator::Chain;
							total_priority += n.0.priority;
						}
						continue;
					}
				}
				if use_lookup {
					total_priority = r.iter().map(|p| p.0.priority).sum();
					if total_priority > max_priority {
						max_priority = total_priority;
						max_index = index;
					}
				}
			}
			// if the first combinator is child, and the max_index > 1, use the max_index's rule first
			if use_lookup && max_index > 0 {
				let is_child = matches!(
					group[0][0].2,
					Combinator::Children | Combinator::ChildrenAll
				);
				if is_child {
					let query = group.split_off(max_index);
					let should_in = Some(group);
					process.push(QueryProcess { should_in, query });
					continue;
				}
			}
			process.push(QueryProcess {
				should_in: None,
				query: group,
			});
		}
		self.process = process;
	}
	// change the combinator
	pub fn head_combinator(&mut self, comb: Combinator) {
		for p in &mut self.process {
			let v = if let Some(should_in) = &mut p.should_in {
				should_in
			} else {
				&mut p.query
			};
			if let Some(rule) = v.get_mut(0) {
				let first_comb = rule[0].2;
				match first_comb {
					Combinator::ChildrenAll => rule[0].2 = comb,
					_ => {
						let segment = Selector::make_comb_all(comb);
						v.insert(0, vec![segment]);
					}
				};
			}
		}
	}
	// make '*' with combinator
	pub fn make_comb_all(comb: Combinator) -> SelectorSegment {
		let mut all_rule = ALL_RULE.lock().unwrap();
		if all_rule.is_none() {
			let rules = RULES.lock().unwrap();
			*all_rule = rules.get("all").map(|r| Arc::clone(r));
		}
		let cur_rule = Arc::clone(all_rule.as_ref().unwrap());
		(
			cur_rule,
			vec![Matched {
				chars: vec!['*'],
				..Default::default()
			}],
			comb,
		)
	}
	// build a selector from a segment
	pub fn from_segment(segment: SelectorSegment) -> Self {
		let process = QueryProcess {
			query: vec![vec![segment]],
			should_in: None,
		};
		Selector {
			process: vec![process],
		}
	}
	// parse until
	pub fn parse_until(
		chars: &[char],
		until: &[Box<dyn Pattern>],
		rules: &HashMap<&str, Arc<Rule>>,
		splitter: &Rule,
		level: usize,
	) -> (usize, Vec<Matched>) {
		let mut index = 0;
		let total = chars.len();
		let mut matched: Vec<Matched> = Vec::with_capacity(until.len() + 1);
		while index < total {
			let next_chars = &chars[index..];
			if let Some((_, len, _)) = splitter.exec(next_chars) {
				index += len;
				continue;
			}
			let mut finded = false;
			for (_, r) in rules.iter() {
				if let Some((_, len, queue_num)) = r.exec(next_chars) {
					let queues = &r.queues;
					// find the rule
					index += len;
					if queue_num == queues.len() {
						// push to selector
						finded = true;
					} else {
						let (nest_count, _) = Selector::parse_until(
							&chars[index..],
							&queues[queue_num + 1..],
							rules,
							splitter,
							level + 1,
						);
						index += nest_count;
					}
					break;
				}
			}
			if !finded {
				if level == 0 {
					matched.push(Matched {
						chars: chars[0..index].iter().copied().collect(),
						name: "selector",
						..Default::default()
					});
				}
				if !until.is_empty() {
					let (util_matched, count, queue_num, _) = exec(until, &chars[index..]);
					if queue_num != until.len() {
						panic!("nested selector parse error");
					} else {
						index += count;
						if level == 0 {
							matched.extend(util_matched);
						}
					}
				}
				break;
			}
		}
		(index, matched)
	}
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum PrevInSelector {
	Begin,
	Splitter,
	Selector,
}

impl From<&str> for Selector {
	fn from(selector: &str) -> Self {
		Selector::from_str(selector, true)
	}
}

use super::interface::{NodeList, Result as NResult};
use super::pattern::{self, exec, to_pattern, Matched, Pattern};
use crate::utils::{to_static_str, vec_char_to_clean_str};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};
lazy_static! {
	pub static ref RULES: Mutex<HashMap<&'static str, Arc<Rule>>> =
		Mutex::new(HashMap::with_capacity(20));
}

pub type RuleMatchedData = HashMap<SavedDataKey, &'static str>;
pub type Handle =
	Box<dyn (for<'a, 'r> Fn(&'a NodeList<'r>, &'a RuleMatchedData) -> NResult<'r>) + Send + Sync>;

pub type AliasRule = Box<dyn (Fn(&[Matched]) -> &'static str) + Send + Sync>;
pub struct Rule {
	pub in_cache: bool,
	pub priority: u32,
	pub(crate) queues: Vec<Box<dyn Pattern>>,
	fields: Vec<DataKey>,
	handle: Option<Handle>,
	alias: Option<AliasRule>,
}

impl fmt::Debug for Rule {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(format!("Rule{{ queues: {:?} }}", self.queues).as_str())
	}
}
#[derive(Debug, Hash, Eq, PartialEq)]
pub struct SavedDataKey(&'static str, usize, &'static str);
pub type DataKey = (&'static str, usize);

impl From<(&'static str,)> for SavedDataKey {
	fn from(t: (&'static str,)) -> Self {
		SavedDataKey(t.0, 0, "_")
	}
}

impl From<(&'static str, usize)> for SavedDataKey {
	fn from(t: (&'static str, usize)) -> Self {
		SavedDataKey(t.0, t.1, "_")
	}
}

impl From<(&'static str, usize, &'static str)> for SavedDataKey {
	fn from(t: (&'static str, usize, &'static str)) -> Self {
		SavedDataKey(t.0, t.1, t.2)
	}
}

impl From<&'static str> for SavedDataKey {
	fn from(s: &'static str) -> Self {
		(s,).into()
	}
}

// get char vec
const DEF_SIZE: usize = 2;
fn get_char_vec() -> Vec<char> {
	Vec::with_capacity(DEF_SIZE)
}

// unmatched start or end
fn panic_unmatched(ch: char, index: usize) -> ! {
	panic!(
		"Unmatched '{ch}' at index {index},you can escape it using both {ch}{ch}",
		ch = ch,
		index = index
	)
}

struct MatchedStore {
	hashs_num: usize,
	is_wait_end: bool,
	is_in_matched: bool,
	raw_params: Vec<char>,
	suf_params: Vec<char>,
	names: Vec<char>,
}

impl Default for MatchedStore {
	fn default() -> Self {
		MatchedStore {
			hashs_num: 0,
			is_wait_end: false,
			is_in_matched: false,
			raw_params: get_char_vec(),
			suf_params: get_char_vec(),
			names: get_char_vec(),
		}
	}
}
impl MatchedStore {
	fn next(&mut self) -> Result<Box<dyn Pattern>, String> {
		self.hashs_num = 0;
		self.is_in_matched = false;
		self.is_wait_end = false;
		let name = vec_char_to_clean_str(&mut self.names);
		let s = vec_char_to_clean_str(&mut self.suf_params);
		let r = vec_char_to_clean_str(&mut self.raw_params);
		to_pattern(name, s, r)
	}
}

impl From<&str> for Rule {
	/// generate a rule from string.
	fn from(content: &str) -> Self {
		const ANCHOR_CHAR: char = '\0';
		const START_CHAR: char = '{';
		const END_CHAR: char = '}';
		let mut prev_char = ANCHOR_CHAR;
		let mut store: MatchedStore = Default::default();
		let mut raw_chars = get_char_vec();
		let mut queues: Vec<Box<dyn Pattern>> = Vec::with_capacity(DEF_SIZE);
		let mut is_matched_finish = false;
		let mut index: usize = 0;
		for ch in content.chars() {
			index += 1;
			let is_prev_matched_finish = if is_matched_finish {
				is_matched_finish = false;
				true
			} else {
				false
			};
			if store.is_wait_end {
				if ch.is_ascii_whitespace() {
					continue;
				}
				if ch == END_CHAR {
					is_matched_finish = true;
				} else {
					panic!(
						"Unexpect end of Pattern type '{}' at index {}, expect '{}' but found '{}'",
						vec_char_to_clean_str(&mut store.names),
						index - 1,
						END_CHAR,
						ch
					);
				}
			} else if !store.is_in_matched {
				// when not in matched
				if prev_char == START_CHAR {
					// translate '{'
					if ch == START_CHAR {
						prev_char = ANCHOR_CHAR;
						continue;
					} else {
						store.names.push(ch);
						store.is_in_matched = true;
						// remove the '{'
						raw_chars.pop();
						if !raw_chars.is_empty() {
							queues.push(Box::new(raw_chars.clone()));
							raw_chars.clear();
						}
					}
				} else if prev_char == END_CHAR {
					if is_prev_matched_finish {
						// is just end of the Pattern type.
						raw_chars.push(ch);
					} else if ch == END_CHAR {
						// translate end char '}'
						prev_char = ANCHOR_CHAR;
						continue;
					} else {
						// panic no matched
						panic_unmatched(END_CHAR, index - 2);
					}
				} else {
					raw_chars.push(ch);
				}
			} else if !store.raw_params.is_empty() {
				// in Pattern's raw params ##gfh#def##
				if ch == '#' {
					let leave_count = store.hashs_num - 1;
					if leave_count == 0 {
						// only one hash
						store.is_wait_end = true;
					} else {
						let raw_len = store.raw_params.len();
						let last_index = raw_len - leave_count;
						if last_index > 0 {
							store.is_wait_end = store.raw_params[last_index..]
								.iter()
								.filter(|&&ch| ch == '#')
								.count() == leave_count;
							if store.is_wait_end {
								store.raw_params.truncate(last_index);
							}
						}
					}
					if !store.is_wait_end {
						store.raw_params.push(ch);
					}
				} else {
					// in raw params
					store.raw_params.push(ch);
				}
			} else {
				// in suf_params or names
				if ch == '}' {
					if store.hashs_num > 0 {
						panic!("Uncomplete raw params: ''");
					}
					is_matched_finish = true;
				} else if ch == '#' {
					// in hashs
					store.hashs_num += 1;
				} else {
					// in names or suf_params
					if prev_char == '#' {
						// in raw params now
						store.raw_params.push(ch);
					} else {
						// check if in suf_params, or character is not a name's character
						if !store.suf_params.is_empty() || !(ch.is_ascii_alphanumeric() || ch == '_') {
							store.suf_params.push(ch);
						} else {
							store.names.push(ch);
						}
					}
				}
			}
			if is_matched_finish {
				match store.next() {
					Ok(queue) => queues.push(queue),
					Err(reason) => panic!(reason),
				};
			}
			prev_char = ch;
		}
		// not end
		if store.is_wait_end || store.is_in_matched {
			panic!(
				"The Mathed type '{}' is not complete",
				store.names.iter().collect::<String>()
			);
		}
		if prev_char == START_CHAR || (prev_char == END_CHAR && !is_matched_finish) {
			panic_unmatched(prev_char, index - 1);
		}
		if !raw_chars.is_empty() {
			if raw_chars.len() == 1 {
				queues.push(Box::new(raw_chars[0]));
			} else {
				queues.push(Box::new(raw_chars));
			}
		}
		Rule {
			queues,
			fields: Vec::with_capacity(3),
			handle: None,
			alias: None,
			priority: 0,
			in_cache: false,
		}
	}
}

impl Rule {
	pub fn exec(&self, chars: &[char]) -> Option<(Vec<Matched>, usize, usize)> {
		let (result, matched_len, matched_queue_item, _) = exec(&self.queues, chars);
		if matched_len > 0 {
			Some((result, matched_len, matched_queue_item))
		} else {
			None
		}
	}
	pub fn apply<'a, 'r>(&self, node_list: &'a NodeList<'r>, matched: &[Matched]) -> NResult<'r> {
		if let Some(alias) = &self.alias {
			let rule = alias(matched);
			node_list.filter(rule)
		} else {
			let handle = self
      .handle
      .as_ref()
      .expect("The rule's handle must set before call `exec`,you should use `set_params` to set the handle.");
			let params = self.data(matched);
			handle(node_list, &params)
		}
	}
	pub fn data(&self, data: &[Matched]) -> RuleMatchedData {
		let mut result: RuleMatchedData = HashMap::with_capacity(5);
		let mut indexs = HashMap::with_capacity(5);
		let fields = &self.fields;
		for item in data.iter() {
			let Matched {
				name,
				data: hash_data,
				chars,
			} = item;
			if !name.is_empty() {
				let index = indexs.entry(name).or_insert(0);
				let data_key = (*name, *index);
				if fields.contains(&data_key) {
					let count = hash_data.len();
					if count == 0 {
						let cur_key = (*name, *index);
						result.insert(
							cur_key.into(),
							to_static_str(chars.iter().collect::<String>()),
						);
					} else {
						for (&key, &val) in hash_data.iter() {
							let cur_key = (*name, *index, key);
							result.insert(cur_key.into(), val);
						}
					}
				}
			}
		}
		result
	}
	// set the data fields need to collect
	pub fn set_params(
		this: &mut Self,
		priority: u32,
		in_cache: bool,
		fields: Vec<DataKey>,
		handle: Option<Handle>,
		alias: Option<AliasRule>,
	) {
		if !this.fields.is_empty() {
			panic!("The rule's `set_params` can only call once");
		}
		this.fields = fields;
		this.handle = handle;
		this.priority = priority;
		this.in_cache = in_cache;
		this.alias = alias;
	}
	// add a rule
	pub fn add(
		context: &str,
		priority: u32,
		in_cache: bool,
		fields: Vec<DataKey>,
		handle: Option<Handle>,
		alias: Option<AliasRule>,
	) -> Self {
		let mut rule: Rule = context.into();
		Rule::set_params(&mut rule, priority, in_cache, fields, handle, alias);
		rule
	}
	// quick method to get param
	pub fn param<T: Into<SavedDataKey>>(params: &RuleMatchedData, v: T) -> Option<&str> {
		params.get(&v.into()).copied()
	}
}

pub struct RuleDefItem(
	pub &'static str,
	pub &'static str,
	pub u32,
	pub Vec<DataKey>,
	pub Handle,
);
pub struct RuleAliasItem(
	pub &'static str,
	pub &'static str,
	pub u32,
	pub Vec<DataKey>,
	pub AliasRule,
);

pub type RuleItem = (
	&'static str,
	&'static str,
	u32,
	bool,
	Vec<DataKey>,
	Option<Handle>,
	Option<AliasRule>,
);

impl From<RuleDefItem> for RuleItem {
	fn from(item: RuleDefItem) -> Self {
		(item.0, item.1, item.2, false, item.3, Some(item.4), None)
	}
}

impl From<RuleAliasItem> for RuleItem {
	fn from(item: RuleAliasItem) -> Self {
		(item.0, item.1, item.2, false, item.3, None, Some(item.4))
	}
}

pub fn add_rules(rules: Vec<RuleItem>) {
	let mut all_rules = RULES.lock().unwrap();
	for (name, context, priority, in_cache, fields, handle, alias) in rules {
		let cur_rule = Rule::add(context, priority, in_cache, fields, handle, alias);
		all_rules.insert(name, Arc::new(cur_rule));
	}
}

pub(crate) fn init() {
	pattern::init();
}

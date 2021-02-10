use std::{
	collections::{HashMap, HashSet},
	ops::Range,
};

use crate::interface::{BoxDynElement, Elements, INodeType};
use crate::selector::rule::{Rule, RuleDefItem, RuleItem};
use crate::selector::{
	pattern::Nth,
	rule::{RuleAliasItem, RuleMatchedData},
};

const PRIORITY: u32 = 10;
const DEF_NODES_LEN: usize = 5;
/// pseudo selector ":empty"
fn pseudo_empty(rules: &mut Vec<RuleItem>) {
	// empty
	let selector = ":empty";
	let name = selector;
	let rule = RuleDefItem(
		name,
		selector,
		PRIORITY,
		vec![],
		Box::new(|eles: &Elements, _| -> Elements {
			let mut result = Elements::with_capacity(DEF_NODES_LEN);
			for node in eles.get_ref() {
				let child_nodes = node.child_nodes();
				if child_nodes.is_empty() {
					result.push(node.cloned());
				} else {
					let mut only_comments = true;
					for node in child_nodes {
						match node.node_type() {
							INodeType::Comment => continue,
							_ => {
								only_comments = false;
								break;
							}
						}
					}
					if only_comments {
						result.push(node.cloned());
					}
				}
			}
			result
		}),
	);
	rules.push(rule.into());
}

// make rule for ':first-child', ':last-child'
fn make_first_or_last_child(selector: &'static str, is_first: bool) -> RuleDefItem {
	let name = selector;
	RuleDefItem(
		name,
		selector,
		PRIORITY,
		vec![],
		Box::new(move |eles: &Elements, _: &RuleMatchedData| -> Elements {
			let mut result = Elements::with_capacity(DEF_NODES_LEN);
			let get_index = if is_first {
				|_: usize| 0
			} else {
				|total: usize| total - 1
			};
			for node in eles.get_ref() {
				if let Some(parent) = node.parent() {
					let childs = parent.children();
					let index = get_index(childs.length());
					if let Some(child) = childs.get(index) {
						if node.is(&child) {
							result.push(node.cloned());
						}
					}
				}
			}
			result
		}),
	)
}

/// pseudo selector `:first-child,:last-child`
fn pseudo_first_child(rules: &mut Vec<RuleItem>) {
	// last_child
	let rule = make_first_or_last_child(":first-child", true);
	rules.push(rule.into());
}

fn pseudo_last_child(rules: &mut Vec<RuleItem>) {
	// last_child
	let rule = make_first_or_last_child(":last-child", false);
	rules.push(rule.into());
}

// group siblings
struct SiblingsNodeData<'a> {
	range: Range<usize>,
	allow_indexs: Option<Vec<usize>>,
	parent: Option<BoxDynElement<'a>>,
}

fn group_siblings_then_done<T, F>(eles: &Elements, allow_indexs_fn: T, mut cb: F)
where
	T: Fn(usize) -> Option<Vec<usize>>,
	F: FnMut(&mut SiblingsNodeData),
{
	let mut data = SiblingsNodeData {
		range: 0..0,
		allow_indexs: None,
		parent: None,
	};
	for (index, ele) in eles.get_ref().iter().enumerate() {
		if let Some(parent) = ele.parent() {
			let mut is_first = false;
			let mut in_next_group = false;
			if let Some(prev_parent) = &data.parent {
				if parent.is(&prev_parent) {
					// sibling node, just add
					data.range.end = index + 1;
				} else {
					// not sibling
					in_next_group = true;
				}
			} else {
				is_first = true;
			}
			// when meet next group siblings
			if in_next_group {
				cb(&mut data);
			}
			// when is first or in next group
			if is_first || in_next_group {
				// init the siblings, allow_index, prev_parent
				data.range.start = index;
				data.range.end = index + 1;
				data.parent = Some(parent.cloned());
				data.allow_indexs = allow_indexs_fn(parent.children().length());
			}
		}
	}
	if !data.range.is_empty() {
		cb(&mut data);
	}
}
// make for 'nth-child','nth-last-child'
fn make_asc_or_desc_nth_child(selector: &'static str, asc: bool) -> RuleDefItem {
	let handle = if asc {
		|eles: &Elements,
		 range: &Range<usize>,
		 allow_indexs: &[usize],
		 childs: &Elements|
		 -> Vec<BoxDynElement> {
			// do with the siblings
			let child_nodes = childs.get_ref();
			let mut finded: Vec<BoxDynElement> = Vec::with_capacity(allow_indexs.len());
			// optimize if loop all the childs
			if range.len() == child_nodes.len() {
				// get all by indexs
				for &index in allow_indexs {
					finded.push(child_nodes[index].cloned());
				}
			} else {
				let mut cur_start = 0;
				let eles = eles.get_ref();
				let siblings = &eles[range.start..range.end];
				for &index in allow_indexs {
					let child = &child_nodes[index];
					let mut has_matched = false;
					for (i, ele) in siblings[cur_start..].iter().enumerate() {
						if child.is(ele) {
							cur_start += i + 1;
							has_matched = true;
							finded.push(ele.cloned());
							break;
						}
					}
					// break if not find a matched or at the end
					if !has_matched || cur_start == range.end {
						break;
					}
				}
			}
			finded
		}
	} else {
		|eles: &Elements, range: &Range<usize>, allow_indexs: &[usize], childs: &Elements| {
			// do with the siblings
			let child_nodes = childs.get_ref();
			let total = child_nodes.len();
			let mut finded: Vec<BoxDynElement> = Vec::with_capacity(allow_indexs.len());
			// optimize when loop all the childrens
			if range.len() == total {
				for &index in allow_indexs.iter().rev() {
					finded.push(child_nodes[total - index - 1].cloned());
				}
			} else {
				let eles = eles.get_ref();
				let siblings = &eles[range.start..range.end];
				let mut cur_end = range.len();
				for (index, child) in child_nodes.iter().rev().enumerate() {
					let last_index = total - index - 1;
					// use binary search for faster speed
					if allow_indexs.binary_search(&last_index).is_err() {
						continue;
					}
					let mut has_matched = false;
					for (i, ele) in siblings[..cur_end].iter().rev().enumerate() {
						if child.is(ele) {
							cur_end -= i + 1;
							has_matched = true;
							finded.push(ele.cloned());
							break;
						}
					}
					if !has_matched || cur_end == 0 {
						break;
					}
				}
				finded.reverse();
			}
			finded
		}
	};
	let name = if asc { ":nth-child" } else { ":nth-last-child" };
	RuleDefItem(
		name,
		selector,
		PRIORITY,
		vec![("nth", 0)],
		Box::new(
			move |eles: &Elements, params: &RuleMatchedData| -> Elements {
				let n = Rule::param(&params, ("nth", 0, "n"));
				let index = Rule::param(&params, ("nth", 0, "index"));
				let mut result: Elements = Elements::with_capacity(DEF_NODES_LEN);
				group_siblings_then_done(
					eles,
					|total: usize| Some(Nth::get_allowed_indexs(n, index, total)),
					|data: &mut SiblingsNodeData| {
						let allow_indexs = data.allow_indexs.as_ref().expect("allow indexs must set");
						if allow_indexs.is_empty() {
							return;
						}
						let childs = data
							.parent
							.as_ref()
							.expect("parent must set in callback")
							.children();
						let finded = handle(&eles, &data.range, &allow_indexs, &childs);
						if !finded.is_empty() {
							result.get_mut_ref().extend(finded);
						}
					},
				);
				result
			},
		),
	)
}
/// pseudo selector: `:nth-child`
fn pseudo_nth_child(rules: &mut Vec<RuleItem>) {
	let rule = make_asc_or_desc_nth_child(":nth-child({spaces}{nth}{spaces})", true);
	rules.push(rule.into());
}

/// pseudo selector: `:nth-child`
fn pseudo_nth_last_child(rules: &mut Vec<RuleItem>) {
	let rule = make_asc_or_desc_nth_child(":nth-last-child({spaces}{nth}{spaces})", false);
	rules.push(rule.into());
}

type NameCountHashMap = HashMap<String, usize>;

// find the element
fn find_matched_ele(
	excludes: &mut Vec<usize>,
	index: usize,
	name: &str,
	ele: &BoxDynElement,
	child: &BoxDynElement,
	finded: &mut Vec<BoxDynElement>,
) -> bool {
	// in excludes, continue to ignore
	if excludes.contains(&index) {
		return false;
	}
	let cur_name = ele.tag_name();
	if cur_name == name {
		excludes.push(index);
		if ele.is(child) {
			finded.push(ele.cloned());
			return true;
		}
	}
	false
}

// make for ':first-of-type', ':last-of-type'
fn make_first_or_last_of_type(selector: &'static str, is_first: bool) -> RuleDefItem {
	let name = selector;
	// last of type
	RuleDefItem(
		name,
		selector,
		PRIORITY,
		vec![],
		Box::new(move |eles: &Elements, _: &RuleMatchedData| -> Elements {
			let mut result: Elements = Elements::with_capacity(DEF_NODES_LEN);
			// check if has name, otherwise insert
			fn has_name(names: &mut HashSet<String>, name: &str) -> bool {
				// not the first type
				if names.get(name).is_some() {
					// continue to next loop, ignore the next process
					return true;
				}
				// the first type of the name tag
				names.insert(String::from(name));
				false
			}

			group_siblings_then_done(
				eles,
				|_| None,
				|data: &mut SiblingsNodeData| {
					let childs = data
						.parent
						.as_ref()
						.expect("parent must set in callback")
						.children();
					// collect the names
					let mut names: HashSet<String> = HashSet::with_capacity(5);
					// eles
					let eles = eles.get_ref();
					let range = &data.range;
					let siblings = &eles[range.start..range.end];
					// exclude detected names
					let mut excludes: Vec<usize> = Vec::with_capacity(siblings.len());
					if is_first {
						let finded = result.get_mut_ref();
						// asc
						for child in childs.get_ref() {
							let name = child.tag_name();
							// check name then find
							if !has_name(&mut names, name) {
								for (index, ele) in siblings.iter().enumerate() {
									if find_matched_ele(&mut excludes, index, name, ele, child, finded) {
										break;
									}
								}
							}
						}
					} else {
						let mut finded: Vec<BoxDynElement> = Vec::with_capacity(5);
						// desc find
						for child in childs.get_ref().iter().rev() {
							let name = child.tag_name();
							if !has_name(&mut names, name) {
								for (index, ele) in siblings.iter().rev().enumerate() {
									if find_matched_ele(&mut excludes, index, name, ele, child, &mut finded) {
										break;
									}
								}
							}
						}
						if !finded.is_empty() {
							finded.reverse();
							result.get_mut_ref().extend(finded);
						}
					}
				},
			);
			result
		}),
	)
}

/// pseudo selector:`:first-of-type `
fn pseudo_first_of_type(rules: &mut Vec<RuleItem>) {
	// last of type
	let rule = make_first_or_last_of_type(":first-of-type", true);
	rules.push(rule.into());
}

/// pseudo selector:`:last-of-type`
fn pseudo_last_of_type(rules: &mut Vec<RuleItem>) {
	// last of type
	let rule = make_first_or_last_of_type(":last-of-type", false);
	rules.push(rule.into());
}

// make nth of type: `:nth-of-type`, `:nth-last-of-type`
fn make_asc_or_desc_nth_of_type(selector: &'static str, asc: bool) -> RuleDefItem {
	let name = if asc {
		":nth-of-type"
	} else {
		":nth-last-of-type"
	};
	// check if cur tag's name is ok
	fn is_name_ok(name: &str, names: &mut NameCountHashMap, allow_indexs: &[usize]) -> bool {
		if let Some(index) = names.get_mut(name) {
			// increase index
			*index += 1;
			// check if last_index is big than index
			let last_index = allow_indexs[allow_indexs.len() - 1];
			if *index > last_index {
				false
			} else {
				allow_indexs.contains(index)
			}
		} else {
			let index = 0;
			names.insert(String::from(name), index);
			allow_indexs.contains(&index)
		}
	}
	// last of type
	RuleDefItem(
		name,
		selector,
		PRIORITY,
		vec![("nth", 0)],
		Box::new(
			move |eles: &Elements, params: &RuleMatchedData| -> Elements {
				let mut result: Elements = Elements::with_capacity(DEF_NODES_LEN);
				let n = Rule::param(&params, ("nth", 0, "n"));
				let index = Rule::param(&params, ("nth", 0, "index"));
				group_siblings_then_done(
					eles,
					|total: usize| Some(Nth::get_allowed_indexs(n, index, total)),
					|data: &mut SiblingsNodeData| {
						let allow_indexs = data.allow_indexs.as_ref().expect("allow indexs must set");
						// return if allow_indexs is empty
						if allow_indexs.is_empty() {
							return;
						}
						// childs
						let childs = data
							.parent
							.as_ref()
							.expect("parent must set in callback")
							.children();
						let mut names: NameCountHashMap = HashMap::with_capacity(5);
						let range = &data.range;
						let eles = eles.get_ref();
						let siblings = &eles[range.start..range.end];
						let mut excludes: Vec<usize> = Vec::with_capacity(siblings.len());
						// loop
						if asc {
							let finded = result.get_mut_ref();
							for child in childs.get_ref() {
								let name = child.tag_name();
								if is_name_ok(name, &mut names, allow_indexs) {
									// loop the eles
									for (index, ele) in siblings.iter().enumerate() {
										if find_matched_ele(&mut excludes, index, name, ele, child, finded) {
											break;
										}
									}
								}
							}
						} else {
							let mut finded: Vec<BoxDynElement> = Vec::with_capacity(5);
							for child in childs.get_ref().iter().rev() {
								let name = child.tag_name();
								if is_name_ok(name, &mut names, allow_indexs) {
									// loop the eles
									for (index, ele) in siblings.iter().rev().enumerate() {
										if find_matched_ele(&mut excludes, index, name, ele, child, &mut finded) {
											break;
										}
									}
								}
							}
							if !finded.is_empty() {
								finded.reverse();
								result.get_mut_ref().extend(finded);
							}
						}
					},
				);
				result
			},
		),
	)
}

/// pseudo selector:`:nth-of-type`
fn pseudo_nth_of_type(rules: &mut Vec<RuleItem>) {
	// last of type
	let rule = make_asc_or_desc_nth_of_type(":nth-of-type({spaces}{nth}{spaces})", true);
	rules.push(rule.into());
}

/// pseudo selector:`:nth-last-of-type`
fn pseudo_nth_last_of_type(rules: &mut Vec<RuleItem>) {
	// last of type
	let rule = make_asc_or_desc_nth_of_type(":nth-last-of-type({spaces}{nth}{spaces})", false);
	rules.push(rule.into());
}

/// pseudo selector: `only-child`
fn pseudo_only_child(rules: &mut Vec<RuleItem>) {
	let selector = ":only-child";
	let name = selector;
	let rule = RuleDefItem(
		name,
		selector,
		PRIORITY,
		vec![],
		Box::new(move |eles: &Elements, _| -> Elements {
			let mut result = Elements::with_capacity(DEF_NODES_LEN);
			for node in eles.get_ref() {
				if let Some(parent) = node.parent() {
					let childs = parent.children();
					if childs.length() == 1 {
						result.push(node.cloned());
					}
				}
			}
			result
		}),
	);
	rules.push(rule.into());
}

/// pseudo selector: `only-child`
fn pseudo_only_of_type(rules: &mut Vec<RuleItem>) {
	let selector = ":only-of-type";
	let name = selector;
	let rule = RuleDefItem(
		name,
		selector,
		PRIORITY,
		vec![],
		Box::new(move |eles: &Elements, _| -> Elements {
			let mut result = Elements::with_capacity(DEF_NODES_LEN);
			group_siblings_then_done(
				eles,
				|_| None,
				|data: &mut SiblingsNodeData| {
					let childs = data
						.parent
						.as_ref()
						.expect("parent must set in callback")
						.children();
					let eles = eles.get_ref();
					let range = &data.range;
					let siblings = &eles[range.start..range.end];
					let mut names: HashMap<String, bool> = HashMap::with_capacity(DEF_NODES_LEN);
					let mut excludes: Vec<usize> = Vec::with_capacity(siblings.len());
					for child in childs.get_ref() {
						let name = child.tag_name();
						if let Some(removed_done) = names.get_mut(name) {
							if *removed_done {
								// has removed the not only type
							} else {
								// remove not only type
								let mut repeat_indexs: Vec<usize> = Vec::with_capacity(siblings.len());
								for (index, ele) in siblings.iter().enumerate() {
									if excludes.contains(&index) {
										continue;
									}
									if ele.tag_name() == name {
										excludes.push(index);
									}
								}
								*removed_done = true;
							}
						} else {
							names.insert(String::from(name), false);
						}
					}
					if !siblings.is_empty() {
						for node in siblings {
							result.push(node.cloned());
						}
					}
				},
			);
			result
		}),
	);
	rules.push(rule.into());
}

/// pseudo selector: `:not`
fn pseudo_not(rules: &mut Vec<RuleItem>) {
	let name = ":not";
	let selector = ":not({spaces}{selector}{spaces})";
	let rule = RuleDefItem(
		name,
		selector,
		PRIORITY,
		vec![("selector", 0)],
		Box::new(|eles: &Elements, params: &RuleMatchedData| -> Elements {
			let selector = Rule::param(&params, "selector").expect("selector param must have.");
			eles.not(selector)
		}),
	);
	rules.push(rule.into());
}

/// pseudo selector: `:contains`
fn pseudo_contains(rules: &mut Vec<RuleItem>) {
	let name = ":contains";
	let selector =
		r##":contains({spaces}{regexp#(?:'((?:\\?+.)*?)'|"((?:\\?+.)*?)"|([^\s'"<>/=`]*))#}{spaces})"##;
	let rule = RuleDefItem(
		name,
		selector,
		PRIORITY,
		vec![("regexp", 0)],
		Box::new(|eles: &Elements, params: &RuleMatchedData| -> Elements {
			let search = Rule::param(&params, ("regexp", 0, "1"))
				.or_else(|| Rule::param(&params, ("regexp", 0, "2")))
				.or_else(|| Rule::param(&params, ("regexp", 0, "3")))
				.expect("The :contains selector must have a content");
			if search.is_empty() {
				return eles.cloned();
			}
			eles.filter_by(|_, ele| ele.text().contains(search))
		}),
	);
	rules.push(rule.into());
}

// -----------jquery selectors----------

/// pseudo selector: `:header`
fn pseudo_alias_header(rules: &mut Vec<RuleItem>) {
	let selector = ":header";
	let name = selector;
	let rule = RuleAliasItem(
		name,
		selector,
		PRIORITY,
		vec![],
		Box::new(|_| "h1,h2,h3,h4,h5,h6"),
	);
	rules.push(rule.into());
}

/// pseudo selector: `:input`
fn pseudo_alias_input(rules: &mut Vec<RuleItem>) {
	let selector = ":input";
	let name = selector;
	let rule = RuleAliasItem(
		name,
		selector,
		PRIORITY,
		vec![],
		Box::new(|_| "input,select,textarea,button"),
	);
	rules.push(rule.into());
}

/// pseudo selector: `:submit`
fn pseudo_alias_submit(rules: &mut Vec<RuleItem>) {
	let selector = ":submit";
	let name = selector;
	let rule = RuleAliasItem(
		name,
		selector,
		PRIORITY,
		vec![],
		Box::new(|_| "input[type='submit'],button[type='submit']"),
	);
	rules.push(rule.into());
}

pub fn init(rules: &mut Vec<RuleItem>) {
	pseudo_empty(rules);
	// first-child, last-child
	pseudo_first_child(rules);
	pseudo_last_child(rules);
	// only-child
	pseudo_only_child(rules);
	// nth-child,nth-last-child
	pseudo_nth_child(rules);
	pseudo_nth_last_child(rules);
	// first-of-type,last-of-type
	pseudo_first_of_type(rules);
	pseudo_last_of_type(rules);
	// nth-of-type,nth-last-of-type
	pseudo_nth_of_type(rules);
	pseudo_nth_last_of_type(rules);
	// only-of-type
	pseudo_only_of_type(rules);
	// not
	pseudo_not(rules);
	// contains
	pseudo_contains(rules);
	// ---- jquery selectors -----
	// :header alias
	pseudo_alias_header(rules);
	// :input alias
	pseudo_alias_input(rules);
	// :submit alias
	pseudo_alias_submit(rules);
}

use std::collections::{HashMap, HashSet};

use crate::selector::{
	interface::{BoxDynNode, INodeType, NodeList, Result},
	pattern::Nth,
	rule::{RuleAliasItem, RuleMatchedData},
};
use crate::{
	selector::{
		interface::EmptyResult,
		rule::{Rule, RuleDefItem, RuleItem},
	},
	utils::retain_by_index,
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
		Box::new(|nodes: &NodeList, _| -> Result {
			let mut result = NodeList::with_capacity(DEF_NODES_LEN);
			for node in nodes.get_ref() {
				let child_nodes = node.child_nodes()?;
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
			Ok(result)
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
		Box::new(move |nodes: &NodeList, _: &RuleMatchedData| -> Result {
			let mut result = NodeList::with_capacity(DEF_NODES_LEN);
			let get_index = if is_first {
				|_: usize| 0
			} else {
				|total: usize| total - 1
			};
			for node in nodes.get_ref() {
				if let Some(parent) = node.parent()? {
					let childs = parent.children()?;
					let index = get_index(childs.length());
					if let Some(child) = childs.get(index) {
						if node.is(&child) {
							result.push(node.cloned());
						}
					}
				}
			}
			Ok(result)
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
	siblings: Vec<BoxDynNode<'a>>,
	allow_indexs: Option<Vec<usize>>,
	parent: Option<BoxDynNode<'a>>,
}

fn group_siblings_then_done<T, F>(nodes: &NodeList, allow_indexs_fn: T, mut cb: F) -> EmptyResult
where
	T: Fn(usize) -> Option<Vec<usize>>,
	F: FnMut(&mut SiblingsNodeData) -> EmptyResult,
{
	let total = nodes.length();
	let mut data = SiblingsNodeData {
		siblings: Vec::with_capacity(total),
		allow_indexs: None,
		parent: None,
	};
	for (i, node) in nodes.get_ref().iter().enumerate() {
		if let Some(parent) = node.parent()? {
			let mut in_next_group = false;
			let mut is_first = false;
			let is_last = i == total - 1;
			if let Some(prev_parent) = &data.parent {
				if parent.is(&prev_parent) {
					// sibling node, just add
					data.siblings.push(node.cloned());
				} else {
					// not sibling
					in_next_group = true;
				}
			} else {
				is_first = true;
			}
			// when meet next group siblings
			if in_next_group {
				cb(&mut data)?;
			}
			// when is first or in next group
			if is_first || in_next_group {
				// init the siblings, allow_index, prev_parent
				data.siblings = vec![node.cloned()];
				data.parent = Some(parent.cloned());
				data.allow_indexs = allow_indexs_fn(parent.children()?.length());
			}
			// when is last
			if is_last {
				cb(&mut data)?;
				break;
			}
		}
	}
	Ok(())
}
// make for 'nth-child','nth-last-child'
fn make_asc_or_desc_nth_child(selector: &'static str, asc: bool) -> RuleDefItem {
	let handle = if asc {
		|siblings: &mut Vec<BoxDynNode>, allow_indexs: &[usize], childs: &NodeList| -> Vec<BoxDynNode> {
			// do with the siblings
			let child_nodes = childs.get_ref();
			let mut finded: Vec<BoxDynNode> = Vec::with_capacity(5);
			for &index in allow_indexs {
				if let Some(child) = child_nodes.get(index) {
					let mut find_index: Option<usize> = None;
					for (idx, node) in siblings.iter().enumerate() {
						if node.is(child) {
							finded.push(node.cloned());
							find_index = Some(idx);
							break;
						}
					}
					if let Some(find_index) = find_index {
						// the last one node
						if siblings.len() == 1 {
							break;
						}
						// remove from the siblings queue
						siblings.remove(find_index);
					}
				}
			}
			finded
		}
	} else {
		|siblings: &mut Vec<BoxDynNode>, allow_indexs: &[usize], childs: &NodeList| {
			// do with the siblings
			let child_nodes = childs.get_ref();
			let mut finded: Vec<BoxDynNode> = Vec::with_capacity(5);
			for (i, child) in child_nodes.iter().rev().enumerate() {
				if !allow_indexs.contains(&i) {
					continue;
				}
				let mut find_index: Option<usize> = None;
				let total = siblings.len();
				for (idx, node) in siblings.iter().rev().enumerate() {
					if node.is(child) {
						finded.push(node.cloned());
						find_index = Some(total - idx - 1);
						break;
					}
				}
				if let Some(find_index) = find_index {
					// the last one node
					if siblings.len() == 1 {
						break;
					}
					// remove from the siblings queue
					siblings.remove(find_index);
				}
			}
			finded.reverse();
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
			move |nodes: &NodeList, params: &RuleMatchedData| -> Result {
				let n = Rule::param(&params, ("nth", 0, "n"));
				let index = Rule::param(&params, ("nth", 0, "index"));
				let mut result: NodeList = NodeList::with_capacity(DEF_NODES_LEN);
				group_siblings_then_done(
					nodes,
					|total: usize| Some(Nth::get_allowed_indexs(n, index, total)),
					|data: &mut SiblingsNodeData| -> EmptyResult {
						let allow_indexs = data.allow_indexs.as_ref().expect("allow indexs must set");
						if allow_indexs.is_empty() {
							return Ok(());
						}
						let childs = data
							.parent
							.as_ref()
							.expect("parent must set in callback")
							.children()?;
						let finded = handle(&mut data.siblings, &allow_indexs, &childs);
						if !finded.is_empty() {
							result.get_mut_ref().extend(finded);
						}
						Ok(())
					},
				)?;
				Ok(result)
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

// make for ':first-of-type', ':last-of-type'
fn make_first_or_last_of_type(selector: &'static str, is_first: bool) -> RuleDefItem {
	let name = selector;
	// last of type
	RuleDefItem(
		name,
		selector,
		PRIORITY,
		vec![],
		Box::new(move |nodes: &NodeList, _: &RuleMatchedData| -> Result {
			let mut result: NodeList = NodeList::with_capacity(DEF_NODES_LEN);
			group_siblings_then_done(
				nodes,
				|_| None,
				|data: &mut SiblingsNodeData| -> EmptyResult {
					let childs = data
						.parent
						.as_ref()
						.expect("parent must set in callback")
						.children()?;
					let mut names: HashSet<String> = HashSet::with_capacity(5);
					// handle
					fn handle(
						child: &BoxDynNode,
						result: &mut NodeList,
						siblings: &mut Vec<BoxDynNode>,
						names: &mut HashSet<String>,
					) -> bool {
						let name = child.tag_name();
						// not the first type
						if names.get(name).is_some() {
							// continue to next loop, ignore the next process
							return true;
						}
						// the first type of the name tag
						names.insert(String::from(name));
						let mut exclude_indexs: Vec<usize> = Vec::with_capacity(2);
						for (i, node) in siblings.iter().enumerate() {
							let cur_name = node.tag_name();
							if cur_name == name {
								if node.is(child) {
									result.push(node.cloned());
								} else {
									exclude_indexs.push(i);
								}
							}
						}
						if !exclude_indexs.is_empty() {
							// delete the not matched
							retain_by_index(siblings, &exclude_indexs);
						}
						!siblings.is_empty()
					}
					if is_first {
						for child in childs.get_ref() {
							if !handle(child, &mut result, &mut data.siblings, &mut names) {
								break;
							}
						}
					} else {
						data.siblings.reverse();
						for child in childs.get_ref().iter().rev() {
							if !handle(child, &mut result, &mut data.siblings, &mut names) {
								break;
							}
						}
					}
					Ok(())
				},
			)?;
			Ok(result)
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

fn handle_nth_of_type(
	child: &BoxDynNode,
	finded: &mut Vec<BoxDynNode>,
	siblings: &mut Vec<BoxDynNode>,
	allow_indexs: &[usize],
	names: &mut NameCountHashMap,
) -> bool {
	let name = child.tag_name();
	let is_ok_index = if let Some(index) = names.get_mut(name) {
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
	};
	if !is_ok_index {
		return true;
	}
	let mut find_index: Option<usize> = None;
	for (i, node) in siblings.iter().enumerate() {
		let cur_name = node.tag_name();
		if cur_name == name && node.is(child) {
			finded.push(node.cloned());
			find_index = Some(i);
			break;
		}
	}
	if let Some(index) = find_index {
		// delete the not matched
		siblings.remove(index);
	}
	!siblings.is_empty()
}

// make nth of type: `:nth-of-type`, `:nth-last-of-type`
fn make_asc_or_desc_nth_of_type(selector: &'static str, asc: bool) -> RuleDefItem {
	let name = if asc {
		":nth-of-type"
	} else {
		":nth-last-of-type"
	};
	// last of type
	RuleDefItem(
		name,
		selector,
		PRIORITY,
		vec![("nth", 0)],
		Box::new(
			move |nodes: &NodeList, params: &RuleMatchedData| -> Result {
				let mut result: NodeList = NodeList::with_capacity(DEF_NODES_LEN);
				let n = Rule::param(&params, ("nth", 0, "n"));
				let index = Rule::param(&params, ("nth", 0, "index"));
				group_siblings_then_done(
					nodes,
					|total: usize| Some(Nth::get_allowed_indexs(n, index, total)),
					|data: &mut SiblingsNodeData| -> EmptyResult {
						let allow_indexs = data.allow_indexs.as_ref().expect("allow indexs must set");
						// return if allow_indexs is empty
						if allow_indexs.is_empty() {
							return Ok(());
						}
						// childs
						let childs = data
							.parent
							.as_ref()
							.expect("parent must set in callback")
							.children()?;
						let mut names: NameCountHashMap = HashMap::with_capacity(5);
						let mut finded: Vec<BoxDynNode> = Vec::with_capacity(5);
						// loop
						if asc {
							for child in childs.get_ref() {
								if !handle_nth_of_type(
									child,
									&mut finded,
									&mut data.siblings,
									allow_indexs,
									&mut names,
								) {
									break;
								}
							}
						} else {
							data.siblings.reverse();
							for child in childs.get_ref().iter().rev() {
								if !handle_nth_of_type(
									child,
									&mut finded,
									&mut data.siblings,
									allow_indexs,
									&mut names,
								) {
									break;
								}
							}
							finded.reverse();
						}
						if !finded.is_empty() {
							result.get_mut_ref().extend(finded);
						}
						Ok(())
					},
				)?;
				Ok(result)
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
		Box::new(move |nodes: &NodeList, _| -> Result {
			let mut result = NodeList::with_capacity(DEF_NODES_LEN);
			for node in nodes.get_ref() {
				if let Some(parent) = node.parent()? {
					let childs = parent.children()?;
					if childs.length() == 1 {
						result.push(node.cloned());
					}
				}
			}
			Ok(result)
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
		Box::new(move |nodes: &NodeList, _| -> Result {
			let mut result = NodeList::with_capacity(DEF_NODES_LEN);
			group_siblings_then_done(
				nodes,
				|_| None,
				|data: &mut SiblingsNodeData| -> EmptyResult {
					let childs = data
						.parent
						.as_ref()
						.expect("parent must set in callback")
						.children()?;
					let siblings = &mut data.siblings;
					let mut names: HashMap<String, bool> = HashMap::with_capacity(DEF_NODES_LEN);
					for child in childs.get_ref() {
						let name = child.tag_name();
						if let Some(removed_done) = names.get_mut(name) {
							if *removed_done {
								// has removed the not only type
							} else {
								// remove not only type
								let mut repeat_indexs: Vec<usize> = Vec::with_capacity(siblings.len());
								for (index, node) in siblings.iter().enumerate() {
									if node.tag_name() == name {
										repeat_indexs.push(index);
									}
								}
								if !repeat_indexs.is_empty() {
									retain_by_index(siblings, &repeat_indexs);
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
					Ok(())
				},
			)?;
			Ok(result)
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
		Box::new(|nodes: &NodeList, params: &RuleMatchedData| -> Result { Ok(nodes.cloned()) }),
	);
	rules.push(rule.into());
}

/// pseudo selector: `:checkbox`
fn pseudo_alias_checkbox(rules: &mut Vec<RuleItem>) {
	let selector = ":checkbox";
	let name = selector;
	let rule = RuleAliasItem(
		name,
		selector,
		PRIORITY,
		vec![],
		Box::new(|_| "[type=\"checkbox\"]"),
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
	// alias
	pseudo_alias_checkbox(rules);
}

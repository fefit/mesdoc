use std::collections::HashSet;

use crate::{selector::{
	interface::EmptyResult,
	rule::{Rule, RuleDefItem, RuleItem},
}, utils::retain_by_index};
use crate::selector::{
	interface::{BoxDynNode, INodeType, NodeList, Result},
	pattern::Nth,
	rule::RuleMatchedData,
};

const PRIORITY: u32 = 10;
const DEF_NODES_LEN: usize = 5;
/// pseudo selector ":empty"
fn pseudo_empty(rules: &mut Vec<RuleItem>) {
	// empty
	let rule = RuleDefItem(
		":empty",
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


fn get_first_or_last(params: &RuleMatchedData)->&str{
  Rule::param(&params, "first_or_last").expect("first_or_last param must be 'first' or 'last'")
}

/// pseudo selector `:first-child,:last-child`
fn pseudo_first_or_last_child(rules: &mut Vec<RuleItem>) {
	// last_child
	let rule = RuleDefItem(
		":{first_or_last}-child",
		PRIORITY,
		vec![("first_or_last", 0)],
		Box::new(|nodes: &NodeList, params: &RuleMatchedData| -> Result {
      let first_or_last = get_first_or_last(params);
      let is_first = first_or_last == "first";
      let mut result = NodeList::with_capacity(DEF_NODES_LEN);
      let get_index = if is_first{
        |_: usize| 0
      } else{
        |total:usize| total - 1
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
	);
	rules.push(rule.into());
}

/// process
fn process(
	siblings: &mut Vec<BoxDynNode>,
	allow_indexs: &[usize],
	childs: &NodeList,
	result: &mut NodeList,
) {
	// do with the siblings
	if !allow_indexs.is_empty() {
		let child_nodes = childs.get_ref();
		for index in allow_indexs {
			if let Some(child) = child_nodes.get(*index) {
				let mut find_index: Option<usize> = None;
				for (idx, node) in siblings.iter().enumerate() {
					if node.is(child) {
						result.push(node.cloned());
						find_index = Some(idx);
					}
				}
				if let Some(find_index) = find_index {
					// the last one node
					if siblings.len() == 1 {
						break;
					}
					// remove from the siblings queue
					siblings.remove(find_index as usize);
				}
			}
		}
	}
}

//
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
				data.allow_indexs = allow_indexs_fn(total);
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

/// pseudo selector: `:nth-child`
fn pseudo_nth_child(rules: &mut Vec<RuleItem>) {
	let rule = RuleDefItem(
		":nth-child({spaces}{nth}{spaces})",
		PRIORITY,
		vec![("nth", 0)],
		Box::new(|nodes: &NodeList, params: &RuleMatchedData| -> Result {
			let n = Rule::param(&params, ("nth", 0, "n"));
			let index = Rule::param(&params, ("nth", 0, "index"));
			let mut result: NodeList = NodeList::with_capacity(DEF_NODES_LEN);
			group_siblings_then_done(
				nodes,
				|total: usize| Some(Nth::get_allowed_indexs(n, index, total)),
				|data: &mut SiblingsNodeData| -> EmptyResult {
					process(
						&mut data.siblings,
						&data.allow_indexs.as_ref().expect("allow indexs must set"),
						&data
							.parent
							.as_ref()
							.expect("parent must set in callback")
							.children()?,
						&mut result,
					);
					Ok(())
				},
			)?;
			Ok(result)
		}),
	);
	rules.push(rule.into());
}



/// pseudo selector:`:first-of-type,:last-of-type`
fn pseudo_first_or_last_of_type(rules: &mut Vec<RuleItem>) {
	// last of type
	let rule = RuleDefItem(
		":{first_or_last}-of-type",
		PRIORITY,
		vec![("first_or_last", 0)],
		Box::new(|nodes: &NodeList, params: &RuleMatchedData| -> Result {
      let first_or_last = get_first_or_last(params);
      let is_first = first_or_last == "first";
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
          fn handle(data: &mut SiblingsNodeData, result: &mut NodeList, child: &BoxDynNode, names:&mut HashSet<String>){
            let name = child.tag_name();
						// not the first type
						if names.get(name).is_some() {
							return;
						}
						// the first type of the name tag
						names.insert(String::from(name));
						let mut exclude_indexs: Vec<usize> = Vec::with_capacity(2);
						for (i, node) in data.siblings.iter().enumerate() {
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
							retain_by_index(&mut data.siblings, &exclude_indexs);
						}
          }
          if is_first{ 
            for child in childs.get_ref() {
              handle(data, &mut result, child, &mut names);
            }
          } else{
            data.siblings.reverse();
            for child in childs.get_ref().iter().rev(){
              handle(data, &mut result, child, &mut names);
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


pub fn init(rules: &mut Vec<RuleItem>) {
	pseudo_empty(rules);
	pseudo_first_or_last_child(rules);
	pseudo_nth_child(rules);
  pseudo_first_or_last_of_type(rules);
}

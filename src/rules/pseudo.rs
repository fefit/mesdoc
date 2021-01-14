use std::{error::Error, result::Result as StdResult};
use crate::selector::{interface::{BoxDynNode, INodeType, NodeList, Result}, pattern::Nth, rule::RuleMatchedData};
use crate::selector::rule::{ Rule, RuleDefItem, RuleItem};
const PRIORITY: u32 = 10;
fn add_empty(rules: &mut Vec<RuleItem>) {
	// empty
	let rule = RuleDefItem(
		":empty",
		PRIORITY,
		vec![],
		Box::new(|nodes: &NodeList, _| -> Result {
			let mut result = NodeList::new();
			for node in nodes.get_ref() {
        let childs = node.children()?;
        if childs.is_empty(){
          result.push(node.cloned());
        } else {
          let mut only_comments = true;
          for child in childs {
            match child.node_type() {
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
fn add_first_child(rules: &mut Vec<RuleItem>) {
	// first-child
	let rule = RuleDefItem(
		":first-child",
		PRIORITY,
		vec![],
		Box::new(|nodes, _params| {
			let mut result = NodeList::new();
			for node in nodes.get_ref() {
				if node.parent().is_ok() {
					if node.node_type().is_element() && node.index().unwrap() == 0 {
						result.push(node.cloned());
					}
				} else {
					result.push(node.cloned());
				}
			}
			Ok(result)
		}),
	);
	rules.push(rule.into());
}
fn add_last_child(rules: &mut Vec<RuleItem>) {
	// last_child
	let rule = RuleDefItem(
		":last-child",
		PRIORITY,
		vec![],
		Box::new(|nodes, _|-> Result {
			let mut result = NodeList::new();
			for node in nodes.get_ref() {
				if let Some(pnode) = node.parent()? {
					let childs = pnode.children()?;
					let mut total = childs.length();
					while total > 0 {
						total -= 1;
						let cur_node = childs.get(total).unwrap();
						if cur_node.node_type().is_element() {
							if node.is(cur_node) {
								result.push(node.cloned());
							}
							break;
						}
					}
				} else {
					result.push(node.cloned());
				}
			}
			Ok(result)
		}),
	);
	rules.push(rule.into());
}
/// selector:`first-of-type`
fn add_first_of_type(rules: &mut Vec<RuleItem>) {
	// first of type
	let rule = RuleDefItem(
		":first-of-type",
		PRIORITY,
		vec![],
		Box::new(|nodes, _params| Ok(nodes.cloned())),
	);
	rules.push(rule.into());
}
/// process
fn process(siblings: &mut Vec<BoxDynNode>, allow_indexs: &[usize], childs: &NodeList,  result: &mut NodeList){
  // do with the siblings
  if !allow_indexs.is_empty(){
    let child_nodes = childs.get_ref();
    for index in allow_indexs{
      if let Some(child) = child_nodes.get(*index){
        let mut find_index: Option<usize> = None;
        for (idx, node) in siblings.iter().enumerate(){
          if node.is(child){
            result.push(node.cloned());
            find_index = Some(idx);
          }
        }
        if let Some(find_index) =  find_index{
          // the last one node
          if siblings.len() == 1{
            break;
          }
          // remove from the siblings queue
          siblings.remove(find_index as usize);
        }
      }
    }
  }
}

/// selector: `nth-child`
fn add_nth_child(rules: &mut Vec<RuleItem>){
  let rule = RuleDefItem(
    ":nth-child({spaces}{nth}{spaces})",
    PRIORITY, 
    vec![("nth", 0)],
    Box::new(|nodes: &NodeList, params: &RuleMatchedData| -> Result {
      let n = Rule::param(&params, ("nth", 0, "n"));
      let index = Rule::param(&params, ("nth", 0, "index"));
      let mut result: NodeList = NodeList::new();
      let total = nodes.length();
      let mut siblings: Vec<BoxDynNode> = Vec::with_capacity(total);
      let mut allow_indexs: Vec<usize> = Vec::with_capacity(5);
      let mut prev_parent: Option<BoxDynNode> = None;
      for (i, node) in nodes.get_ref().iter().enumerate(){
        if let Some(parent) = node.parent()?{
          let mut in_next_group  = false;
          let mut is_first = false;
          let is_last= i == total - 1;
          if let Some(prev_parent) = &prev_parent{
            if parent.is(&prev_parent){
              // sibling node, just add
              siblings.push(node.cloned());
            } else {
              // not sibling
              in_next_group = true;
            }
          } else {
            is_first = true;
          }
          // when meet next group siblings
          if in_next_group {
            process(&mut siblings, &allow_indexs, &prev_parent.as_ref().expect("prev parent must not empty").children()?, &mut result);
          }
          // when is first or in next group
          if is_first || in_next_group{
            // init the siblings, allow_index, prev_parent
            siblings = vec![node.cloned()];
            allow_indexs = Nth::get_allowed_indexs(n, index, parent.children()?.length());
            prev_parent = Some(parent.cloned());
          }
          // when is last
          if is_last{
            process(&mut siblings, &allow_indexs, &parent.children()?, &mut result);
            break;
          }
        }
      }
      Ok(result)
    })
  );
  rules.push(rule.into());
}


pub fn init(rules: &mut Vec<RuleItem>) {
	add_empty(rules);
	add_first_child(rules);
	add_last_child(rules);
  add_first_of_type(rules);
  add_nth_child(rules);
}

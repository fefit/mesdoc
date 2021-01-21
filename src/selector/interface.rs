use crate::utils::{get_class_list, retain_by_index, to_static_str};
use std::{any::Any, mem::swap};
use std::{result::Result as StdResult, usize};

use super::{Combinator, QueryProcess, Selector, SelectorSegment};

pub type Result<'a> = StdResult<NodeList<'a>, &'static str>;
pub type MaybeResult<'a> = StdResult<Option<BoxDynNode<'a>>, &'static str>;
pub type MaybeDocResult = StdResult<Option<Box<dyn IDocumentTrait>>, &'static str>;
pub type EmptyResult = StdResult<(), &'static str>;
pub type BoxDynNode<'a> = Box<dyn INodeTrait + 'a>;
#[derive(Debug)]
pub enum IAttrValue {
	Value(String, Option<char>),
	True,
}
#[derive(Debug, PartialEq, Eq)]
pub enum InsertPosition {
	BeforeBegin,
	AfterBegin,
	BeforeEnd,
	AfterEnd,
}

impl InsertPosition {
	pub fn action(&self) -> &'static str {
		use InsertPosition::*;
		match self {
			BeforeBegin => "insert before",
			AfterBegin => "prepend",
			BeforeEnd => "append",
			AfterEnd => "insert after",
		}
	}
}

#[derive(Debug)]
pub enum INodeType {
	Comment,
	Document,
	Element,
	Text,
	HTMLDOCTYPE,
	XMLCDATA,
	AbstractRoot,
	Other,
}

impl INodeType {
	pub fn is_element(&self) -> bool {
		matches!(self, INodeType::Element)
	}
}
pub trait IDocumentTrait {
	fn get_element_by_id<'b>(&self, id: &str) -> Option<BoxDynNode<'b>>;
}
pub trait INodeTrait {
	fn to_node(self: Box<Self>) -> Box<dyn Any>;
	// clone a node
	fn cloned<'b>(&self) -> BoxDynNode<'b>;
	// tag name
	fn tag_name(&self) -> &str;
	// get node type
	fn node_type(&self) -> INodeType;
	// get node index
	fn index(&self) -> Option<usize> {
		let parent = self.parent();
		if let Ok(Some(childs)) = parent {
			let childs = childs.children().unwrap();
			let mut index = 0;
			for node in childs {
				if node.node_type().is_element() {
					if self.is(&node) {
						return Some(index);
					}
					index += 1;
				}
			}
		}
		None
	}
	// find parents
	fn parent<'b>(&self) -> MaybeResult<'b>;
	// childs
	fn child_nodes<'b>(&self) -> Result<'b>;
	fn children<'b>(&self) -> Result<'b> {
		let child_nodes = self.child_nodes()?;
		let mut result = NodeList::with_capacity(child_nodes.length());
		for node in child_nodes.get_ref() {
			if let INodeType::Element = node.node_type() {
				result.push(node.cloned());
			}
		}
		Ok(result)
	}
	// get all childrens
	fn childrens<'b>(&self) -> Result<'b> {
		let childs = self.children()?;
		let count = childs.length();
		if count > 0 {
			let mut result = NodeList::with_capacity(5);
			let all_nodes = result.get_mut_ref();
			for c in childs.get_ref() {
				all_nodes.push(c.cloned());
				all_nodes.extend(c.childrens()?);
			}
			return Ok(result);
		}
		Ok(NodeList::new())
	}
	// next sibling
	fn next_sibling<'b>(&self) -> MaybeResult<'b> {
		let parent = self.parent()?;
		if let Some(p) = parent {
			let childs = p.children()?;
			let mut finded = false;
			for c in childs {
				if finded {
					return Ok(Some(c.cloned()));
				}
				if self.is(&c) {
					finded = true;
				}
			}
		}
		Ok(None)
	}
	// next siblings
	fn next_siblings<'b>(&self) -> Result<'b> {
		let parent = self.parent()?;
		let mut result = NodeList::with_capacity(2);
		if let Some(p) = parent {
			let childs = p.children()?;
			let mut finded = false;
			for c in childs {
				if finded {
					result.push(c.cloned());
				}
				if self.is(&c) {
					finded = true;
				}
			}
		}
		Ok(result)
	}
	// prev
	fn previous_sibling<'b>(&self) -> MaybeResult<'b> {
		let parent = self.parent()?;
		if let Some(p) = parent {
			let childs = p.children()?;
			let mut prev: Option<BoxDynNode> = None;
			for c in childs {
				if self.is(&c) {
					return Ok(prev.map(|n| n.cloned()));
				} else {
					prev = Some(c);
				}
			}
		}
		Ok(None)
	}
	// next siblings
	fn previous_siblings<'b>(&self) -> Result<'b> {
		let parent = self.parent()?;
		let mut result = NodeList::with_capacity(2);
		if let Some(p) = parent {
			let childs = p.children()?;
			for c in childs {
				if self.is(&c) {
					break;
				}
				result.push(c.cloned());
			}
		}
		Ok(result)
	}
	// siblings
	fn siblings<'b>(&self) -> Result<'b> {
		let parent = self.parent()?;
		let mut result = NodeList::with_capacity(2);
		if let Some(p) = parent {
			let childs = p.children()?;
			for c in childs {
				if self.is(&c) {
					continue;
				}
				result.push(c.cloned());
			}
		}
		Ok(result)
	}
	// attribute
	fn get_attribute(&self, name: &str) -> Option<IAttrValue>;
	fn set_attribute(&mut self, name: &str, value: Option<&str>);
	fn remove_attribute(&mut self, name: &str);
	fn has_attribute(&self, name: &str) -> bool {
		self.get_attribute(name).is_some()
	}
	// html
	fn html(&self) -> &str {
		self.inner_html()
	}
	fn inner_html(&self) -> &str;
	fn outer_html(&self) -> &str;
	// text
	fn text_content(&self) -> &str;
	fn text(&self) -> &str {
		self.text_content()
	}
	// append child, insert before, remove child
	fn insert_adjacent(&mut self, position: &InsertPosition, node: &BoxDynNode);
	fn remove_child(&mut self, node: BoxDynNode);
	// check if two node are the same
	fn uuid(&self) -> Option<&str>;
	fn is(&self, node: &BoxDynNode) -> bool {
		if let Some(uuid) = self.uuid() {
			if let Some(o_uuid) = node.uuid() {
				return uuid == o_uuid;
			}
		}
		false
	}
	// owner document
	fn owner_document(&self) -> MaybeDocResult;
}

#[derive(Debug, PartialEq, Eq)]
enum FilterType {
	Filter,
	Not,
	Is,
}
#[derive(Default)]
pub struct NodeList<'a> {
	nodes: Vec<BoxDynNode<'a>>,
}

impl<'a> NodeList<'a> {
	// crate only methods
	pub(crate) fn new() -> Self {
		Default::default()
	}
	pub(crate) fn with_nodes(nodes: Vec<BoxDynNode<'a>>) -> Self {
		NodeList { nodes }
	}
	pub(crate) fn get_ref(&self) -> &Vec<BoxDynNode<'a>> {
		&self.nodes
	}
	pub(crate) fn get_mut_ref(&mut self) -> &mut Vec<BoxDynNode<'a>> {
		&mut self.nodes
	}
	pub(crate) fn push(&mut self, node: BoxDynNode<'a>) {
		self.get_mut_ref().push(node);
	}
	pub(crate) fn with_capacity(size: usize) -> Self {
		NodeList {
			nodes: Vec::with_capacity(size),
		}
	}
	pub(crate) fn get(&self, index: usize) -> Option<&BoxDynNode<'a>> {
		self.get_ref().get(index)
	}

	fn get_out(&mut self, index: usize) -> BoxDynNode<'a> {
		self.nodes.remove(index)
	}
	/// pub fn `length`
	pub fn length(&self) -> usize {
		self.nodes.len()
	}
	/// pub fn `is_empty`
	pub fn is_empty(&self) -> bool {
		self.length() == 0
	}
	// for all combinator selectors
	fn select_with_comb<'b>(&self, selector: &str, comb: Combinator) -> Result<'b> {
		if selector.is_empty() {
			let segment = Selector::make_comb_all(comb);
			let selector = Selector::from_segment(segment);
			return self.find_selector(selector);
		}
		let mut selector: Selector = selector.into();
		selector.head_combinator(comb);
		self.find_selector(selector)
	}
	// prev
	pub fn prev<'b>(&self, selector: &str) -> Result<'b> {
		self.select_with_comb(selector, Combinator::Prev)
	}
	// prev_all
	pub fn prev_all<'b>(&self, selector: &str) -> Result<'b> {
		self.select_with_comb(selector, Combinator::PrevAll)
	}
	// next
	pub fn next<'b>(&self, selector: &str) -> Result<'b> {
		self.select_with_comb(selector, Combinator::Next)
	}
	// next_all
	pub fn next_all<'b>(&self, selector: &str) -> Result<'b> {
		self.select_with_comb(selector, Combinator::NextAll)
	}
	// siblings
	pub fn siblings<'b>(&self, selector: &str) -> Result<'b> {
		self.select_with_comb(selector, Combinator::Siblings)
	}
	// children
	pub fn children<'b>(&self, selector: &str) -> Result<'b> {
		self.select_with_comb(selector, Combinator::Children)
	}
	// parent
	pub fn parent<'b>(&self, selector: &str) -> Result<'b> {
		self.select_with_comb(selector, Combinator::Parent)
	}
	// parents
	pub fn parents<'b>(&self, selector: &str) -> Result<'b> {
		self.select_with_comb(selector, Combinator::ParentAll)
	}
	// for `find` and `select_with_comb`
	fn find_selector<'b>(&self, selector: Selector) -> Result<'b> {
		let process = selector.process;
		let mut result = NodeList::with_capacity(5);
		for p in process {
			let QueryProcess {
				should_in,
				mut query,
			} = p;
			let first = &mut query[0];
			let mut lookup_comb = first[0].2;
			let mut group: NodeList;
			if let Some(mut lookup) = should_in {
				first[0].2 = Combinator::ChildrenAll;
				group = NodeList::with_capacity(5);
				// get finded
				let finded = NodeList::select(self, first)?;
				if !finded.is_empty() {
					let firsts = NodeList::select(self, &lookup[0])?;
					if !firsts.is_empty() {
						let lookup_rules = if lookup.len() > 1 {
							for rule in &mut lookup[1..] {
								swap(&mut rule[0].2, &mut lookup_comb);
							}
							Some(&lookup[1..])
						} else {
							None
						};
						// remove the first
						query.remove(0);
						// check if the previous node and the current node are siblings.
						let mut prev_node: Option<&BoxDynNode> = None;
						let mut is_find = false;
						for node in finded.get_ref() {
							if prev_node.is_some() && NodeList::is_sibling(node, prev_node.unwrap()) {
								match lookup_comb {
									Combinator::Next => {
										if is_find {
											// do nothing, because has finded the only sibling node matched.
											continue;
										}
										// if not find, should detect the current node
									}
									Combinator::NextAll => {
										if is_find {
											group.push(node.cloned());
											continue;
										}
										// if not find, should detect the node
									}
									_ => {
										// do the same thing as `prev_node`
										// if `is_find` is true, then add the node, otherwise it's not matched too.
										// keep the `is_find` value
										if is_find {
											group.push(node.cloned());
										}
										continue;
									}
								};
							}
							// check if the node is in firsts
							if firsts.contains_node(node, &lookup_comb.reverse(), lookup_rules) {
								group.push(node.cloned());
								is_find = true;
							} else {
								is_find = false;
							}
							// set the prev node
							prev_node = Some(node);
						}
					}
				}
			} else {
				group = self.cloned();
			}
			let mut is_empty = false;
			if !group.is_empty() && !query.is_empty() {
				for rules in query {
					group = NodeList::select(&group, &rules)?;
					if group.is_empty() {
						is_empty = true;
						break;
					}
				}
			}
			if !is_empty {
				result.get_mut_ref().extend(group);
			}
		}
		Ok(result)
	}
	// `find`
	pub fn find<'b>(&self, selector: &str) -> Result<'b> {
		let selector: Selector = selector.into();
		self.find_selector(selector)
	}
	// filter_by:
	//          |   `loop_group:rule groups      |     'loop_node: node list
	// Filter   |     match one rule item        |      should loop all nodes
	// Not      |        all not matched         |      should loop all nodes
	// Is       |           all matched          |  once one node is not matched, break the loop
	fn filter_by<'b>(&self, selector: &str, filter_type: FilterType) -> Result<'b> {
		let selector: Selector = Selector::from_str(selector, false);
		let groups_num = selector.process.len();
		let mut result = NodeList::with_capacity(self.get_ref().len());
		let is_not = filter_type == FilterType::Not;
		for node in self.get_ref() {
			let mut ok_nums = 0;
			'loop_group: for process in &selector.process {
				let QueryProcess { query, .. } = process;
				let mut node_list = NodeList::with_nodes(vec![node.cloned()]);
				let mut comb = Combinator::Chain;
				// loop cur group's rule
				for rules in query.iter().rev() {
					let first_rule = &rules[0];
					for (index, rule) in rules.iter().enumerate() {
						let find_list: NodeList;
						if index == 0 {
							find_list = NodeList::select_by_rule(&node_list, rule, Some(comb))?.unique();
						} else {
							find_list = NodeList::select_by_rule(&node_list, rule, None)?;
						}
						if first_rule.0.in_cache {
							// the node list is in cache
							let total = find_list.length();
							if total > 0 {
								let mut last_list = NodeList::with_capacity(total);
								let reverse_comb = comb.reverse();
								for node in find_list.get_ref() {
									if node_list.contains_node(node, &reverse_comb, None) {
										last_list.push(node.cloned());
									}
								}
								node_list = last_list;
							} else {
								node_list = find_list;
							}
						} else {
							node_list = find_list;
						}
						if node_list.is_empty() {
							break;
						}
					}
					// change the comb into cur first rule's reverse comb.
					comb = first_rule.2.reverse();
				}
				if node_list.is_empty() {
					if is_not {
						// if is `not`, then the node is not in cur group selector.
						ok_nums += 1;
					}
				} else {
					// match one of the group item
					match filter_type {
						FilterType::Filter => {
							result.push(node.cloned());
							break 'loop_group;
						}
						FilterType::Is => {
							ok_nums += 1;
						}
						FilterType::Not => {}
					}
				}
			}
			// check the node loop
			match filter_type {
				FilterType::Not => {
					if ok_nums == groups_num {
						result.push(node.cloned());
					}
				}
				FilterType::Is => {
					if ok_nums == groups_num {
						result.push(node.cloned());
					} else {
						// break the loop for node
						break;
					}
				}
				_ => {}
			}
		}
		Ok(result)
	}
	// filter
	pub fn filter<'b>(&self, selector: &str) -> Result<'b> {
		self.filter_by(selector, FilterType::Filter)
	}
	// is
	pub fn is(&self, selector: &str) -> StdResult<bool, &'static str> {
		if self.is_empty() {
			return Ok(false);
		}
		Ok(self.filter_by(selector, FilterType::Is)?.length() == self.length())
	}
	// not
	pub fn not<'b>(&self, selector: &str) -> Result<'b> {
		println!("selector:{}, length: {}", selector, self.length());
		self.filter_by(selector, FilterType::Not)
	}
	// unique the nodes
	fn unique<'b>(&self) -> NodeList<'b> {
		let total = self.length();
		let mut result = NodeList::with_capacity(total);
		for node in self.get_ref() {
			let is_exists = {
				let mut flag = false;
				for cur in result.get_ref().iter().rev() {
					if cur.is(node) {
						flag = true;
						break;
					}
				}
				flag
			};
			if is_exists {
				continue;
			}
			result.push(node.cloned());
		}
		result
	}
	// select one rule
	// the rule must not in cache
	fn select_by_rule<'b>(
		node_list: &'b NodeList<'a>,
		rule_item: &'b SelectorSegment,
		comb: Option<Combinator>,
	) -> Result<'a> {
		let cur_comb = comb.unwrap_or(rule_item.2);
		let (rule, matched, ..) = rule_item;
		let mut result = NodeList::with_capacity(5);
		use Combinator::*;
		match cur_comb {
			ChildrenAll => {
				// depth first search, keep the appear order
				for node in node_list.get_ref() {
					// get children
					let childs = node.children()?;
					if !childs.is_empty() {
						// apply rule
						for child in childs.get_ref() {
							let mut cur = NodeList::with_nodes(vec![child.cloned()]);
							let is_matched = !rule.apply(&cur, matched)?.is_empty();
							let sub_childs = child.children()?;
							if !sub_childs.is_empty() {
								// add has finded
								if is_matched {
									result.get_mut_ref().push(child.cloned());
								}
								// search sub child
								let sub_matched = NodeList::select_by_rule(&cur, rule_item, comb)?;
								if !sub_matched.is_empty() {
									result.get_mut_ref().extend(sub_matched);
								}
							} else if is_matched {
								// move the matched node out from cur
								result.get_mut_ref().push(cur.get_out(0));
							}
						}
					}
				}
			}
			Children => {
				for node in node_list.get_ref() {
					let childs = node.children()?;
					let match_childs = rule.apply(&childs, matched)?;
					if !match_childs.is_empty() {
						result.get_mut_ref().extend(match_childs);
					}
				}
			}
			Parent => {
				for node in node_list.get_ref() {
					if let Some(pnode) = node.parent()? {
						let cur_pnode = NodeList::with_nodes(vec![pnode.cloned()]);
						let parent = rule.apply(&cur_pnode, matched)?;
						if !parent.is_empty() {
							result.get_mut_ref().extend(parent);
						}
					}
				}
			}
			ParentAll => {
				let mut ancestors = NodeList::with_capacity(node_list.length());
				for node in node_list.get_ref() {
					if let Some(pnode) = node.parent()? {
						let cur_pnode = NodeList::with_nodes(vec![pnode.cloned()]);
						let parent = rule.apply(&cur_pnode, matched)?;
						if !parent.is_empty() {
							result.get_mut_ref().extend(parent);
						}
						if let Some(ancestor) = pnode.parent()? {
							ancestors.push(ancestor.cloned());
						}
					}
				}
				if !ancestors.is_empty() {
					result
						.get_mut_ref()
						.extend(NodeList::select_by_rule(&ancestors, rule_item, comb)?);
				}
			}
			NextAll => {
				for node in node_list.get_ref() {
					let nexts = node.next_siblings()?;
					let matched_nexts = rule.apply(&nexts, matched)?;
					if !matched_nexts.is_empty() {
						result.get_mut_ref().extend(matched_nexts);
					}
				}
			}
			Next => {
				let mut nexts = NodeList::with_capacity(node_list.length());
				for node in node_list.get_ref() {
					if let Some(next) = node.next_sibling()? {
						nexts.push(next.cloned());
					}
				}
				if !nexts.is_empty() {
					result = rule.apply(&nexts, matched)?;
				}
			}
			PrevAll => {
				for node in node_list.get_ref() {
					let nexts = node.previous_siblings()?;
					result.get_mut_ref().extend(rule.apply(&nexts, matched)?);
				}
			}
			Prev => {
				let mut prevs = NodeList::with_capacity(node_list.length());
				for node in node_list.get_ref() {
					if let Some(next) = node.previous_sibling()? {
						prevs.push(next.cloned());
					}
				}
				if !prevs.is_empty() {
					result = rule.apply(&prevs, matched)?;
				}
			}
			Siblings => {
				for node in node_list.get_ref() {
					let siblings = node.siblings()?;
					result.get_mut_ref().extend(rule.apply(&siblings, matched)?);
				}
			}
			Chain => {
				result = rule.apply(&node_list, matched)?;
			}
		};
		Ok(result)
	}
	// select node by rules
	fn select<'b>(node_list: &'b NodeList<'a>, rules: &'b [SelectorSegment]) -> Result<'a> {
		let mut node_list = node_list.cloned();
		for rule_item in rules.iter() {
			let (rule, matched, comb) = rule_item;
			let mut cur_result = NodeList::with_capacity(5);
			if rule.in_cache {
				// in cache
				let finded = rule.apply(&node_list, matched)?;
				if !finded.is_empty() {
					let reverse_comb = comb.reverse();
					for node in finded.get_ref() {
						if node_list.contains_node(node, &reverse_comb, None) {
							cur_result.push(node.cloned());
						}
					}
				}
			} else {
				cur_result = NodeList::select_by_rule(&node_list, rule_item, None)?;
			}
			node_list = cur_result.unique();
			if node_list.is_empty() {
				break;
			}
		}
		Ok(node_list.unique())
	}
	// cloned
	pub fn cloned<'b>(&'a self) -> NodeList<'b> {
		let mut result = NodeList::with_capacity(self.length());
		for node in &self.nodes {
			result.push(node.cloned());
		}
		result
	}
	// `contains_node`
	pub(crate) fn contains_node<'b>(
		&self,
		node: &'b BoxDynNode,
		comb: &Combinator,
		lookup: Option<&'b [Vec<SelectorSegment>]>,
	) -> bool {
		let mut node_list = NodeList::with_nodes(vec![node.cloned()]);
		if let Some(lookup) = lookup {
			for rules in lookup.iter().rev() {
				if let Ok(finded) = NodeList::select(&node_list, rules) {
					node_list = finded;
				} else {
					node_list = NodeList::new();
				}
				if node_list.is_empty() {
					return false;
				}
			}
		}
		use Combinator::*;
		match comb {
			Parent => {
				for node in node_list.get_ref() {
					if let Some(parent) = node.parent().unwrap_or(None) {
						if self.includes(&parent) {
							return true;
						}
					}
				}
			}
			ParentAll => {
				for node in node_list.get_ref() {
					if let Some(parent) = node.parent().unwrap_or(None) {
						if self.includes(&parent) {
							return true;
						}
						if let Some(ancestor) = parent.parent().unwrap_or(None) {
							if self.includes(&ancestor) {
								return true;
							}
							if self.contains_node(&ancestor, comb, None) {
								return true;
							}
						}
					}
				}
			}
			Prev => {
				for node in node_list.get_ref() {
					if let Some(prev) = node.previous_sibling().unwrap_or(None) {
						if self.includes(&prev) {
							return true;
						}
					}
				}
			}
			PrevAll => {
				for node in node_list.get_ref() {
					if let Ok(prevs) = node.previous_siblings() {
						for prev in prevs.get_ref() {
							if self.includes(prev) {
								return true;
							}
						}
					}
				}
			}
			Chain => {
				for node in node_list.get_ref() {
					if self.includes(node) {
						return true;
					}
				}
			}
			_ => panic!("Unsupported lookup combinator:{:?}", comb),
		};
		false
	}
	/// check if the node list contains some node
	fn includes(&self, node: &BoxDynNode) -> bool {
		self.get_ref().iter().any(|n| node.is(n))
	}
	/// check if two nodes are siblings.
	fn is_sibling(cur: &BoxDynNode, other: &BoxDynNode) -> bool {
		// just check if they have same parent.
		if let Ok(Some(parent)) = cur.parent() {
			if let Ok(Some(other_parent)) = other.parent() {
				return parent.is(&other_parent);
			}
		}
		false
	}
	/// pub fn `text`
	pub fn text(&self) -> &str {
		let mut result = String::with_capacity(50);
		for node in self.get_ref() {
			result.push_str(node.text_content());
		}
		to_static_str(result)
	}
	/// pub fn `html`
	pub fn html(&self) -> &str {
		if let Some(node) = self.get(0) {
			return node.inner_html();
		}
		""
	}
	/// pub fn `outer_html`
	pub fn outer_html(&self) -> &str {
		if let Some(node) = self.get(0) {
			return node.outer_html();
		}
		""
	}
	/// pub fn `attr`
	pub fn attr(&self, attr_name: &str) -> Option<IAttrValue> {
		if let Some(node) = self.get(0) {
			return node.get_attribute(attr_name);
		}
		None
	}
	/// pub fn `set_attr`
	pub fn set_attr(&mut self, attr_name: &str, value: Option<&str>) {
		for node in self.get_mut_ref() {
			node.set_attribute(attr_name, value);
		}
	}
	/// pub fn `remove_attr`
	pub fn remove_attr(&mut self, attr_name: &str) {
		for node in self.get_mut_ref() {
			node.remove_attribute(attr_name);
		}
	}
	/// pub fn `add_class`
	pub fn add_class(&mut self, class_name: &str) {
		const ATTR_CLASS: &str = "class";
		let class_name = class_name.trim();
		let class_list = get_class_list(class_name);
		for node in self.get_mut_ref() {
			let class_value = node.get_attribute(ATTR_CLASS);
			if let Some(IAttrValue::Value(cls, _)) = class_value {
				let mut orig_class_list = get_class_list(&cls);
				for class_name in &class_list {
					if !orig_class_list.contains(class_name) {
						orig_class_list.push(class_name);
					}
				}
				node.set_attribute(ATTR_CLASS, Some(orig_class_list.join(" ").as_str()));
				continue;
			}
			node.set_attribute(ATTR_CLASS, Some(class_name));
		}
	}
	/// pub fn `remove_class`
	pub fn remove_class(&mut self, class_name: &str) {
		const ATTR_CLASS: &str = "class";
		let class_list = get_class_list(class_name);
		for node in self.get_mut_ref() {
			let class_value = node.get_attribute(ATTR_CLASS);
			if let Some(IAttrValue::Value(cls, _)) = class_value {
				let mut orig_class_list = get_class_list(&cls);
				let mut removed_indexs: Vec<usize> = Vec::with_capacity(class_list.len());
				for class_name in &class_list {
					if let Some(index) = orig_class_list.iter().position(|name| name == class_name) {
						removed_indexs.push(index);
					}
				}
				if !removed_indexs.is_empty() {
					retain_by_index(&mut orig_class_list, &removed_indexs);
					node.set_attribute(ATTR_CLASS, Some(orig_class_list.join(" ").as_str()));
				}
			}
		}
	}
	/// pub fn `toggle_class`
	pub fn toggle_class(&mut self, class_name: &str) {
		const ATTR_CLASS: &str = "class";
		let class_name = class_name.trim();
		let class_list = get_class_list(class_name);
		let total = class_list.len();
		for node in self.get_mut_ref() {
			let class_value = node.get_attribute(ATTR_CLASS);
			if let Some(IAttrValue::Value(cls, _)) = class_value {
				let mut orig_class_list = get_class_list(&cls);
				let mut removed_indexs: Vec<usize> = Vec::with_capacity(total);
				let mut added_class_list: Vec<&str> = Vec::with_capacity(total);
				for class_name in &class_list {
					if let Some(index) = orig_class_list.iter().position(|name| name == class_name) {
						removed_indexs.push(index);
					} else {
						added_class_list.push(class_name);
					}
				}
				let mut need_set = false;
				if !removed_indexs.is_empty() {
					retain_by_index(&mut orig_class_list, &removed_indexs);
					need_set = true;
				}
				if !added_class_list.is_empty() {
					orig_class_list.extend(added_class_list);
					need_set = true;
				}
				if need_set {
					node.set_attribute(ATTR_CLASS, Some(orig_class_list.join(" ").as_str()));
				}
				continue;
			}
			node.set_attribute(ATTR_CLASS, Some(class_name));
		}
	}
	// -----------------DOM API--------------
	/// pub fn `remove`
	pub fn remove(self) {
		for node in self.into_iter() {
			if let Some(parent) = node.parent().unwrap_or(None).as_mut() {
				parent.remove_child(node);
			}
		}
	}
	// `insert`
	fn insert(&mut self, dest: &NodeList, position: &InsertPosition) {
		for node in self.get_mut_ref() {
			for inserted in dest.get_ref().iter().rev() {
				node.insert_adjacent(position, inserted);
			}
		}
	}
	/// pub fn `append`
	pub fn append(&mut self, node_list: &NodeList) {
		self.insert(node_list, &InsertPosition::BeforeEnd);
	}
	/// pub fn `append_to`
	pub fn append_to(&self, node_list: &mut NodeList) {
		node_list.append(self);
	}
	/// pub fn `prepend`
	pub fn prepend(&mut self, node_list: &NodeList) {
		self.insert(node_list, &InsertPosition::AfterBegin);
	}
	/// pub fn `prepend_to`
	pub fn prepend_to(&self, node_list: &mut NodeList) {
		node_list.prepend(self);
	}
	/// pub fn `insert_before`
	pub fn insert_before(&mut self, node_list: &NodeList) {
		self.insert(node_list, &InsertPosition::BeforeBegin);
	}
	/// pub fn `before`
	pub fn before(&self, node_list: &mut NodeList) {
		node_list.insert_before(self);
	}
	/// pub fn `insert_after`
	pub fn insert_after(&mut self, node_list: &NodeList) {
		self.insert(node_list, &InsertPosition::AfterEnd);
	}
	/// pub fn `before`
	pub fn after(&self, node_list: &mut NodeList) {
		node_list.insert_after(self);
	}
}

impl<'a> IntoIterator for NodeList<'a> {
	type Item = BoxDynNode<'a>;
	type IntoIter = Box<dyn Iterator<Item = Self::Item> + 'a>;
	fn into_iter(self) -> Self::IntoIter {
		Box::new(self.nodes.into_iter())
	}
}

impl<'a> From<Vec<BoxDynNode<'a>>> for NodeList<'a> {
	fn from(nodes: Vec<BoxDynNode<'a>>) -> Self {
		NodeList { nodes }
	}
}

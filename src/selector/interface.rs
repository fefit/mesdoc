use std::mem::swap;
use std::result::Result as StdResult;

use crate::utils::to_static_str;

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
pub enum INodeType {
	Element,
	Text,
	Comment,
	Spaces,
	Document,
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
		let mut result = self.children()?.cloned();
		let count = result.length();
		if count > 0 {
			let mut descendants = NodeList::with_capacity(5);
			let all_nodes = descendants.get_mut_ref();
			for c in &result.nodes {
				all_nodes.extend(c.childrens()?);
			}
			result.get_mut_ref().extend(descendants);
		}
		Ok(result)
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
	fn siblings(&self) -> Result {
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
	// // node
	// fn append_child(&mut self);
	// fn remove_child(&mut self, node: BoxDynNode);
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

#[derive(Default)]
pub struct NodeList<'a> {
	nodes: Vec<BoxDynNode<'a>>,
}

impl<'a> NodeList<'a> {
	pub fn new() -> Self {
		Default::default()
	}
	pub fn with_nodes(nodes: Vec<BoxDynNode<'a>>) -> Self {
		NodeList { nodes }
	}
	pub fn get_ref(&self) -> &Vec<BoxDynNode<'a>> {
		&self.nodes
	}
	pub fn get_mut_ref(&mut self) -> &mut Vec<BoxDynNode<'a>> {
		&mut self.nodes
	}
	pub(crate) fn push(&mut self, node: BoxDynNode<'a>) {
		self.get_mut_ref().push(node);
	}
	pub fn with_capacity(size: usize) -> Self {
		NodeList {
			nodes: Vec::with_capacity(size),
		}
	}
	pub(crate) fn get(&self, index: usize) -> Option<&BoxDynNode<'a>> {
		self.get_ref().get(index)
	}
	pub fn length(&self) -> usize {
		self.nodes.len()
	}
	pub fn is_empty(&self) -> bool {
		self.length() == 0
	}
	// filter some rule
	pub fn find<'b>(&self, selector: &str) -> Result<'b> {
		let selector: Selector = selector.into();
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
				if finded.length() > 0 {
					let firsts = NodeList::select(self, &lookup[0])?;
					if firsts.length() > 0 {
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
			if group.length() > 0 && !query.is_empty() {
				for rules in query {
					group = NodeList::select(&group, &rules)?;
					if group.length() == 0 {
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
	// unique the nodes
	fn unique<'b>(&self) -> NodeList<'b> {
		let total = self.length();
		let mut result = NodeList::with_capacity(total);
		for node in self.get_ref() {
			let is_exists = {
				let mut flag = false;
				for cur in result.get_ref() {
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
	// select node by rules
	fn select<'b>(node_list: &'b NodeList<'a>, rules: &'b [SelectorSegment]) -> Result<'a> {
		let mut node_list = node_list.cloned();
		use Combinator::*;
		for (index, r) in rules.iter().enumerate() {
			let (rule, matched, comb) = r;
			let cur_rule = &rules[index..index + 1];
			let mut cur_result = NodeList::with_capacity(5);
			if rule.in_cache {
				// in cache
				let finded = rule.apply(&node_list, matched)?;
				if finded.length() > 0 {
					for node in finded.get_ref() {
						if node_list.contains_node(node, &comb.reverse(), None) {
							cur_result.push(node.cloned());
						}
					}
				}
			} else {
				match comb {
					ChildrenAll => {
						// depth first search, keep the appear order
						for node in node_list.get_ref() {
							// get children
							let childs = node.children()?;
							if childs.length() > 0 {
								// apply rule
								let match_childs = rule.apply(&childs, matched)?;
								// merge to result
								if match_childs.length() > 0 {
									cur_result.get_mut_ref().extend(match_childs);
								}
								let sub_childs = NodeList::select(&childs, cur_rule)?;
								if !sub_childs.is_empty() {
									cur_result.get_mut_ref().extend(sub_childs);
								}
							}
						}
					}
					Combinator::Children => {
						for node in node_list.get_ref() {
							let childs = node.children()?;
							let match_childs = rule.apply(&childs, matched)?;
							if match_childs.length() > 0 {
								cur_result.get_mut_ref().extend(match_childs);
							}
						}
					}
					Combinator::Parent => {
						for node in node_list.get_ref() {
							if let Some(pnode) = node.parent()? {
								let cur_pnode = NodeList::with_nodes(vec![pnode.cloned()]);
								let parent = rule.apply(&cur_pnode, matched)?;
								if parent.length() > 0 {
									cur_result.get_mut_ref().extend(parent);
								}
							}
						}
					}
					Combinator::ParentAll => {
						let mut ancestors = NodeList::with_capacity(node_list.length());
						for node in node_list.get_ref() {
							if let Some(pnode) = node.parent()? {
								let cur_pnode = NodeList::with_nodes(vec![pnode.cloned()]);
								let parent = rule.apply(&cur_pnode, matched)?;
								if parent.length() > 0 {
									cur_result.get_mut_ref().extend(parent);
								}
								if let Some(ancestor) = pnode.parent()? {
									ancestors.push(ancestor.cloned());
								}
							}
						}
						if ancestors.length() > 0 {
							cur_result
								.get_mut_ref()
								.extend(NodeList::select(&ancestors, cur_rule)?);
						}
					}
					Combinator::NextAll => {
						for node in node_list.get_ref() {
							let nexts = node.next_siblings()?;
							let matched_nexts = rule.apply(&nexts, matched)?;
							if matched_nexts.length() > 0 {
								cur_result.get_mut_ref().extend(matched_nexts);
							}
						}
					}
					Combinator::Next => {
						let mut nexts = NodeList::with_capacity(node_list.length());
						for node in node_list.get_ref() {
							if let Some(next) = node.next_sibling()? {
								nexts.push(next.cloned());
							}
						}
						if nexts.length() > 0 {
							cur_result = rule.apply(&nexts, matched)?;
						}
					}
					Combinator::PrevAll => {
						for node in node_list.get_ref() {
							let nexts = node.previous_siblings()?;
							cur_result
								.get_mut_ref()
								.extend(rule.apply(&nexts, matched)?);
						}
					}
					Combinator::Prev => {
						let mut prevs = NodeList::with_capacity(node_list.length());
						for node in node_list.get_ref() {
							if let Some(next) = node.previous_sibling()? {
								prevs.push(next.cloned());
							}
						}
						if prevs.length() > 0 {
							cur_result = rule.apply(&prevs, matched)?;
						}
					}
					Combinator::Chain => {
						cur_result = rule.apply(&node_list, matched)?;
					}
				};
			}
			node_list = cur_result.unique();
			if node_list.length() == 0 {
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
				if node_list.length() == 0 {
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
						for node in prevs.get_ref() {
							if self.includes(node) {
								return true;
							}
						}
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
	/// pub fn `html`
	pub fn html(&self) -> &str {
		if let Some(node) = self.get(0) {
			return node.inner_html();
		}
		""
	}
	/// pub fn `text`
	pub fn text(&self) -> &str {
		let mut result = String::with_capacity(50);
		for node in self.get_ref() {
			result.push_str(node.text_content());
		}
		to_static_str(result)
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

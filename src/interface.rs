use crate::selector::{rule::Rule, Combinator, QueryProcess, Selector, SelectorSegment};
use crate::utils::{get_class_list, retain_by_index, to_static_str};
use crate::{constants::ATTR_CLASS, error::Error as IError};
use std::{any::Any, cmp::Ordering, collections::VecDeque, ops::Range};
use std::{collections::HashMap, error::Error};
use std::{collections::HashSet, rc::Rc};

pub type MaybeElement<'a> = Option<BoxDynElement<'a>>;
pub type MaybeDoc = Option<Box<dyn IDocumentTrait>>;
pub type BoxDynElement<'a> = Box<dyn IElementTrait + 'a>;
pub type BoxDynNode<'a> = Box<dyn INodeTrait + 'a>;
pub type BoxDynText<'a> = Box<dyn ITextTrait + 'a>;
pub type BoxDynUncareNode<'a> = Box<dyn IUncareNodeTrait + 'a>;
#[derive(Debug)]
pub enum IAttrValue {
	Value(String, Option<char>),
	True,
}

impl IAttrValue {
	/// pub fn `is_true`
	pub fn is_true(&self) -> bool {
		matches!(self, IAttrValue::True)
	}
	/// pub fn `is_str`
	pub fn is_str(&self, value: &str) -> bool {
		match self {
			IAttrValue::Value(v, _) => v == value,
			IAttrValue::True => false,
		}
	}
	/// pub fn `to_list`
	pub fn to_list(&self) -> Vec<&str> {
		match self {
			IAttrValue::Value(v, _) => v.trim().split_ascii_whitespace().collect::<Vec<&str>>(),
			IAttrValue::True => vec![],
		}
	}
}

/// impl `ToString` for IAttrValue
impl ToString for IAttrValue {
	fn to_string(&self) -> String {
		match self {
			IAttrValue::Value(v, _) => v.clone(),
			IAttrValue::True => String::new(),
		}
	}
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
	Element = 1,
	Text = 3,
	XMLCDATA = 4,
	Comment = 8,
	Document = 9,
	HTMLDOCTYPE = 10,
	DocumentFragement = 11,
	Other = 14,
}

impl INodeType {
	pub fn is_element(&self) -> bool {
		matches!(self, INodeType::Element)
	}
}
pub type IErrorHandle = Box<dyn Fn(Box<dyn Error>)>;
pub trait IDocumentTrait {
	fn get_element_by_id<'b>(&self, id: &str) -> Option<BoxDynElement<'b>>;
	fn onerror(&self) -> Option<Rc<IErrorHandle>> {
		None
	}
	fn trigger_error(&self, error: Box<dyn Error>) {
		if let Some(handle) = &self.onerror() {
			handle(error);
		}
	}
}
pub enum IEnumTyped<'a> {
	Element(BoxDynElement<'a>),
	Text(BoxDynText<'a>),
	UncareNode(BoxDynUncareNode<'a>),
}

impl<'a> IEnumTyped<'a> {
	pub fn into_element(self) -> Option<BoxDynElement<'a>> {
		match self {
			IEnumTyped::Element(ele) => Some(ele),
			_ => None,
		}
	}
	pub fn into_text(self) -> Option<BoxDynText<'a>> {
		match self {
			IEnumTyped::Text(ele) => Some(ele),
			_ => None,
		}
	}
}

pub trait INodeTrait {
	fn to_node(self: Box<Self>) -> Box<dyn Any>;
	// clone a ele
	fn clone_node<'b>(&self) -> BoxDynNode<'b>;
	// typed,whether element or text
	fn typed<'b>(self: Box<Self>) -> IEnumTyped<'b>;
	// get ele type
	fn node_type(&self) -> INodeType;
	// find parents
	fn parent<'b>(&self) -> MaybeElement<'b>;
	// check if two ele are the same
	fn uuid(&self) -> Option<&str>;
	// owner document
	fn owner_document(&self) -> MaybeDoc;
	// text
	fn text_content(&self) -> &str;
	fn text(&self) -> &str {
		self.text_content()
	}
	fn set_text(&mut self, content: &str);
	// set html
	fn set_html(&mut self, content: &str);
	// ele index
	fn index(&self) -> usize;
}

// get the ele indexs in tree
fn get_tree_indexs(ele: &BoxDynElement) -> VecDeque<usize> {
	let mut indexs: VecDeque<usize> = VecDeque::with_capacity(5);
	fn loop_handle(ele: &BoxDynElement, indexs: &mut VecDeque<usize>) {
		indexs.push_front(ele.index());
		if let Some(parent) = &ele.parent() {
			loop_handle(parent, indexs);
		}
	}
	loop_handle(ele, &mut indexs);
	indexs
}

// compare indexs
pub fn compare_indexs(a: &VecDeque<usize>, b: &VecDeque<usize>) -> Ordering {
	let a_total = a.len();
	let b_total = b.len();
	let loop_total = if a_total > b_total { b_total } else { a_total };
	for i in 0..loop_total {
		let a_index = a[i];
		let b_index = b[i];
		match a_index.cmp(&b_index) {
			Ordering::Equal => continue,
			order => return order,
		}
	}
	a_total.cmp(&b_total)
}

pub trait ITextTrait: INodeTrait {
	// remove the ele
	fn remove(self: Box<Self>);
	// append text at the end
	fn append_text(&mut self, content: &str);
	// prepend text at the start
	fn prepend_text(&mut self, content: &str);
}
pub trait IUncareNodeTrait: INodeTrait {}
#[derive(Default)]
pub struct Texts<'a> {
	nodes: Vec<BoxDynText<'a>>,
}

impl<'a> Texts<'a> {
	pub fn with_capacity(cap: usize) -> Self {
		Texts {
			nodes: Vec::with_capacity(cap),
		}
	}
	pub fn length(&self) -> usize {
		self.nodes.len()
	}
	pub fn is_empty(&self) -> bool {
		self.length() == 0
	}
	// get ref
	pub fn get_ref(&self) -> &Vec<BoxDynText<'a>> {
		&self.nodes
	}
	// get mut ref
	pub fn get_mut_ref(&mut self) -> &mut Vec<BoxDynText<'a>> {
		&mut self.nodes
	}
	// for each
	pub fn for_each<F>(&mut self, mut handle: F) -> &mut Self
	where
		F: FnMut(usize, &mut BoxDynText) -> bool,
	{
		for (index, ele) in self.get_mut_ref().iter_mut().enumerate() {
			if !handle(index, ele) {
				break;
			}
		}
		self
	}
	// alias for `for_each`
	pub fn each<F>(&mut self, handle: F) -> &mut Self
	where
		F: FnMut(usize, &mut BoxDynText) -> bool,
	{
		self.for_each(handle)
	}
	// filter_by
	pub fn filter_by<F>(&self, handle: F) -> Texts<'a>
	where
		F: Fn(usize, &BoxDynText) -> bool,
	{
		let mut result: Texts = Texts::with_capacity(self.length());
		for (index, ele) in self.get_ref().iter().enumerate() {
			if handle(index, ele) {
				result.get_mut_ref().push(
					ele
						.clone_node()
						.typed()
						.into_text()
						.expect("Text ele must can use 'into_text'."),
				);
			}
		}
		result
	}
	// remove
	pub fn remove(self) {
		for ele in self.into_iter() {
			ele.remove();
		}
	}
}
pub trait IElementTrait: INodeTrait {
	fn is(&self, ele: &BoxDynElement) -> bool {
		if let Some(uuid) = self.uuid() {
			if let Some(o_uuid) = ele.uuid() {
				return uuid == o_uuid;
			}
		}
		false
	}
	// root element
	fn root<'b>(&self) -> BoxDynElement<'b> {
		let mut root = self.parent();
		loop {
			if root.is_some() {
				let parent = root.as_ref().unwrap().parent();
				if let Some(parent) = &parent {
					root = Some(parent.cloned());
				} else {
					break;
				}
			} else {
				break;
			}
		}
		root.unwrap_or_else(|| self.cloned())
	}
	// cloned
	fn cloned<'b>(&self) -> BoxDynElement<'b> {
		let ele = self.clone_node();
		ele.typed().into_element().unwrap()
	}
	// next sibling
	fn next_element_sibling<'b>(&self) -> MaybeElement<'b> {
		// use child_nodes instead of chilren, reduce one loop
		if let Some(parent) = &self.parent() {
			// self index
			let index = self.index();
			let total = parent.child_nodes_length();
			// find the next
			for cur_index in index + 1..total {
				let ele = parent
					.child_nodes_item(cur_index)
					.expect("Child nodes item index must less than total");
				if matches!(ele.node_type(), INodeType::Element) {
					return Some(
						ele
							.typed()
							.into_element()
							.expect("Call `typed` for element ele."),
					);
				}
			}
		}
		None
	}
	// next siblings
	fn next_element_siblings<'b>(&self) -> Elements<'b> {
		// use child_nodes instead of chilren, reduce one loop
		if let Some(parent) = &self.parent() {
			// self index
			let index = self.index();
			let total = parent.child_nodes_length();
			let start_index = index + 1;
			// find the next
			let mut result: Elements = Elements::with_capacity(total - start_index);
			for cur_index in start_index..total {
				let ele = parent
					.child_nodes_item(cur_index)
					.expect("Child nodes item index must less than total");
				if matches!(ele.node_type(), INodeType::Element) {
					result.push(
						ele
							.typed()
							.into_element()
							.expect("Call `typed` for element ele."),
					);
				}
			}
			return result;
		}
		Elements::new()
	}
	// previous sibling
	fn previous_element_sibling<'b>(&self) -> MaybeElement<'b> {
		// use child_nodes instead of chilren, reduce one loop
		if let Some(parent) = &self.parent() {
			// self index
			let index = self.index();
			if index > 0 {
				// find the prev
				for cur_index in (0..index).rev() {
					let ele = parent
						.child_nodes_item(cur_index)
						.expect("Child nodes item index must less than total");
					if matches!(ele.node_type(), INodeType::Element) {
						return Some(
							ele
								.typed()
								.into_element()
								.expect("Call `typed` for element ele."),
						);
					}
				}
			}
		}
		None
	}
	// previous siblings
	fn previous_element_siblings<'b>(&self) -> Elements<'b> {
		// use child_nodes instead of chilren, reduce one loop
		if let Some(parent) = &self.parent() {
			// self index
			let index = self.index();
			if index > 0 {
				// find the prev
				let mut result: Elements = Elements::with_capacity(index);
				for cur_index in 0..index {
					let ele = parent
						.child_nodes_item(cur_index)
						.expect("Child nodes item index must less than total");
					if matches!(ele.node_type(), INodeType::Element) {
						result.push(
							ele
								.typed()
								.into_element()
								.expect("Call `typed` for element ele."),
						);
					}
				}
				return result;
			}
		}
		Elements::new()
	}
	// siblings
	fn siblings<'b>(&self) -> Elements<'b> {
		// use child_nodes instead of chilren, reduce one loop
		if let Some(parent) = &self.parent() {
			// self index
			let index = self.index();
			if index == 0 {
				return self.next_element_siblings();
			}
			let total = parent.child_nodes_length();
			if index == total - 1 {
				return self.previous_element_siblings();
			}
			let mut result: Elements = Elements::with_capacity(total - 1);
			fn loop_handle(range: &Range<usize>, parent: &BoxDynElement, result: &mut Elements) {
				for cur_index in range.start..range.end {
					let ele = parent
						.child_nodes_item(cur_index)
						.expect("Child nodes item index must less than total");
					if matches!(ele.node_type(), INodeType::Element) {
						result.push(
							ele
								.typed()
								.into_element()
								.expect("Call `typed` for element ele."),
						);
					}
				}
			}
			loop_handle(&(0..index), parent, &mut result);
			loop_handle(&(index + 1..total), parent, &mut result);
			return result;
		}
		Elements::new()
	}
	// tag name
	fn tag_name(&self) -> &str;
	// childs
	fn child_nodes_length(&self) -> usize;
	fn child_nodes_item<'b>(&self, index: usize) -> Option<BoxDynNode<'b>>;
	fn child_nodes<'b>(&self) -> Vec<BoxDynNode<'b>> {
		let total = self.child_nodes_length();
		let mut result = Vec::with_capacity(total);
		for index in 0..total {
			result.push(
				self
					.child_nodes_item(index)
					.expect("child nodes index must less than total."),
			);
		}
		result
	}
	fn children<'b>(&self) -> Elements<'b> {
		let child_nodes = self.child_nodes();
		let mut result = Elements::with_capacity(child_nodes.len());
		for ele in child_nodes.iter() {
			if let INodeType::Element = ele.node_type() {
				let ele = ele.clone_node();
				result.push(ele.typed().into_element().unwrap());
			}
		}
		result
	}
	// get all childrens
	fn childrens<'b>(&self) -> Elements<'b> {
		let childs = self.children();
		let count = childs.length();
		if count > 0 {
			let mut result = Elements::with_capacity(5);
			let all_nodes = result.get_mut_ref();
			for c in childs.get_ref() {
				all_nodes.push(c.cloned());
				all_nodes.extend(c.childrens());
			}
			return result;
		}
		Elements::new()
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

	// append child, insert before, remove child
	fn insert_adjacent(&mut self, position: &InsertPosition, ele: &BoxDynElement);
	fn remove_child(&mut self, ele: BoxDynElement);
	// texts
	fn texts<'b>(&self, _limit_depth: u32) -> Option<Texts<'b>> {
		None
	}
	// special for content tag, 'style','script','title','textarea'
	#[allow(clippy::boxed_local)]
	fn into_text<'b>(self: Box<Self>) -> Result<BoxDynText<'b>, Box<dyn Error>> {
		Err(Box::new(IError::InvalidTraitMethodCall {
			method: "into_text".into(),
			message: "The into_text method is not implemented.".into(),
		}))
	}
}

#[derive(Debug, PartialEq, Eq)]
enum FilterType {
	Filter,
	Not,
	Is,
	IsAll,
}
#[derive(Default)]
pub struct Elements<'a> {
	nodes: Vec<BoxDynElement<'a>>,
}

impl<'a> Elements<'a> {
	// crate only methods
	pub(crate) fn with_node(ele: &BoxDynElement) -> Self {
		Elements {
			nodes: vec![ele.cloned()],
		}
	}
	pub(crate) fn push(&mut self, ele: BoxDynElement<'a>) {
		self.get_mut_ref().push(ele);
	}

	pub(crate) fn trigger_method<F, T: Default>(&self, method: &str, selector: &str, handle: F) -> T
	where
		F: Fn(&mut Selector) -> T,
	{
		if !self.is_empty() {
			// filter handles don't use lookup
			const USE_LOOKUP: bool = false;
			let s = Selector::from_str(selector, USE_LOOKUP);
			if let Ok(mut s) = s {
				return handle(&mut s);
			}
			self.trigger_method_throw_error(method, Box::new(s.unwrap_err()));
		}
		Default::default()
	}

	pub(crate) fn trigger_method_throw_error(&self, method: &str, error: Box<dyn Error>) {
		if let Some(doc) = &self
			.get(0)
			.expect("Use index 0 when length > 0")
			.owner_document()
		{
			doc.trigger_error(Box::new(IError::MethodOnInvalidSelector {
				method: String::from(method),
				error: error.to_string(),
			}));
		}
	}
	// ------------Create new element-----------
	// new
	pub fn new() -> Self {
		Default::default()
	}
	// with nodes
	pub fn with_nodes(nodes: Vec<BoxDynElement<'a>>) -> Self {
		Elements { nodes }
	}
	// with capacity
	pub fn with_capacity(size: usize) -> Self {
		Elements {
			nodes: Vec::with_capacity(size),
		}
	}

	// -------------Helpers------------
	// get a element from the set
	pub fn get(&self, index: usize) -> Option<&BoxDynElement<'a>> {
		self.get_ref().get(index)
	}

	// get ref
	pub fn get_ref(&self) -> &Vec<BoxDynElement<'a>> {
		&self.nodes
	}

	// get mut ref
	pub fn get_mut_ref(&mut self) -> &mut Vec<BoxDynElement<'a>> {
		&mut self.nodes
	}
	// pub fn `for_each`
	pub fn for_each<F>(&mut self, mut handle: F) -> &mut Self
	where
		F: FnMut(usize, &mut BoxDynElement) -> bool,
	{
		for (index, ele) in self.get_mut_ref().iter_mut().enumerate() {
			if !handle(index, ele) {
				break;
			}
		}
		self
	}
	// alias for `for_each`
	pub fn each<F>(&mut self, handle: F) -> &mut Self
	where
		F: FnMut(usize, &mut BoxDynElement) -> bool,
	{
		self.for_each(handle)
	}
	// pub fn `map`
	pub fn map<F, T: Sized>(&self, handle: F) -> Vec<T>
	where
		F: Fn(usize, &BoxDynElement) -> T,
	{
		let mut result: Vec<T> = Vec::with_capacity(self.length());
		for (index, ele) in self.get_ref().iter().enumerate() {
			result.push(handle(index, ele));
		}
		result
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
	fn select_with_comb(&self, method: &str, selector: &str, comb: Combinator) -> Elements<'a> {
		if selector.is_empty() {
			let segment = Selector::make_comb_all(comb);
			let selector = Selector::from_segment(segment);
			return self.find_selector(&selector);
		}
		self.trigger_method(method, selector, |selector| {
			selector.head_combinator(comb);
			self.find_selector(&selector)
		})
	}
	// for all combinator until selectors
	fn select_with_comb_until(
		&self,
		method: &str,
		selector: &str,
		filter: &str,
		contains: bool,
		comb: Combinator,
	) -> Elements<'a> {
		let selector = selector.parse::<Selector>();
		if let Ok(selector) = &selector {
			let segment = Selector::make_comb_all(comb);
			let next_selector = Selector::from_segment(segment);
			let mut result = Elements::with_capacity(5);
			let (next_ok, filter) = if !filter.is_empty() {
				let filter = filter.parse::<Selector>();
				if let Ok(filter) = filter {
					(true, Some(filter))
				} else {
					self.trigger_method_throw_error(method, Box::new(filter.unwrap_err()));
					(false, None)
				}
			} else {
				(true, None)
			};
			if next_ok {
				// has filter
				for ele in self.get_ref() {
					let mut cur_eles = Elements::with_node(ele);
					loop {
						// find the next element
						cur_eles = cur_eles.find_selector(&next_selector);
						if !cur_eles.is_empty() {
							let meet_until = cur_eles.filter_type_handle(&selector, &FilterType::Is).1;
							// meet the until element, and not contains, stop before check element
							if meet_until && !contains {
								break;
							}
							// check if cur_eles filter
							let should_add = if let Some(filter) = &filter {
								// filter true
								cur_eles.filter_type_handle(filter, &FilterType::Is).1
							} else {
								// no need filter, just add
								true
							};
							if should_add {
								result.push(
									cur_eles
										.get(0)
										.expect("Elements get 0 must have when length > 0")
										.cloned(),
								);
							}
							// find the until, stop the loop at the end whenever contains or not
							if meet_until {
								break;
							}
						} else {
							break;
						}
					}
				}
				return result;
			}
		} else {
			self.trigger_method_throw_error(method, Box::new(selector.unwrap_err()));
		}
		Elements::new()
	}
	// keep one sibling, first<asc:true> or last<asc:false>
	fn unique_sibling(&self, asc: bool) -> Elements<'a> {
		let total = self.length();
		let mut parents_indexs: HashSet<VecDeque<usize>> = HashSet::with_capacity(total);
		let mut uniques = Elements::with_capacity(total);
		let mut prev_parent: Option<BoxDynElement> = None;
		let mut has_root = false;
		let mut handle = |ele: &BoxDynElement| {
			if let Some(parent) = &ele.parent() {
				if let Some(prev_parent) = &prev_parent {
					if parent.is(prev_parent) {
						return;
					}
				}
				// set prev parent
				prev_parent = Some(parent.cloned());
				// parents
				let indexs = get_tree_indexs(parent);
				// new parent
				if parents_indexs.get(&indexs).is_none() {
					parents_indexs.insert(indexs);
					uniques.push(ele.cloned());
				}
			} else if !has_root {
				has_root = true;
				uniques.push(ele.cloned());
			}
		};
		// just keep one sibling node
		if asc {
			for ele in self.get_ref() {
				handle(ele);
			}
		} else {
			for ele in self.get_ref().iter().rev() {
				handle(ele)
			}
		}
		uniques
	}
	// keep first sibling
	fn unique_sibling_first(&self) -> Elements<'a> {
		self.unique_sibling(true)
	}
	// keep last sibling
	fn unique_sibling_last(&self) -> Elements<'a> {
		self.unique_sibling(false)
	}
	// sort
	fn sort(&mut self) {
		self.get_mut_ref().sort_by(|a, b| {
			let a_index = get_tree_indexs(a);
			let b_index = get_tree_indexs(b);
			compare_indexs(&a_index, &b_index)
		});
	}
	// unique
	fn unique(&mut self) {
		self.get_mut_ref().dedup_by(|a, b| a.is(b));
	}
	// sort then unique
	fn sort_and_unique(&mut self) {
		self.sort();
		self.unique();
	}
	// prev
	pub fn prev(&self, selector: &str) -> Elements<'a> {
		self.select_with_comb("prev", selector, Combinator::Prev)
	}
	// prev_all
	pub fn prev_all(&self, selector: &str) -> Elements<'a> {
		let uniques = self.unique_sibling_last();
		uniques.select_with_comb("prev_all", selector, Combinator::PrevAll)
	}
	// prev_until
	pub fn prev_until(&self, selector: &str, filter: &str, contains: bool) -> Elements<'a> {
		let uniques = self.unique_sibling_last();
		uniques.select_with_comb_until("prev_until", selector, filter, contains, Combinator::Prev)
	}
	// next
	pub fn next(&self, selector: &str) -> Elements<'a> {
		self.select_with_comb("next", selector, Combinator::Next)
	}
	// next_all
	pub fn next_all(&self, selector: &str) -> Elements<'a> {
		// unique, keep the first sibling node
		let uniques = self.unique_sibling_first();
		uniques.select_with_comb("next_all", selector, Combinator::NextAll)
	}
	// next_until
	pub fn next_until(&self, selector: &str, filter: &str, contains: bool) -> Elements<'a> {
		// unique, keep the first sibling node
		let uniques = self.unique_sibling_first();
		uniques.select_with_comb_until("next_until", selector, filter, contains, Combinator::Next)
	}
	// siblings
	pub fn siblings(&self, selector: &str) -> Elements<'a> {
		// should first unique siblings
		// if have two siblings, then use parent.children
		let total = self.length();
		let mut parents_indexs: HashMap<VecDeque<usize>, (usize, bool)> = HashMap::with_capacity(total);
		let mut uniques: Vec<(BoxDynElement, bool)> = Vec::with_capacity(total);
		let mut prev_parent: Option<BoxDynElement> = None;
		let mut continued = false;
		// just keep one sibling node
		for ele in self.get_ref() {
			if let Some(parent) = &ele.parent() {
				if let Some(prev_parent) = &prev_parent {
					if parent.is(prev_parent) {
						if !continued {
							// may first meet the sibling, set use all children
							if let Some(pair) = uniques.last_mut() {
								*pair = (parent.cloned(), true);
							}
							continued = true;
						}
						continue;
					}
				}
				// reset continued
				continued = false;
				// set prev parent
				prev_parent = Some(parent.cloned());
				// parent indexs
				let indexs = get_tree_indexs(parent);
				// new parent
				if let Some((index, setted)) = parents_indexs.get_mut(&indexs) {
					if !*setted {
						if let Some(pair) = uniques.get_mut(*index) {
							*pair = (parent.cloned(), true);
						}
						*setted = true;
					}
				} else {
					parents_indexs.insert(indexs, (uniques.len(), false));
					uniques.push((ele.cloned(), false));
				}
			}
		}
		// when selector is empty or only
		let mut siblings_selector: Selector;
		let siblings_comb = Combinator::Siblings;
		let mut child_selector: Selector;
		let child_comb = Combinator::Children;
		let selector = selector.trim();
		if selector.is_empty() {
			siblings_selector = Selector::from_segment(Selector::make_comb_all(siblings_comb));
			child_selector = Selector::from_segment(Selector::make_comb_all(child_comb));
		} else {
			// self
			let sib_selector = selector.parse::<Selector>();
			if let Ok(sib_selector) = sib_selector {
				// clone the selector to a child selector
				child_selector = sib_selector.clone();
				child_selector.head_combinator(child_comb);
				// use siblings selector
				siblings_selector = sib_selector;
				siblings_selector.head_combinator(siblings_comb);
			} else {
				self.trigger_method_throw_error(
					"siblings",
					Box::new(IError::InvalidTraitMethodCall {
						method: "siblings".to_string(),
						message: format!(
							"Invalid selector:{}",
							sib_selector.err().expect("Selector parse error")
						),
					}),
				);
				return Elements::new();
			}
		}
		// uniques
		let mut result = Elements::with_capacity(5);
		for (ele, is_parent) in &uniques {
			let eles = Elements::with_node(ele);
			let finded = if *is_parent {
				eles.find_selector(&child_selector)
			} else {
				eles.find_selector(&siblings_selector)
			};
			result.get_mut_ref().extend(finded);
		}
		// sort the result
		result.sort();
		result
	}
	// children
	pub fn children(&self, selector: &str) -> Elements<'a> {
		self.select_with_comb("children", selector, Combinator::Children)
	}

	// parent
	pub fn parent(&self, selector: &str) -> Elements<'a> {
		// unique, keep the first sibling node
		let uniques = self.unique_sibling_first();
		uniques.select_with_comb("parent", selector, Combinator::Parent)
	}
	// parents
	pub fn parents(&self, selector: &str) -> Elements<'a> {
		// unique, keep the first sibling node
		let uniques = self.unique_sibling_first();
		let mut result = uniques.select_with_comb("parents", selector, Combinator::ParentAll);
		result.sort_and_unique();
		result
	}
	// parents_until
	pub fn parents_until(&self, selector: &str, filter: &str, contains: bool) -> Elements<'a> {
		// unique, keep the first sibling node
		let uniques = self.unique_sibling_first();
		let mut result = uniques.select_with_comb_until(
			"parents_until",
			selector,
			filter,
			contains,
			Combinator::Parent,
		);
		result.sort_and_unique();
		result
	}
	// closest
	pub fn closest(&self, selector: &str) -> Elements<'a> {
		// when selector is not provided
		if selector.is_empty() {
			return Elements::new();
		}
		// find the nearst node
		const METHOD: &str = "closest";
		let selector = selector.parse::<Selector>();
		if let Ok(selector) = selector {
			let total = self.length();
			let mut result = Elements::with_capacity(total);
			let mut propagations = Elements::with_capacity(total);
			for ele in self.get_ref() {
				let mut cur_eles = Elements::with_node(ele);
				if cur_eles.filter_type_handle(&selector, &FilterType::Is).1 {
					// check self
					result.get_mut_ref().push(cur_eles.get_mut_ref().remove(0));
				} else {
					propagations
						.get_mut_ref()
						.push(cur_eles.get_mut_ref().remove(0));
				}
			}
			if !propagations.is_empty() {
				let uniques = propagations.unique_sibling_first();
				for ele in uniques.get_ref() {
					let mut cur_eles = Elements::with_node(ele);
					loop {
						if cur_eles.filter_type_handle(&selector, &FilterType::Is).1 {
							result.get_mut_ref().push(cur_eles.get_mut_ref().remove(0));
							break;
						}
						if let Some(parent) = &cur_eles
							.get(0)
							.expect("Elements must have one node")
							.parent()
						{
							cur_eles = Elements::with_node(parent);
						} else {
							break;
						}
					}
				}
				// need sort and unique
				result.sort_and_unique();
			}
			result
		} else {
			self.trigger_method_throw_error(METHOD, Box::new(selector.unwrap_err()));
			Elements::new()
		}
	}
	// for `find` and `select_with_comb`
	fn find_selector(&self, selector: &Selector) -> Elements<'a> {
		let mut result = Elements::with_capacity(5);
		if !self.is_empty() {
			for p in &selector.process {
				let QueryProcess { should_in, query } = p;
				let first_query = &query[0];
				let mut group: Option<Elements> = None;
				let mut start_rule_index: usize = 0;
				let mut is_empty = false;
				if let Some(lookup) = should_in {
					let mut cur_group = Elements::with_capacity(5);
					// get finded
					let finded = Elements::select(self, first_query, Some(&Combinator::ChildrenAll));
					if !finded.is_empty() {
						let tops = Elements::select(self, &lookup[0], None);
						if !tops.is_empty() {
							// remove the first
							start_rule_index = 1;
							// check if the previous ele and the current ele are siblings.
							let mut prev_ele: Option<&BoxDynElement> = None;
							let mut is_find = false;
							let first_comb = &first_query[0].2;
							for ele in finded.get_ref() {
								if prev_ele.is_some()
									&& Elements::is_sibling(ele, prev_ele.expect("Has test is_some"))
								{
									match first_comb {
										Combinator::Next => {
											if is_find {
												// do nothing, because has finded the only sibling ele matched.
												continue;
											}
											// if not find, should detect the current ele
										}
										Combinator::NextAll => {
											if is_find {
												cur_group.push(ele.cloned());
												continue;
											}
											// if not find, should detect the ele
										}
										_ => {
											// do the same thing as `prev_ele`
											// if `is_find` is true, then add the ele, otherwise it's not matched too.
											// keep the `is_find` value
											if is_find {
												cur_group.push(ele.cloned());
											}
											continue;
										}
									};
								}
								// check if the ele is in firsts
								if tops.has_ele(ele, &first_comb, Some(&lookup[1..])) {
									cur_group.push(ele.cloned());
									is_find = true;
								} else {
									is_find = false;
								}
								// set the prev ele
								prev_ele = Some(ele);
							}
						}
					}
					is_empty = cur_group.is_empty();
					group = Some(cur_group);
				}
				if !is_empty {
					let query = &query[start_rule_index..];
					if !query.is_empty() {
						let mut is_empty = false;
						let mut group = Elements::select(group.as_ref().unwrap_or(self), &query[0], None);
						for rules in &query[1..] {
							group = Elements::select(&group, rules, None);
							if group.is_empty() {
								is_empty = true;
								break;
							}
						}
						if !is_empty {
							result = result.add(group);
						}
					} else {
						let group = group.unwrap_or_else(|| self.cloned());
						if !group.is_empty() {
							result = result.add(group);
						}
					}
				}
			}
		}
		result
	}

	/// pub fn `find`
	/// get elements by selector, support most of css selectors
	pub fn find(&self, selector: &str) -> Elements<'a> {
		self.trigger_method("find", selector, |selector| self.find_selector(selector))
	}

	// filter_type_handle:
	// type     | rule processes
	// ----------------------------------------
	// Filter   | merge all elmements which matched each process
	// Not      | merge all elmements which matched each process, then exclude them all.
	// Is       | once matched a process, break
	// IsAll    | merge all elmements which matched each process, check if the matched equal to self
	fn filter_type_handle(
		&self,
		selector: &Selector,
		filter_type: &FilterType,
	) -> (Elements<'a>, bool) {
		let eles = self.get_ref();
		let total = eles.len();
		let mut result = Elements::with_capacity(total);
		let mut all_matched = false;
		let chain_comb = Combinator::Chain;
		let mut root: Option<Elements> = None;
		for process in selector.process.iter() {
			// filter methods make sure do not use `should_in`
			let QueryProcess { query, .. } = process;
			let query_num = query.len();
			let mut filtered = Elements::new();
			if query_num > 0 {
				let last_query = &query[query_num - 1];
				let last_query_first_rule = &last_query[0];
				filtered =
					Elements::select_by_rule(self, last_query_first_rule, Some(&chain_comb)).cloned();
				if !filtered.is_empty() && last_query.len() > 1 {
					for rule in &last_query[1..] {
						filtered = Elements::select_by_rule(&filtered, rule, None);
						if filtered.is_empty() {
							break;
						}
					}
				}
				if !filtered.is_empty() && query_num > 1 {
					// set root first
					root = root.or_else(|| {
						let root_element = filtered
							.get(0)
							.expect("Filtered length greater than 0")
							.root();
						Some(Elements::with_node(&root_element))
					});
					// get root elements
					let root_eles = root.as_ref().expect("root element must have");
					// find elements from root_eles by selector
					let lookup = Some(&query[..query_num - 1]);
					let mut lasts = Elements::with_capacity(filtered.length());
					let comb = &last_query_first_rule.2;
					for ele in filtered.get_ref() {
						if root_eles.has_ele(ele, comb, lookup) {
							lasts.get_mut_ref().push(ele.cloned());
						}
					}
					filtered = lasts;
				}
			}
			if !filtered.is_empty() {
				match filter_type {
					FilterType::Is => {
						all_matched = true;
						break;
					}
					_ => {
						result = result.add(filtered);
					}
				}
			}
		}
		match filter_type {
			FilterType::IsAll => {
				all_matched = result.length() == total;
			}
			FilterType::Not => {
				// Exclude `filtered` from self
				if result.is_empty() {
					// no element matched the not selector
					result = self.cloned();
				} else {
					// filtered by not in
					result = self.not_in(&result);
				}
			}
			_ => {
				// FilterType::Is: just return 'all_matched'
				// FilterType::Filter: just return 'result'
			}
		}
		(result, all_matched)
	}

	// filter in type
	fn filter_in_handle(&self, search: &Elements, filter_type: FilterType) -> (Elements<'a>, bool) {
		let eles = self.get_ref();
		let total = eles.len();
		let mut result = Elements::with_capacity(total);
		let mut all_matched = false;
		match filter_type {
			FilterType::Filter => {
				let mut start_index = 0;
				let search_total = search.length();
				for ele in eles {
					if let Some(index) = search.index_of(ele, start_index) {
						// also in search, include
						start_index = index + 1;
						result.push(ele.cloned());
						if start_index >= search_total {
							break;
						}
					}
				}
			}
			FilterType::Not => {
				let mut start_index = 0;
				for ele in eles {
					if let Some(index) = search.index_of(ele, start_index) {
						// also in search, exclude
						start_index = index + 1;
					} else {
						result.push(ele.cloned());
					}
				}
			}
			FilterType::Is => {
				for ele in eles {
					if search.includes(ele) {
						all_matched = true;
						break;
					}
				}
			}
			FilterType::IsAll => {
				if total <= search.length() {
					let mut is_all_matched = true;
					let mut start_index = 0;
					for ele in eles {
						if let Some(index) = search.index_of(ele, start_index) {
							// also in search, exclude
							start_index = index + 1;
						} else {
							is_all_matched = false;
							break;
						}
					}
					all_matched = is_all_matched;
				}
			}
		}
		(result, all_matched)
	}

	// filter
	pub fn filter(&self, selector: &str) -> Elements<'a> {
		const METHOD: &str = "filter";
		self.trigger_method(METHOD, selector, |selector| {
			self.filter_type_handle(&selector, &FilterType::Filter).0
		})
	}

	// filter_by
	pub fn filter_by<F>(&self, handle: F) -> Elements<'a>
	where
		F: Fn(usize, &BoxDynElement) -> bool,
	{
		let mut result = Elements::with_capacity(self.length());
		for (index, ele) in self.get_ref().iter().enumerate() {
			if handle(index, ele) {
				// find the ele, allow cloned
				result.push(ele.cloned());
			}
		}
		result
	}

	// filter in
	pub fn filter_in(&self, search: &Elements) -> Elements<'a> {
		self.filter_in_handle(search, FilterType::Filter).0
	}

	// is
	pub fn is(&self, selector: &str) -> bool {
		const METHOD: &str = "is";
		self.trigger_method(METHOD, selector, |selector| {
			self.filter_type_handle(selector, &FilterType::Is).1
		})
	}

	// is by
	pub fn is_by<F>(&self, handle: F) -> bool
	where
		F: Fn(usize, &BoxDynElement) -> bool,
	{
		let mut flag = false;
		for (index, ele) in self.get_ref().iter().enumerate() {
			if handle(index, ele) {
				flag = true;
				break;
			}
		}
		flag
	}

	// is in
	pub fn is_in(&self, search: &Elements) -> bool {
		self.filter_in_handle(search, FilterType::Is).1
	}

	// is_all
	pub fn is_all(&self, selector: &str) -> bool {
		const METHOD: &str = "is_all";
		self.trigger_method(METHOD, selector, |selector| {
			self.filter_type_handle(&selector, &FilterType::IsAll).1
		})
	}

	// is_all_by
	pub fn is_all_by<F>(&self, handle: F) -> bool
	where
		F: Fn(usize, &BoxDynElement) -> bool,
	{
		let mut flag = true;
		for (index, ele) in self.get_ref().iter().enumerate() {
			if !handle(index, ele) {
				flag = false;
				break;
			}
		}
		flag
	}

	// is_all_in
	pub fn is_all_in(&self, search: &Elements) -> bool {
		self.filter_in_handle(search, FilterType::IsAll).1
	}

	// not
	pub fn not(&self, selector: &str) -> Elements<'a> {
		const METHOD: &str = "not";
		self.trigger_method(METHOD, selector, |selector| {
			self.filter_type_handle(&selector, &FilterType::Not).0
		})
	}

	// not by
	pub fn not_by<F>(&self, handle: F) -> Elements<'a>
	where
		F: Fn(usize, &BoxDynElement) -> bool,
	{
		let mut result = Elements::with_capacity(self.length());
		for (index, ele) in self.get_ref().iter().enumerate() {
			if !handle(index, ele) {
				result.push(ele.cloned());
			}
		}
		result
	}

	/// pub fn `not_in`
	/// remove element from `Self` which is also in `search`
	pub fn not_in(&self, search: &Elements) -> Elements<'a> {
		self.filter_in_handle(search, FilterType::Not).0
	}

	// has
	pub fn has(&self, selector: &str) -> Elements<'a> {
		const METHOD: &str = "has";
		fn loop_handle(ele: &BoxDynElement, selector: &Selector) -> bool {
			let childs = ele.children();
			if !childs.is_empty() {
				let (_, all_matched) = childs.filter_type_handle(selector, &FilterType::Is);
				if all_matched {
					return true;
				}
				for child in childs.get_ref() {
					if loop_handle(child, selector) {
						return true;
					}
				}
			}
			false
		}
		self.trigger_method(METHOD, selector, |selector| {
			self.filter_by(|_, ele| loop_handle(ele, selector))
		})
	}

	// has_in
	pub fn has_in(&self, search: &Elements) -> Elements<'a> {
		fn loop_handle(ele: &BoxDynElement, search: &Elements) -> bool {
			let childs = ele.children();
			if !childs.is_empty() {
				let (_, all_matched) = childs.filter_in_handle(search, FilterType::Is);
				if all_matched {
					return true;
				}
				for child in childs.get_ref() {
					if loop_handle(child, search) {
						return true;
					}
				}
			}
			false
		}
		self.filter_by(|_, ele| loop_handle(ele, &search))
	}

	/// pub fn `eq`
	/// get a element by index
	pub fn eq(&self, index: usize) -> Elements<'a> {
		if let Some(ele) = self.get(index) {
			Elements::with_node(ele)
		} else {
			Elements::new()
		}
	}

	/// pub fn `first`
	/// get the first element, alias for 'eq(0)'
	pub fn first(&self) -> Elements<'a> {
		self.eq(0)
	}

	/// pub fn `last`
	/// get the last element, alias for 'eq(len - 1)'
	pub fn last(&self) -> Elements<'a> {
		self.eq(self.length() - 1)
	}

	/// pub fn `slice`
	/// get elements by a range parameter
	/// `slice(0..1)` equal to `eq(0)`, `first`
	pub fn slice(&self, range: &Range<usize>) -> Elements<'a> {
		let start = range.start;
		let end = range.end;
		let total = self.length();
		if start >= total {
			return Elements::new();
		}
		let end = if end <= total { end } else { total };
		let mut result = Elements::with_capacity(end - start);
		let eles = self.get_ref();
		for ele in &eles[start..end] {
			result.push(ele.cloned());
		}
		result
	}

	/// pub fn `add`
	/// concat two element set to a new set,
	/// it will take the owership of the parameter element set, but no sence to `Self`
	pub fn add(&self, eles: Elements<'a>) -> Elements<'a> {
		if self.is_empty() {
			return eles;
		}
		if eles.is_empty() {
			return self.cloned();
		}
		let first_eles = self;
		let second_eles = &eles;
		// compare first and second
		let first_count = first_eles.length();
		let second_count = second_eles.length();
		let avg = second_count / 3;
		let mut prevs: Vec<usize> = Vec::with_capacity(avg);
		let mut mids: Vec<(usize, usize)> = Vec::with_capacity(avg);
		let mut afters: Vec<usize> = Vec::with_capacity(avg);
		let mut sec_left_index = 0;
		let sec_right_index = second_count - 1;
		let mut first_indexs: HashMap<usize, VecDeque<usize>> = HashMap::with_capacity(first_count);
		let mut fir_left_index = 0;
		let fir_right_index = first_count - 1;
		let first = first_eles.get_ref();
		let second = second_eles.get_ref();
		// get first index cached or from cached
		fn get_first_index_cached<'a>(
			first_indexs: &'a mut HashMap<usize, VecDeque<usize>>,
			first: &[BoxDynElement],
			index: usize,
		) -> &'a mut VecDeque<usize> {
			first_indexs
				.entry(index)
				.or_insert_with(|| get_tree_indexs(&first[index]))
		};
		while fir_left_index <= fir_right_index && sec_left_index <= sec_right_index {
			// the second left
			let sec_left = &second[sec_left_index];
			let sec_left_level = get_tree_indexs(sec_left);
			// the first left
			let fir_left_level = get_first_index_cached(&mut first_indexs, &first, fir_left_index);
			match compare_indexs(&sec_left_level, &fir_left_level) {
				Ordering::Equal => {
					// move forward both
					sec_left_index += 1;
					fir_left_index += 1;
				}
				Ordering::Greater => {
					// second left is behind first left
					// if second left is also behind first right
					let fir_right_level = get_first_index_cached(&mut first_indexs, &first, fir_right_index);
					match compare_indexs(&sec_left_level, &fir_right_level) {
						Ordering::Greater => {
							// now second is all after first
							afters.extend(sec_left_index..=sec_right_index);
							break;
						}
						Ordering::Less => {
							// second left is between first left and right
							// use binary search
							let mut l = fir_left_index;
							let mut r = fir_right_index;
							let mut mid = (l + r) / 2;
							let mut find_equal = false;
							while mid != l {
								let mid_level = get_first_index_cached(&mut first_indexs, &first, mid);
								match compare_indexs(&sec_left_level, &mid_level) {
									Ordering::Greater => {
										// second left is behind middle
										l = mid;
										mid = (l + r) / 2;
									}
									Ordering::Less => {
										// second left is before middle
										r = mid;
										mid = (l + r) / 2;
									}
									Ordering::Equal => {
										// find equal
										find_equal = true;
										break;
									}
								}
							}
							if !find_equal {
								mids.push((sec_left_index, mid + 1));
							}
							// set first left from mid
							fir_left_index = mid;
							// move second left index
							sec_left_index += 1;
						}
						Ordering::Equal => {
							// equal to first right, now all the second after current is behind first
							afters.extend(sec_left_index + 1..=sec_right_index);
							break;
						}
					}
				}
				Ordering::Less => {
					let sec_right = &second[sec_right_index];
					let sec_right_level = get_tree_indexs(sec_right);
					match compare_indexs(&sec_right_level, &fir_left_level) {
						Ordering::Less => {
							// now second is all before first
							prevs.extend(sec_left_index..=sec_right_index);
							break;
						}
						Ordering::Greater => {
							// second contains first or second right is in first
							// just move second left
							prevs.push(sec_left_index);
							sec_left_index += 1;
						}
						Ordering::Equal => {
							// equal to first left, now all the second are before first left
							prevs.extend(sec_left_index..sec_right_index);
							break;
						}
					}
				}
			}
		}
		let prevs_count = prevs.len();
		let mids_count = mids.len();
		let afters_count = afters.len();
		let mut result = Elements::with_capacity(first_count + prevs_count + mids_count + afters_count);
		if prevs_count > 0 {
			// add prevs
			for index in prevs {
				let ele = &second[index];
				result.push(ele.cloned());
			}
		}
		// add first and mids
		let mut mid_loop = 0;
		for (index, ele) in first_eles.get_ref().iter().enumerate() {
			if mid_loop < mids_count {
				let cur_mids = &mids[mid_loop..];
				// maybe multiple middles is between first left and right
				for (sec_index, mid_index) in cur_mids {
					if *mid_index == index {
						mid_loop += 1;
						let mid_ele = &second[*sec_index];
						result.push(mid_ele.cloned());
					} else {
						break;
					}
				}
			}
			result.push(ele.cloned());
		}
		// add afters
		if afters_count > 0 {
			// add afters
			for index in afters {
				let ele = &second[index];
				result.push(ele.cloned());
			}
		}
		result
	}
	// find a ele and then remove it
	fn find_out(elements: &mut Elements<'a>, item: &BoxDynElement) -> Option<BoxDynElement<'a>> {
		let mut find_index: Option<usize> = None;
		for (index, ele) in elements.get_ref().iter().enumerate() {
			if ele.is(item) {
				find_index = Some(index);
				break;
			}
		}
		if let Some(index) = find_index {
			return Some(elements.get_mut_ref().remove(index));
		}
		None
	}
	// select one rule
	// the rule must not in cache
	fn select_by_rule(
		elements: &Elements<'a>,
		rule_item: &SelectorSegment,
		comb: Option<&Combinator>,
	) -> Elements<'a> {
		let cur_comb = comb.unwrap_or(&rule_item.2);
		let (rule, matched, ..) = rule_item;
		let mut result = Elements::with_capacity(5);
		use Combinator::*;
		match cur_comb {
			ChildrenAll => {
				// depth first search, keep the appear order
				for ele in elements.get_ref() {
					// get children
					let childs = ele.children();
					if !childs.is_empty() {
						// apply rule
						let mut matched_childs = rule.apply(&childs, matched);
						for child in childs.get_ref() {
							let matched = Elements::find_out(&mut matched_childs, child);
							let is_matched = matched.is_some();
							let sub_childs = child.children();
							if !sub_childs.is_empty() {
								// add has finded
								if is_matched {
									result
										.get_mut_ref()
										.push(matched.expect("Has test is_some"));
								}
								// search sub child
								let cur = Elements::with_node(child);
								let sub_matched = Elements::select_by_rule(&cur, rule_item, comb);
								if !sub_matched.is_empty() {
									result.get_mut_ref().extend(sub_matched);
								}
							} else if is_matched {
								// move the matched ele out from cur
								result
									.get_mut_ref()
									.push(matched.expect("Has test is_some"));
							}
						}
					}
				}
				// maybe not unique, because some elements may be parent and child relation.
				result.sort_and_unique();
			}
			Children => {
				// because elements is unique, so the children is unique too
				for ele in elements.get_ref() {
					let childs = ele.children();
					let match_childs = rule.apply(&childs, matched);
					if !match_childs.is_empty() {
						result.get_mut_ref().extend(match_childs);
					}
				}
			}
			Parent => {
				// because elements is unique, so the parent is unique too
				for ele in elements.get_ref() {
					if let Some(parent) = &ele.parent() {
						let plist = Elements::with_node(parent);
						let matched = rule.apply(&plist, matched);
						if !matched.is_empty() {
							result.get_mut_ref().extend(matched);
						}
					}
				}
			}
			ParentAll => {
				for ele in elements.get_ref() {
					if let Some(parent) = &ele.parent() {
						let plist = Elements::with_node(parent);
						let matched = rule.apply(&plist, matched);
						if !matched.is_empty() {
							result.get_mut_ref().extend(matched);
						}
						if parent.parent().is_some() {
							let ancestors = Elements::select_by_rule(&plist, rule_item, comb);
							if !ancestors.is_empty() {
								result.get_mut_ref().extend(ancestors);
							}
						}
					}
				}
				// maybe not unique
				result.sort_and_unique();
			}
			NextAll => {
				// unique siblings just keep first
				let uniques = elements.unique_sibling_first();
				for ele in uniques.get_ref() {
					let nexts = ele.next_element_siblings();
					let matched_nexts = rule.apply(&nexts, matched);
					if !matched_nexts.is_empty() {
						result.get_mut_ref().extend(matched_nexts);
					}
				}
			}
			Next => {
				// because elements is unique, so the next is unique too
				let mut nexts = Elements::with_capacity(elements.length());
				for ele in elements.get_ref() {
					if let Some(next) = ele.next_element_sibling() {
						nexts.push(next.cloned());
					}
				}
				if !nexts.is_empty() {
					result = rule.apply(&nexts, matched);
				}
			}
			PrevAll => {
				// unique siblings just keep last
				let uniques = elements.unique_sibling_last();
				for ele in uniques.get_ref() {
					let nexts = ele.previous_element_siblings();
					result.get_mut_ref().extend(rule.apply(&nexts, matched));
				}
			}
			Prev => {
				let mut prevs = Elements::with_capacity(elements.length());
				for ele in elements.get_ref() {
					if let Some(next) = ele.previous_element_sibling() {
						prevs.push(next.cloned());
					}
				}
				if !prevs.is_empty() {
					result = rule.apply(&prevs, matched);
				}
			}
			Siblings => {
				for ele in elements.get_ref() {
					let siblings = ele.siblings();
					result.get_mut_ref().extend(rule.apply(&siblings, matched));
				}
				// maybe not unique
				result.sort_and_unique();
			}
			Chain => {
				result = rule.apply(&elements, matched);
			}
		};
		result
	}
	// select ele by rules
	fn select(
		elements: &Elements<'a>,
		rules: &[SelectorSegment],
		comb: Option<&Combinator>,
	) -> Elements<'a> {
		let first_rule = &rules[0];
		let comb = comb.unwrap_or(&first_rule.2);
		let mut elements = if first_rule.0.in_cache && matches!(comb, Combinator::ChildrenAll) {
			// set use cache data
			let (rule, matched, ..) = first_rule;
			// clone matched data
			let mut matched = matched.clone();
			// add use cache
			Rule::use_cache(&mut matched);
			let cached = rule.apply(&elements, &matched);
			let count = cached.length();
			if count > 0 {
				let mut result = Elements::with_capacity(count);
				for ele in cached.get_ref() {
					if elements.has_ele(ele, comb, None) {
						result.push(ele.cloned());
					}
				}
				result.sort_and_unique();
				result
			} else {
				Elements::new()
			}
		} else {
			Elements::select_by_rule(&elements, first_rule, Some(comb))
		};
		if !elements.is_empty() && rules.len() > 1 {
			for rule in &rules[1..] {
				elements = Elements::select_by_rule(&elements, rule, None);
				if elements.is_empty() {
					break;
				}
			}
		}
		elements
	}
	// cloned
	pub fn cloned(&self) -> Elements<'a> {
		let mut result = Elements::with_capacity(self.length());
		for ele in &self.nodes {
			result.push(ele.cloned());
		}
		result
	}
	// `has_ele`
	pub(crate) fn has_ele(
		&self,
		ele: &BoxDynElement,
		comb: &Combinator,
		lookup: Option<&[Vec<SelectorSegment>]>,
	) -> bool {
		let mut elements = Elements::with_node(ele);
		let mut lookup_comb = comb.reverse();
		if let Some(lookup) = lookup {
			for rules in lookup.iter().rev() {
				let finded = Elements::select(&elements, rules, Some(&lookup_comb));
				if finded.is_empty() {
					return false;
				}
				lookup_comb = rules[0].2.reverse();
				elements = finded;
			}
		}
		use Combinator::*;
		match lookup_comb {
			Parent => {
				for ele in elements.get_ref() {
					if let Some(parent) = &ele.parent() {
						if self.includes(&parent) {
							return true;
						}
					}
				}
			}
			ParentAll => {
				for ele in elements.get_ref() {
					if let Some(parent) = &ele.parent() {
						if self.includes(&parent) {
							return true;
						}
						if let Some(ancestor) = &parent.parent() {
							if self.includes(&ancestor) {
								return true;
							}
							if self.has_ele(&ancestor, comb, None) {
								return true;
							}
						}
					}
				}
			}
			Prev => {
				for ele in elements.get_ref() {
					if let Some(prev) = &ele.previous_element_sibling() {
						if self.includes(&prev) {
							return true;
						}
					}
				}
			}
			PrevAll => {
				for ele in elements.get_ref() {
					let prevs = ele.previous_element_siblings();
					for prev in prevs.get_ref() {
						if self.includes(prev) {
							return true;
						}
					}
				}
			}
			Chain => {
				for ele in elements.get_ref() {
					if self.includes(ele) {
						return true;
					}
				}
			}
			_ => panic!("Unsupported lookup combinator:{:?}", comb),
		};
		false
	}
	/// check if the ele list contains some ele
	fn includes(&self, ele: &BoxDynElement) -> bool {
		self.get_ref().iter().any(|n| ele.is(n))
	}
	/// index of
	fn index_of(&self, ele: &BoxDynElement, start_index: usize) -> Option<usize> {
		let total = self.length();
		if start_index < total {
			let nodes = self.get_ref();
			for (index, cur_ele) in nodes[start_index..].iter().enumerate() {
				if ele.is(cur_ele) {
					return Some(start_index + index);
				}
			}
		}
		None
	}
	/// check if two nodes are siblings.
	fn is_sibling(cur: &BoxDynElement, other: &BoxDynElement) -> bool {
		// just check if they have same parent.
		if let Some(parent) = cur.parent() {
			if let Some(other_parent) = other.parent() {
				return parent.is(&other_parent);
			}
		}
		false
	}

	// -------------Content API----------------
	/// pub fn `text`
	/// get the text of each element in the set
	pub fn text(&self) -> &str {
		let mut result = String::with_capacity(50);
		for ele in self.get_ref() {
			result.push_str(ele.text_content());
		}
		to_static_str(result)
	}

	/// pub fn `set_text`
	/// set each element's text to content
	pub fn set_text(&mut self, content: &str) -> &mut Self {
		for ele in self.get_mut_ref() {
			ele.set_text(content);
		}
		self
	}

	/// pub fn `html`
	/// get the first element's html
	pub fn html(&self) -> &str {
		if let Some(ele) = self.get(0) {
			return ele.inner_html();
		}
		""
	}

	/// pub fn `set_html`
	/// set each element's html to content
	pub fn set_html(&mut self, content: &str) -> &mut Self {
		for ele in self.get_mut_ref() {
			ele.set_html(content);
		}
		self
	}

	/// pub fn `outer_html`
	/// get the first element's outer html
	pub fn outer_html(&self) -> &str {
		if let Some(ele) = self.get(0) {
			return ele.outer_html();
		}
		""
	}

	/// pub fn `texts`
	/// get the text node of each element
	pub fn texts(&self, limit_depth: u32) -> Texts<'a> {
		let mut result = Texts::with_capacity(5);
		for ele in self.get_ref() {
			if let Some(text_nodes) = ele.texts(limit_depth) {
				result.get_mut_ref().extend(text_nodes);
			}
		}
		result
	}

	// ---------------Attribute API------------------
	/// pub fn `attr`
	/// get the first element's attribute value
	pub fn attr(&self, attr_name: &str) -> Option<IAttrValue> {
		if let Some(ele) = self.get(0) {
			return ele.get_attribute(attr_name);
		}
		None
	}

	/// pub fn `set_attr`
	/// set each element's attribute to `key` = attr_name, `value` = value.  
	pub fn set_attr(&mut self, attr_name: &str, value: Option<&str>) -> &mut Self {
		for ele in self.get_mut_ref() {
			ele.set_attribute(attr_name, value);
		}
		self
	}

	/// pub fn `remove_attr`
	pub fn remove_attr(&mut self, attr_name: &str) -> &mut Self {
		for ele in self.get_mut_ref() {
			ele.remove_attribute(attr_name);
		}
		self
	}

	/// pub fn `has_class`
	pub fn has_class(&self, class_name: &str) -> bool {
		let class_name = class_name.trim();
		if !class_name.is_empty() {
			let class_list = get_class_list(class_name);
			for ele in self.get_ref() {
				let class_value = ele.get_attribute(ATTR_CLASS);
				if let Some(IAttrValue::Value(cls, _)) = class_value {
					let orig_class_list = get_class_list(&cls);
					for class_name in &class_list {
						// if any of element contains the class
						if orig_class_list.contains(class_name) {
							return true;
						}
					}
				}
			}
		}
		false
	}

	/// pub fn `add_class`
	pub fn add_class(&mut self, class_name: &str) -> &mut Self {
		let class_name = class_name.trim();
		if !class_name.is_empty() {
			let class_list = get_class_list(class_name);
			for ele in self.get_mut_ref() {
				let class_value = ele.get_attribute(ATTR_CLASS);
				if let Some(IAttrValue::Value(cls, _)) = class_value {
					let mut orig_class_list = get_class_list(&cls);
					for class_name in &class_list {
						if !orig_class_list.contains(class_name) {
							orig_class_list.push(class_name);
						}
					}
					ele.set_attribute(ATTR_CLASS, Some(orig_class_list.join(" ").as_str()));
					continue;
				}
				ele.set_attribute(ATTR_CLASS, Some(class_name));
			}
		}
		self
	}
	/// pub fn `remove_class`
	pub fn remove_class(&mut self, class_name: &str) -> &mut Self {
		let class_name = class_name.trim();
		if !class_name.is_empty() {
			let class_list = get_class_list(class_name);
			for ele in self.get_mut_ref() {
				let class_value = ele.get_attribute(ATTR_CLASS);
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
						ele.set_attribute(ATTR_CLASS, Some(orig_class_list.join(" ").as_str()));
					}
				}
			}
		}
		self
	}
	/// pub fn `toggle_class`
	pub fn toggle_class(&mut self, class_name: &str) -> &mut Self {
		let class_name = class_name.trim();
		if !class_name.is_empty() {
			let class_list = get_class_list(class_name);
			let total = class_list.len();
			for ele in self.get_mut_ref() {
				let class_value = ele.get_attribute(ATTR_CLASS);
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
						ele.set_attribute(ATTR_CLASS, Some(orig_class_list.join(" ").as_str()));
					}
					continue;
				}
				ele.set_attribute(ATTR_CLASS, Some(class_name));
			}
		}
		self
	}

	// -----------------DOM API--------------
	/// pub fn `remove`
	pub fn remove(self) {
		for ele in self.into_iter() {
			if let Some(parent) = ele.parent().as_mut() {
				parent.remove_child(ele);
			}
		}
	}
	// pub fn `empty`
	pub fn empty(&mut self) -> &mut Self {
		self.set_text("");
		self
	}
	// `insert`
	fn insert(&mut self, dest: &Elements, position: &InsertPosition) -> &mut Self {
		for ele in self.get_mut_ref() {
			for inserted in dest.get_ref().iter().rev() {
				ele.insert_adjacent(position, inserted);
			}
		}
		self
	}
	/// pub fn `append`
	pub fn append(&mut self, elements: &mut Elements) -> &mut Self {
		self.insert(elements, &InsertPosition::BeforeEnd);
		self
	}
	/// pub fn `append_to`
	pub fn append_to(&mut self, elements: &mut Elements) -> &mut Self {
		elements.append(self);
		self
	}
	/// pub fn `prepend`
	pub fn prepend(&mut self, elements: &mut Elements) -> &mut Self {
		self.insert(elements, &InsertPosition::AfterBegin);
		self
	}
	/// pub fn `prepend_to`
	pub fn prepend_to(&mut self, elements: &mut Elements) -> &mut Self {
		elements.prepend(self);
		self
	}
	/// pub fn `insert_before`
	pub fn insert_before(&mut self, elements: &mut Elements) -> &mut Self {
		elements.before(self);
		self
	}
	/// pub fn `before`
	pub fn before(&mut self, elements: &mut Elements) -> &mut Self {
		// insert the elements before self
		self.insert(elements, &InsertPosition::BeforeBegin);
		self
	}
	/// pub fn `insert_after`
	pub fn insert_after(&mut self, elements: &mut Elements) -> &mut Self {
		elements.after(self);
		self
	}
	/// pub fn `after`
	pub fn after(&mut self, elements: &mut Elements) -> &mut Self {
		// insert the elements after self
		self.insert(elements, &InsertPosition::AfterEnd);
		self
	}
}

impl<'a> IntoIterator for Elements<'a> {
	type Item = BoxDynElement<'a>;
	type IntoIter = Box<dyn Iterator<Item = Self::Item> + 'a>;
	fn into_iter(self) -> Self::IntoIter {
		Box::new(self.nodes.into_iter())
	}
}

impl<'a> From<Vec<BoxDynElement<'a>>> for Elements<'a> {
	fn from(nodes: Vec<BoxDynElement<'a>>) -> Self {
		Elements { nodes }
	}
}

impl<'a> IntoIterator for Texts<'a> {
	type Item = BoxDynText<'a>;
	type IntoIter = Box<dyn Iterator<Item = Self::Item> + 'a>;
	fn into_iter(self) -> Self::IntoIter {
		Box::new(self.nodes.into_iter())
	}
}

impl<'a> From<Vec<BoxDynText<'a>>> for Texts<'a> {
	fn from(nodes: Vec<BoxDynText<'a>>) -> Self {
		Texts { nodes }
	}
}

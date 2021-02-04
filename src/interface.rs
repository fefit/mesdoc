use crate::error::Error as IError;
use crate::selector::{Combinator, QueryProcess, Selector, SelectorSegment};
use crate::utils::{get_class_list, retain_by_index, to_static_str};
use std::error::Error;
use std::rc::Rc;
use std::{any::Any, cmp::Ordering, collections::VecDeque, ops::Range};
const ATTR_CLASS: &str = "class";
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
	pub fn filter_by<'b, F>(&self, handle: F) -> Texts<'b>
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
	pub fn get(&self, index: usize) -> Option<&BoxDynElement<'a>> {
		self.get_ref().get(index)
	}
	pub(crate) fn trigger_method<F, T: Default>(&self, method: &str, selector: &str, handle: F) -> T
	where
		F: Fn(&mut Selector) -> T,
	{
		if !self.is_empty() {
			let s = selector.parse::<Selector>();
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
	/// pub fn `sort`
	pub fn sort(mut self) -> Self {
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
		// compare
		fn compare_indexs(a: &VecDeque<usize>, b: &VecDeque<usize>) -> Ordering {
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
			Ordering::Equal
		}
		// sort
		self.get_mut_ref().sort_by(|a, b| {
			let a_indexs = get_tree_indexs(a);
			let b_indexs = get_tree_indexs(b);
			compare_indexs(&a_indexs, &b_indexs)
		});
		self
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
	fn select_with_comb<'b>(&self, method: &str, selector: &str, comb: Combinator) -> Elements<'b> {
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
	fn select_with_comb_until<'b>(
		&self,
		method: &str,
		selector: &str,
		filter: &str,
		contains: bool,
		comb: Combinator,
	) -> Elements<'b> {
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
							let meet_until = cur_eles
								.filter_type_handle(method, &selector, &FilterType::Is)
								.1 > 0;
							// meet the until element, and not contains, stop before check element
							if meet_until && !contains {
								break;
							}
							// check if cur_eles filter
							let should_add = if let Some(filter) = &filter {
								// filter true
								cur_eles
									.filter_type_handle(method, filter, &FilterType::Is)
									.1 > 0
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
	// prev
	pub fn prev<'b>(&self, selector: &str) -> Elements<'b> {
		self.select_with_comb("prev", selector, Combinator::Prev)
	}
	// prev_all
	pub fn prev_all<'b>(&self, selector: &str) -> Elements<'b> {
		self.select_with_comb("prev_all", selector, Combinator::PrevAll)
	}
	// prev_until
	pub fn prev_until<'b>(&self, selector: &str, filter: &str, contains: bool) -> Elements<'b> {
		self.select_with_comb_until("prev_until", selector, filter, contains, Combinator::Prev)
	}
	// next
	pub fn next<'b>(&self, selector: &str) -> Elements<'b> {
		self.select_with_comb("next", selector, Combinator::Next)
	}
	// next_all
	pub fn next_all<'b>(&self, selector: &str) -> Elements<'b> {
		self.select_with_comb("next_all", selector, Combinator::NextAll)
	}
	// next_until
	pub fn next_until<'b>(&self, selector: &str, filter: &str, contains: bool) -> Elements<'b> {
		self.select_with_comb_until("next_until", selector, filter, contains, Combinator::Next)
	}
	// siblings
	pub fn siblings<'b>(&self, selector: &str) -> Elements<'b> {
		self.select_with_comb("siblings", selector, Combinator::Siblings)
	}
	// children
	pub fn children<'b>(&self, selector: &str) -> Elements<'b> {
		self.select_with_comb("children", selector, Combinator::Children)
	}
	// parent
	pub fn parent<'b>(&self, selector: &str) -> Elements<'b> {
		self.select_with_comb("parent", selector, Combinator::Parent)
	}
	// parents
	pub fn parents<'b>(&self, selector: &str) -> Elements<'b> {
		self.select_with_comb("parents", selector, Combinator::ParentAll)
	}
	// parents_until
	pub fn parents_until<'b>(&self, selector: &str, filter: &str, contains: bool) -> Elements<'b> {
		self.select_with_comb_until(
			"parents_until",
			selector,
			filter,
			contains,
			Combinator::Parent,
		)
	}
	// closest
	pub fn closest<'b>(&self, selector: &str) -> Elements<'b> {
		const METHOD: &str = "closest";
		let selector = selector.parse::<Selector>();
		if let Ok(selector) = selector {
			let segment = Selector::make_comb_all(Combinator::Parent);
			let lookup_selector = Selector::from_segment(segment);
			let mut result = Elements::with_capacity(self.length());
			for ele in self.get_ref() {
				let mut cur_eles = Elements::with_node(ele);
				loop {
					// find begin with self, then parents
					if cur_eles
						.filter_type_handle(METHOD, &selector, &FilterType::Is)
						.1 > 0
					{
						result.get_mut_ref().push(
							cur_eles
								.get(0)
								.expect("Elements get 0 must have when length > 0")
								.cloned(),
						);
						break;
					}
					// set the cur_eles as parent
					cur_eles = cur_eles.find_selector(&lookup_selector);
					// break if parent is none
					if cur_eles.is_empty() {
						break;
					}
				}
			}
			result
		} else {
			self.trigger_method_throw_error(METHOD, Box::new(selector.unwrap_err()));
			Elements::new()
		}
	}
	// for `find` and `select_with_comb`
	fn find_selector<'b>(&self, selector: &Selector) -> Elements<'b> {
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
							let lookup_comb = &first_comb.reverse();
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
								if tops.has_ele(ele, &lookup_comb, Some(&lookup[1..])) {
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
							result = Elements::unique(result, group);
						}
					} else {
						let group = group.unwrap_or_else(|| self.cloned());
						result = Elements::unique(result, group);
					}
				}
			}
		}
		result
	}
	// `find`
	pub fn find<'b>(&self, selector: &str) -> Elements<'b> {
		self.trigger_method("find", selector, |selector| self.find_selector(selector))
	}
	// filter_type_handle:
	//          |   `loop_group:rule groups      |     'loop_node: ele list
	// Filter   |     match one rule item        |      should loop all nodes
	// Not      |        all not matched         |      should loop all nodes
	// Is       |           all matched          |  once one ele is not matched, break the loop
	fn filter_type_handle<'b>(
		&self,
		method: &str,
		selector: &Selector,
		filter_type: &FilterType,
	) -> (Elements<'b>, usize) {
		let groups_num = selector.process.len();
		let eles = self.get_ref();
		let total = eles.len();
		let mut result = Elements::with_capacity(total);
		let mut matched_num = 0;
		let is_not = *filter_type == FilterType::Not;
		// loop for rules
		fn loop_rules(
			elements: &mut Elements,
			query: &[Vec<SelectorSegment>],
			method: &str,
			comb: Combinator,
			last_must_match: bool,
		) -> Combinator {
			let mut comb = comb;
			let mut last_must_match = last_must_match;
			// loop for the query
			for rules in query.iter().rev() {
				let first_rule = &rules[0];
				for (index, rule) in rules.iter().enumerate() {
					let find_list: Elements;
					if index == 0 {
						find_list = Elements::select_by_rule(&elements, rule, Some(&comb));
					} else {
						find_list = Elements::select_by_rule(&elements, rule, None);
					}
					if rule.0.in_cache {
						// the ele list is in cache
						let total = find_list.length();
						if total > 0 {
							let mut last_list = Elements::with_capacity(total);
							let cur_comb = if index == 0 { comb } else { rule.2 };
							// the last rule or the chain rule must match the ele list
							if !(last_must_match || cur_comb == Combinator::Chain) {
								// otherwise change the ele list to the real should matched ele list
								*elements = elements.select_with_comb(method, "", cur_comb);
								// test the find_list if in the real elements,so change the comb to chain
								comb = Combinator::Chain;
							}
							for ele in find_list.get_ref() {
								if elements.has_ele(ele, &comb, None) {
									last_list.push(ele.cloned());
								}
							}
							*elements = last_list;
						} else {
							*elements = find_list;
						}
					} else {
						*elements = find_list;
					}
					if elements.is_empty() {
						break;
					}
				}
				// change the comb into cur first rule's reverse comb.
				comb = first_rule.2.reverse();
				// the last filter rule must in ele list
				last_must_match = false;
			}
			comb
		};
		for ele in eles {
			let mut ok_nums = 0;
			'loop_group: for process in selector.process.iter() {
				let QueryProcess { query, should_in } = process;
				let mut elements = Elements::with_node(ele);
				let comb = loop_rules(&mut elements, &query, method, Combinator::Chain, true);
				// if has optimized `should_in` selectors
				if let Some(should_in) = should_in {
					if !elements.is_empty() {
						loop_rules(&mut elements, &should_in, method, comb, false);
					}
				}
				if elements.is_empty() {
					if is_not {
						// if is `not`, then the ele is not in cur group selector.
						ok_nums += 1;
					}
				} else {
					// match one of the group item
					match filter_type {
						FilterType::Filter => {
							result.push(ele.cloned());
							break 'loop_group;
						}
						FilterType::Is | FilterType::IsAll => {
							ok_nums += 1;
						}
						FilterType::Not => {}
					}
				}
			}
			// check the ele loop
			match filter_type {
				FilterType::Not => {
					if ok_nums == groups_num {
						result.push(ele.cloned());
					}
				}
				FilterType::Is => {
					// just find one matched
					if ok_nums == groups_num {
						matched_num += 1;
						// break the loop for ele
						break;
					}
				}
				FilterType::IsAll => {
					// should all matched, when find one is not matched, break the loop
					if ok_nums != groups_num {
						break;
					} else {
						matched_num += 1;
					}
				}
				_ => {}
			}
		}
		(result, matched_num)
	}
	// filter in type
	fn filter_in_handle<'b>(
		&self,
		search: &Elements,
		filter_type: FilterType,
	) -> (Elements<'b>, usize) {
		let eles = self.get_ref();
		let total = eles.len();
		let mut result = Elements::with_capacity(total);
		let mut matched_num = 0;
		match filter_type {
			FilterType::Filter => {
				for ele in eles {
					if search.includes(ele) {
						result.push(ele.cloned());
					}
				}
			}
			FilterType::Not => {
				for ele in eles {
					if !search.includes(ele) {
						result.push(ele.cloned());
					}
				}
			}
			FilterType::Is => {
				for ele in eles {
					if search.includes(ele) {
						matched_num += 1;
						break;
					}
				}
			}
			FilterType::IsAll => {
				if total <= search.length() {
					for ele in eles {
						if !search.includes(ele) {
							break;
						}
						matched_num += 1;
					}
				}
			}
		}
		(result, matched_num)
	}

	// filter
	pub fn filter<'b>(&self, selector: &str) -> Elements<'b> {
		const METHOD: &str = "filter";
		self.trigger_method(METHOD, selector, |selector| {
			self
				.filter_type_handle(METHOD, &selector, &FilterType::Filter)
				.0
		})
	}

	// filter_by
	pub fn filter_by<'b, F>(&self, handle: F) -> Elements<'b>
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
	pub fn filter_in<'b>(&self, search: &Elements) -> Elements<'b> {
		self.filter_in_handle(search, FilterType::Filter).0
	}
	// is
	pub fn is(&self, selector: &str) -> bool {
		const METHOD: &str = "is";
		self.trigger_method(METHOD, selector, |selector| {
			self.filter_type_handle(METHOD, selector, &FilterType::Is).1 > 0
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
		self.filter_in_handle(search, FilterType::Is).1 > 0
	}
	// is_all
	pub fn is_all(&self, selector: &str) -> bool {
		const METHOD: &str = "is_all";
		self.trigger_method(METHOD, selector, |selector| {
			let count = self
				.filter_type_handle(METHOD, &selector, &FilterType::IsAll)
				.1;
			count > 0 && count == self.length()
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
		let count = self.filter_in_handle(search, FilterType::IsAll).1;
		count > 0 && count == self.length()
	}
	// not
	pub fn not<'b>(&self, selector: &str) -> Elements<'b> {
		const METHOD: &str = "not";
		self.trigger_method(METHOD, selector, |selector| {
			self
				.filter_type_handle(METHOD, &selector, &FilterType::Not)
				.0
		})
	}
	// not by
	pub fn not_by<'b, F>(&self, handle: F) -> Elements<'b>
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
	// not in
	pub fn not_in<'b>(&self, search: &Elements) -> Elements<'b> {
		self.filter_in_handle(search, FilterType::Not).0
	}

	// has
	pub fn has<'b>(&self, selector: &str) -> Elements<'b> {
		const METHOD: &str = "has";
		fn loop_handle(ele: &BoxDynElement, selector: &Selector) -> bool {
			let childs = ele.children();
			if !childs.is_empty() {
				let (_, count) = childs.filter_type_handle(METHOD, selector, &FilterType::Is);
				if count > 0 {
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
	pub fn has_in<'b>(&self, search: &Elements) -> Elements<'b> {
		fn loop_handle(ele: &BoxDynElement, search: &Elements) -> bool {
			let childs = ele.children();
			if !childs.is_empty() {
				let (_, count) = childs.filter_in_handle(search, FilterType::Is);
				if count > 0 {
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

	// eq
	pub fn eq<'b>(&self, index: usize) -> Elements<'b> {
		if let Some(ele) = self.get(index) {
			Elements::with_node(ele)
		} else {
			Elements::new()
		}
	}

	// slice
	pub fn slice<'b>(&self, range: Range<usize>) -> Elements<'b> {
		let Range { start, end } = range;
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

	// unique the nodes
	fn unique<'b>(first: Elements<'b>, second: Elements<'b>) -> Elements<'b> {
		if first.is_empty() {
			return second;
		}
		// Todo: unique
		first
	}
	// find a ele and then remove it
	fn find_out<'b>(
		elements: &'b mut Elements<'a>,
		item: &BoxDynElement,
	) -> Option<BoxDynElement<'a>> {
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
	fn select_by_rule<'b>(
		elements: &Elements<'b>,
		rule_item: &SelectorSegment,
		comb: Option<&Combinator>,
	) -> Elements<'b> {
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
			}
			Children => {
				for ele in elements.get_ref() {
					let childs = ele.children();
					let match_childs = rule.apply(&childs, matched);
					if !match_childs.is_empty() {
						result.get_mut_ref().extend(match_childs);
					}
				}
			}
			Parent => {
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
			}
			NextAll => {
				for ele in elements.get_ref() {
					let nexts = ele.next_element_siblings();
					let matched_nexts = rule.apply(&nexts, matched);
					if !matched_nexts.is_empty() {
						result.get_mut_ref().extend(matched_nexts);
					}
				}
			}
			Next => {
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
				for ele in elements.get_ref() {
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
			}
			Chain => {
				result = rule.apply(&elements, matched);
			}
		};
		result
	}
	// select ele by rules
	fn select<'b>(
		elements: &Elements,
		rules: &[SelectorSegment],
		comb: Option<&Combinator>,
	) -> Elements<'b> {
		let mut elements = elements.cloned();
		for (index, rule_item) in rules.iter().enumerate() {
			let (rule, matched, cur_comb) = rule_item;
			let comb = if index == 0 {
				comb.unwrap_or(cur_comb)
			} else {
				cur_comb
			};
			if rule.in_cache {
				// in cache
				let finded = rule.apply(&elements, matched);
				if !finded.is_empty() {
					let mut result = Elements::with_capacity(finded.length());
					let lookup_comb = comb.reverse();
					for ele in finded.get_ref() {
						if elements.has_ele(ele, &lookup_comb, None) {
							result.push(ele.cloned());
						}
					}
					elements = result;
				} else {
					// set elements empty
					elements = Elements::new();
				}
			} else {
				elements = Elements::select_by_rule(&elements, rule_item, None);
			}
			if elements.is_empty() {
				break;
			}
		}
		elements
	}
	// cloned
	pub fn cloned<'b>(&'a self) -> Elements<'b> {
		let mut result = Elements::with_capacity(self.length());
		for ele in &self.nodes {
			result.push(ele.cloned());
		}
		result
	}
	// `has_ele`
	pub(crate) fn has_ele<'b>(
		&self,
		ele: &'b BoxDynElement,
		comb: &Combinator,
		lookup: Option<&'b [Vec<SelectorSegment>]>,
	) -> bool {
		let mut elements = Elements::with_node(ele);
		let mut comb = comb;
		if let Some(lookup) = lookup {
			for rules in lookup.iter().rev() {
				let finded = Elements::select(&elements, rules, Some(comb));
				if elements.is_empty() {
					return false;
				}
				comb = &rules[0].2;
				elements = finded;
			}
		}
		use Combinator::*;
		match comb {
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
	/// pub fn `text`
	pub fn text(&self) -> &str {
		let mut result = String::with_capacity(50);
		for ele in self.get_ref() {
			result.push_str(ele.text_content());
		}
		to_static_str(result)
	}
	/// pub fn `set_text`
	pub fn set_text(&mut self, content: &str) -> &mut Self {
		for ele in self.get_mut_ref() {
			ele.set_text(content);
		}
		self
	}
	/// pub fn `html`
	pub fn html(&self) -> &str {
		if let Some(ele) = self.get(0) {
			return ele.inner_html();
		}
		""
	}
	/// pub fn `set_html`
	pub fn set_html(&mut self, content: &str) -> &mut Self {
		for ele in self.get_mut_ref() {
			ele.set_html(content);
		}
		self
	}
	/// pub fn `outer_html`
	pub fn outer_html(&self) -> &str {
		if let Some(ele) = self.get(0) {
			return ele.outer_html();
		}
		""
	}
	/// pub fn `attr`
	pub fn attr(&self, attr_name: &str) -> Option<IAttrValue> {
		if let Some(ele) = self.get(0) {
			return ele.get_attribute(attr_name);
		}
		None
	}
	/// pub fn `set_attr`
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
	/// pub fn `texts`
	pub fn texts<'b>(&self, limit_depth: u32) -> Texts<'b> {
		let mut result = Texts::with_capacity(5);
		for ele in self.get_ref() {
			if let Some(text_nodes) = ele.texts(limit_depth) {
				result.get_mut_ref().extend(text_nodes);
			}
		}
		result
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
	/// pub fn `before`
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

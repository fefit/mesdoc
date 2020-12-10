use std::{cell::RefCell, rc::Rc};
pub trait NodeTrait {
	type NodeList: NodeListTrait;
	fn parent(&self) -> Result<Self::NodeList, &'static str>;
}

pub trait NodeListTrait: IntoIterator<Item = <Self as NodeListTrait>::Node> {
	type Node: NodeTrait;
	fn length(&self) -> usize;
	fn item(&self, index: usize) -> Option<Self::Node>;
}

impl<T> NodeTrait for T {
	type NodeList = Vec<T>;
	fn parent(&self) -> Result<Self::NodeList, &'static str> {
		Err("")
	}
}

impl<T> NodeListTrait for Vec<T> {
	type Node = T;
	fn length(&self) -> usize {
		0
	}
	fn item(&self, _index: usize) -> Option<Self::Node> {
		None
	}
}

pub mod matched;
pub mod rule;
use matched::*;
use std::cell::RefCell;
use std::rc::{Rc, Weak};
pub trait NodeApi {
  type List: NodeList;
  fn parent(&self) -> Result<Self::List, &'static str>;
}

pub trait NodeList: IntoIterator<Item = <Self as NodeList>::Node> {
  type Node: NodeApi;
  fn length(&self) -> usize;
  fn item(&self, index: usize) -> Option<Self::Node>;
}

pub struct Node {
  pub children: Vec<Rc<RefCell<Node>>>,
  pub parent: Weak<RefCell<Node>>,
}
pub type RNode = Rc<RefCell<Node>>;
impl NodeApi for RNode {
  type List = Vec<RNode>;
  fn parent(&self) -> Result<Self::List, &'static str> {
    let node = self
      .borrow()
      .parent
      .upgrade()
      .map_or(Vec::new(), |v| vec![Rc::clone(&v)]);
    Ok(node)
  }
}

impl NodeList for Vec<RNode> {
  type Node = RNode;
  fn length(&self) -> usize {
    0
  }
  fn item(&self, index: usize) -> Option<Self::Node> {
    Some(Rc::clone(&self[index]))
  }
}

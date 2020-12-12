use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::result::Result as OResult;
pub type Result<'a> = OResult<NodeList<'a>, &'static str>;
pub type BoxDynNode<'a> = Box<dyn NodeTrait + 'a>;
type RRC<T> = Rc<RefCell<T>>;
pub trait NodeTrait {
  fn parent(&self) -> Result;
}

#[derive(Default)]
pub struct NodeList<'a> {
  nodes: Vec<BoxDynNode<'a>>,
}

impl<'a> NodeList<'a> {
  pub fn new() -> Self {
    Default::default()
  }
  pub fn push(&mut self, node: BoxDynNode<'a>) {
    self.nodes.push(node);
  }
  pub fn with_capacity(size: usize) -> Self {
    NodeList {
      nodes: Vec::with_capacity(size),
    }
  }
  pub fn from_rrc_slice<T: 'a>(v: &[RRC<T>]) -> Self
  where
    RRC<T>: NodeTrait,
  {
    let mut nodes: Vec<BoxDynNode> = Vec::with_capacity(v.len());
    for item in v.iter() {
      nodes.push(Box::new(Rc::clone(item)) as BoxDynNode<'a>)
    }
    nodes.into()
  }
}
impl<'a> IntoIterator for NodeList<'a> {
  type Item = BoxDynNode<'a>;
  type IntoIter = Box<dyn Iterator<Item = Self::Item> + 'a>;
  fn into_iter(self) -> Self::IntoIter {
    Box::new(self.nodes.into_iter())
  }
}
pub struct Node {
  pub parent: Weak<RefCell<Node>>,
  pub children: Vec<Rc<RefCell<Node>>>,
}

impl<'a> From<Vec<BoxDynNode<'a>>> for NodeList<'a> {
  fn from(nodes: Vec<BoxDynNode<'a>>) -> Self {
    NodeList { nodes }
  }
}
impl NodeTrait for Rc<RefCell<Node>> {
  fn parent(&self) -> Result {
    let result = NodeList::from_rrc_slice(&self.borrow().children);
    Ok(result)
  }
}

pub fn init() {
  let node1 = Node {
    parent: Weak::new(),
    children: Vec::new(),
  };
  let node2 = Node {
    parent: Weak::new(),
    children: vec![Rc::new(RefCell::new(node1))],
  };
  let result = node2.children[0].parent();
}

use std::cell::RefCell;
use std::rc::Rc;
use std::result::Result as OResult;

use super::{QueryProcess, Selector};

pub type Result<'a> = OResult<NodeList<'a>, &'static str>;
pub type BoxDynNode<'a> = Box<dyn NodeTrait + 'a>;
pub enum AttrValue {
  Value(&'static str),
  Flag(bool),
}
pub enum NodeType {
  Element,
  Text,
  Comment,
  Spaces,
  Other,
}

impl NodeType {
  pub fn is_element(&self) -> bool {
    matches!(self, NodeType::Element)
  }
}
pub trait Document {
  fn get_element_by_id(id: &str) -> BoxDynNode;
}
type RRC<T> = Rc<RefCell<T>>;
pub trait NodeTrait {
  // clone a node
  fn cloned(&self) -> Box<dyn NodeTrait>;
  // tag name
  fn tag_name(&self) -> &str;
  // get node type
  fn node_type(&self) -> NodeType;
  // get node index
  fn index(&self) -> Option<usize> {
    if self.node_type().is_element() {
      let parent = self.parent();
      if let Ok(childs) = parent {
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
    }
    None
  }
  // find parents
  fn parent(&self) -> OResult<BoxDynNode, &'static str>;
  fn children(&self) -> Result;
  // attribute
  fn get_attribute(&self, name: &str) -> Option<AttrValue>;
  fn set_attribute(&mut self, value: AttrValue);
  fn has_attribute(&self, name: &str) -> bool {
    self.get_attribute(name).is_some()
  }
  // html/text
  fn html(&self) -> &str;
  fn inner_html(&self) -> &str;
  fn text_content(&self) -> &str;
  // node
  fn append_child(&mut self);
  fn remove_child(&mut self, node: BoxDynNode);
  // check if two node are the same
  fn uuid(&self) -> &str;
  fn is(&self, node: &BoxDynNode) -> bool {
    self.uuid() == node.uuid()
  }
  // owner document
}

#[derive(Default)]
pub struct NodeList<'a> {
  nodes: Vec<BoxDynNode<'a>>,
}

impl<'a> NodeList<'a> {
  pub fn new() -> Self {
    Default::default()
  }
  pub fn get_ref(&self) -> &Vec<BoxDynNode<'a>> {
    self.nodes.as_ref()
  }
  pub fn get_mut_ref(&mut self) -> &mut Vec<BoxDynNode<'a>> {
    self.nodes.as_mut()
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
  pub fn count(&self) -> usize {
    self.nodes.len()
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
  // filter some rule
  pub fn find(&self, selector: &str) -> Result {
    let selector: Selector = selector.into();
    let process = selector.process;
    let mut result = NodeList::with_capacity(5);
    for p in process {
      let QueryProcess { should_in, find } = p;
      // let first = find[0];
      // let first_rule = first;
      // let first_rule_comb = first[0].2;

      // if let Some(lookup) = should_in {}
    }
    Err("")
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

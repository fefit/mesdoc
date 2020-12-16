use std::cell::RefCell;
use std::rc::Rc;
use std::result::Result as OResult;
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
  Other,
}

impl NodeType {
  pub fn is_element(&self) -> bool {
    match self {
      NodeType::Element => true,
      _ => false,
    }
  }
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
      if parent.is_ok() {
        let childs = parent.unwrap().get(0).unwrap().children().unwrap();
        let mut index = 0;
        for node in childs {
          if node.node_type().is_element() {
            if self.is(node) {
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
  fn parent(&self) -> Result;
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
  fn is(&self, node: BoxDynNode) -> bool;
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
  pub fn get(&self, index: usize) -> Option<&BoxDynNode> {
    self.nodes.get(index)
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
  // get children
  pub fn children(&self, selector: &str) -> NodeList {
    NodeList::new()
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

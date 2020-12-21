use std::cell::RefCell;
use std::rc::Rc;
use std::result::Result as OResult;

use super::{Combinator, QueryProcess, Selector, SelectorSegment};

pub type Result<'a> = OResult<NodeList<'a>, &'static str>;
pub type SResult<'a> = OResult<Option<BoxDynNode<'a>>, &'static str>;
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
  fn parent(&self) -> SResult;
  fn children<'a, 'b>(&'a self) -> Result<'b>;
  // get all childrens
  fn childrens<'a, 'b>(&'a self) -> Result<'b> {
    let mut result = self.children()?.cloned();
    let count = result.count();
    if count > 0 {
      let mut descendants = NodeList::with_capacity(5);
      for c in &result.nodes {
        descendants.get_mut_ref().extend(c.childrens()?);
      }
      result.get_mut_ref().extend(descendants);
    }
    Ok(result)
  }
  // next sibling
  fn next_sibling(&self) -> SResult {
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
  fn next_siblings(&self) -> Result {
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
  fn previous_sibling(&self) -> SResult {
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
  fn previous_siblings(&self) -> Result {
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
      let first = &find[0];
      let first_rule_comb = &first[0].2;
      // if let Some(lookup) = should_in {}
    }
    Err("")
  }
  // select node by rules
  fn select<'b, 'r>(node_list: &'b NodeList<'r>, rules: &[SelectorSegment]) -> Result<'r> {
    let mut result = NodeList::new();
    use Combinator::*;
    for (index, r) in rules.iter().enumerate() {
      let (rule, matched, comb) = r;
      match comb {
        ChildrenAll => {
          let finded = rule.apply(node_list, matched)?;
          if rule.no_traverse {
            result = finded;
          } else {
            let mut subs = NodeList::with_capacity(2);
            for node in node_list.get_ref() {
              let sub_nodes = node.children()?;
              let nested = NodeList::select(&sub_nodes, &rules[index..index + 1])?;
              subs.get_mut_ref().extend(nested);
            }
            result.get_mut_ref().extend(subs);
          }
        }
        Combinator::Children => {
          result = rule.apply(node_list, matched)?;
        }
        Combinator::Parent => {}
        Combinator::ParentAll => {}
        Combinator::NextAll => {}
        Combinator::Next => {}
        Combinator::PrevAll => {}
        Combinator::Prev => {}
        Combinator::Chain => {}
      };
      if result.count() == 0 {
        break;
      }
    }
    Ok(result)
  }
  //
  pub fn cloned<'b>(&'a self) -> NodeList<'b> {
    let mut result = NodeList::with_capacity(self.count());
    for node in &self.nodes {
      result.push(node.cloned());
    }
    result
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

use std::cell::RefCell;
use std::rc::Rc;
use std::result::Result as OResult;

use super::{Combinator, QueryProcess, Selector, SelectorGroupsItem, SelectorSegment};

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

pub type UUID = &'static str;
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
  fn parent<'b>(&self) -> SResult<'b>;
  fn children<'b>(&self) -> Result<'b>;
  // get all childrens
  fn childrens<'b>(&self) -> Result<'b> {
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
  fn next_sibling<'b>(&self) -> SResult<'b> {
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
  fn previous_sibling<'b>(&self) -> SResult<'b> {
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
  fn uuid(&self) -> UUID;
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
      let lookup_comb = first[0].2;
      let mut group: NodeList;
      if let Some(lookup) = should_in {
        first[0].2 = Combinator::ChildrenAll;
        group = NodeList::with_capacity(5);
        // get finded
        let finded = NodeList::select(self, first, false)?;
        if finded.count() > 0 {
          let firsts = NodeList::select(self, &lookup[0], false)?;
          if firsts.count() > 0 {
            let lookup_rules = if lookup.len() > 1 {
              Some(&lookup[1..])
            } else {
              None
            };
            // remove the first
            query.remove(0);
            for node in finded.get_ref() {
              if firsts.contains(node, &lookup_comb, lookup_rules) {
                group.push(node.cloned());
              }
            }
          }
        }
      } else {
        group = self.cloned();
      }
      let mut is_empty = false;
      if group.count() > 0 && !query.is_empty() {
        for rules in query {
          group = NodeList::select(&group, &rules, false)?;
          if group.count() == 0 {
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
    let total = self.count();
    let mut uuids: Vec<&str> = Vec::with_capacity(total);
    let mut result = NodeList::with_capacity(total);
    for node in self.get_ref() {
      let uuid = node.uuid();
      if !uuids.contains(&uuid) {
        result.push(node.cloned());
        uuids.push(uuid);
      }
    }
    result
  }
  // select node by rules
  fn select<'b>(
    node_list: &'b NodeList<'a>,
    rules: &'b [SelectorSegment],
    forbid_cache: bool,
  ) -> Result<'a> {
    let mut node_list = node_list.cloned();
    use Combinator::*;
    for (index, r) in rules.iter().enumerate() {
      let (rule, matched, comb) = r;
      let cur_rule = &rules[index..index + 1];
      let mut cur_result = NodeList::with_capacity(5);
      if rule.in_cache && !forbid_cache {
        // in cache
        let finded = rule.apply(&node_list, matched)?;
        if finded.count() > 0 {
          for node in finded.get_ref() {
            if node_list.contains(node, comb, None) {
              cur_result.push(node.cloned());
            }
          }
        }
      } else {
        match comb {
          ChildrenAll => {
            for node in node_list.get_ref() {
              // get children
              let childs = node.children()?;
              // apply rule
              let match_childs = rule.apply(&childs, matched)?;
              // merge to result
              if match_childs.count() > 0 {
                cur_result.get_mut_ref().extend(match_childs);
              }
              // traversal
              let sub_childs = NodeList::select(&childs, cur_rule, true)?;
              cur_result.get_mut_ref().extend(sub_childs);
            }
          }
          Combinator::Children => {
            for node in node_list.get_ref() {
              let childs = node.children()?;
              let match_childs = rule.apply(&childs, matched)?;
              if match_childs.count() > 0 {
                cur_result.get_mut_ref().extend(match_childs);
              }
            }
          }
          Combinator::Parent => {
            for node in node_list.get_ref() {
              if let Some(pnode) = node.parent()? {
                let cur_pnode = NodeList::with_nodes(vec![pnode.cloned()]);
                let parent = rule.apply(&cur_pnode, matched)?;
                if parent.count() > 0 {
                  cur_result.get_mut_ref().extend(parent);
                }
              }
            }
          }
          Combinator::ParentAll => {
            let mut ancestors = NodeList::with_capacity(node_list.count());
            for node in node_list.get_ref() {
              if let Some(pnode) = node.parent()? {
                let cur_pnode = NodeList::with_nodes(vec![pnode.cloned()]);
                let parent = rule.apply(&cur_pnode, matched)?;
                if parent.count() > 0 {
                  cur_result.get_mut_ref().extend(parent);
                }
                if let Some(ancestor) = pnode.parent()? {
                  ancestors.push(ancestor.cloned());
                }
              }
            }
            if ancestors.count() > 0 {
              cur_result
                .get_mut_ref()
                .extend(NodeList::select(&ancestors, cur_rule, true)?);
            }
          }
          Combinator::NextAll => {
            for node in node_list.get_ref() {
              let nexts = node.next_siblings()?;
              let matched_nexts = rule.apply(&nexts, matched)?;
              if matched_nexts.count() > 0 {
                cur_result.get_mut_ref().extend(matched_nexts);
              }
            }
          }
          Combinator::Next => {
            let mut nexts = NodeList::with_capacity(node_list.count());
            for node in node_list.get_ref() {
              if let Some(next) = node.next_sibling()? {
                nexts.push(next.cloned());
              }
            }
            if nexts.count() > 0 {
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
            let mut prevs = NodeList::with_capacity(node_list.count());
            for node in node_list.get_ref() {
              if let Some(next) = node.previous_sibling()? {
                prevs.push(next.cloned());
              }
            }
            if prevs.count() > 0 {
              cur_result = rule.apply(&prevs, matched)?;
            }
          }
          Combinator::Chain => {
            cur_result = rule.apply(&node_list, matched)?;
          }
        };
      }
      node_list = cur_result.unique();
      if node_list.count() == 0 {
        break;
      }
    }
    Ok(node_list.unique())
  }
  // cloned
  pub fn cloned<'b>(&'a self) -> NodeList<'b> {
    let mut result = NodeList::with_capacity(self.count());
    for node in &self.nodes {
      result.push(node.cloned());
    }
    result
  }
  // contains
  pub fn contains<'b>(
    &self,
    node: &'b BoxDynNode,
    comb: &Combinator,
    lookup: Option<&'b [Vec<SelectorSegment>]>,
  ) -> bool {
    let mut comb = *comb;
    // if let Some(lookup) = lookup {
    //   for mut rules in lookup {}
    // }
    false
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

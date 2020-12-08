/*
*
* all: *
* id: #{identity}
* class: .{identity}
* attribute: [{identity}{rule##"(^|*~$)?=('")"##}]
*/
use crate::utils::to_static_str;
use lazy_static::lazy_static;
use regex::Regex;
use std::sync::{Arc, Mutex};
use std::{collections::HashMap, fmt::Debug};

lazy_static! {
  static ref REGEXS: Mutex<HashMap<&'static str, Arc<Regex>>> = Mutex::new(HashMap::new());
}
pub fn no_implemented(name: &str) -> ! {
  panic!("No supported Matched type '{}' found", name);
}
pub trait Matched: Send + Debug {
  fn matched(&mut self, chars: &[char]) -> Option<Vec<char>>;
  fn from_params(s: &str, _p: &str) -> Result<Box<Self>, String>
  where
    Self: Sized,
  {
    no_implemented(s);
  }
}

impl Matched for &[char] {
  fn matched(&mut self, chars: &[char]) -> Option<Vec<char>> {
    let first = chars[0];
    self.iter().find(|&&ch| ch == first).map(|ch| vec![*ch])
  }
}

impl Matched for char {
  fn matched(&mut self, chars: &[char]) -> Option<Vec<char>> {
    if *self == chars[0] {
      return Some(vec![*self]);
    }
    None
  }
}

impl Matched for Vec<char> {
  fn matched(&mut self, chars: &[char]) -> Option<Vec<char>> {
    self.as_slice().matched(chars)
  }
}
/// Identity
#[derive(Debug)]
pub struct Identity;

impl Matched for Identity {
  fn matched(&mut self, chars: &[char]) -> Option<Vec<char>> {
    let mut result: Vec<char> = Vec::with_capacity(5);
    let first = chars[0];
    if !(first.is_ascii_alphabetic() || first == '_') {
      return None;
    }
    for &c in chars {
      if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
        result.push(c);
      } else {
        break;
      }
    }
    Some(result)
  }
  // from_str
  fn from_params(s: &str, p: &str) -> Result<Box<Self>, String> {
    check_params_return(&[s, p], || Box::new(Identity))
  }
}
/// AttrKey
#[derive(Debug)]
pub struct AttrKey;

impl Matched for AttrKey {
  fn matched(&mut self, chars: &[char]) -> Option<Vec<char>> {
    let mut identity = Identity;
    let mut start_index: usize = 0;
    let mut result = Vec::with_capacity(5);
    let total_chars = chars.len();
    while let Some(matched) = Matched::matched(&mut identity, &chars[start_index..]) {
      let count = matched.len();
      let next_index = count + start_index;
      result.extend(matched);
      if total_chars > next_index {
        let next = chars[next_index];
        if next == '.' || next == ':' {
          result.push(next);
          start_index = next_index + 1;
        }
      }
    }
    if !result.is_empty() {
      return Some(result);
    }
    None
  }
  // from_params
  fn from_params(s: &str, p: &str) -> Result<Box<Self>, String> {
    check_params_return(&[s, p], || Box::new(AttrKey))
  }
}
/// Spaces
#[derive(Debug)]
pub struct Spaces(usize);

impl Matched for Spaces {
  fn matched(&mut self, chars: &[char]) -> Option<Vec<char>> {
    let mut result: Vec<char> = Vec::with_capacity(2);
    for ch in chars {
      if ch.is_ascii_whitespace() {
        result.push(*ch);
      } else {
        break;
      }
    }
    if result.len() >= self.0 {
      return Some(result);
    }
    None
  }
  fn from_params(s: &str, p: &str) -> Result<Box<Self>, String> {
    let mut min_count = 0;
    if !p.is_empty() {
      return Err(format!("Spaces not support param '{}'", p));
    }
    if !s.trim().is_empty() {
      let mut rule: [Box<dyn Matched>; 3] = [Box::new('('), Box::new(Index), Box::new(')')];
      let (result, match_all) = exec(&mut rule, s);
      println!("matched result:{:?}", result);
      if !match_all {
        return Err(format!("wrong 'Spaces{}'", s));
      }
      let index = result[1].iter().collect::<String>();
      min_count = index.parse::<usize>().map_err(|e| e.to_string())?;
      println!("min_count:{}", min_count);
    }
    Ok(Box::new(Spaces(min_count)))
  }
}

/// Index
#[derive(Debug)]
pub struct Index;

impl Matched for Index {
  fn matched(&mut self, chars: &[char]) -> Option<Vec<char>> {
    let first = chars[0];
    let mut result = Vec::with_capacity(2);
    let numbers = '0'..'9';
    if numbers.contains(&first) {
      result.push(first);
      if first != '0' {
        for ch in &chars[1..] {
          if numbers.contains(ch) {
            result.push(*ch);
          }
        }
      }
      return Some(result);
    }
    None
  }
  fn from_params(s: &str, p: &str) -> Result<Box<Self>, String> {
    check_params_return(&[s, p], || Box::new(Index))
  }
}
/// Regexp
#[derive(Debug)]
pub struct Regexp<'a> {
  pub cache: bool,
  pub context: &'a str,
  pub captures: Vec<&'a str>,
}

impl<'a> Matched for Regexp<'a> {
  fn matched(&mut self, chars: &[char]) -> Option<Vec<char>> {
    let Self { context, cache, .. } = *self;
    let content = to_static_str(chars.iter().collect::<String>());
    let rule = Regexp::get_rule(context, cache);
    if let Some(caps) = rule.captures(content) {
      let total_len = caps[0].len();
      for m in caps.iter().skip(1) {
        if let Some(m) = m {
          self.captures.push(m.as_str());
        }
      }
      return Some(chars[..total_len].to_vec());
    }
    None
  }
  fn from_params(s: &str, p: &str) -> Result<Box<Self>, String> {
    let mut cache = true;
    if !s.is_empty() {
      if s == "!" {
        cache = false;
      } else {
        return Err("Wrong param of Matched type 'regexp', just allow '!' to generate a regexp with 'cached' field falsely.".into());
      }
    }
    Ok(Box::new(Regexp {
      context: to_static_str(p.to_string()),
      cache,
      captures: vec![],
    }))
  }
}

impl<'a> Regexp<'a> {
  fn get_rule(context: &str, cache: bool) -> Arc<Regex> {
    let wrong_regex = format!("Wrong regex context '{}'", context);
    let last_context = String::from("^") + context;
    let rule = if cache {
      let mut regexs = REGEXS.lock().unwrap();
      if let Some(rule) = regexs.get(&last_context[..]) {
        Arc::clone(rule)
      } else {
        let key = &to_static_str(last_context);
        let rule = Regex::new(key).expect(&wrong_regex);
        let value = Arc::new(rule);
        let result = Arc::clone(&value);
        regexs.insert(key, value);
        result
      }
    } else {
      let key = &last_context[..];
      Arc::new(Regex::new(key).expect(&wrong_regex))
    };
    rule
  }
}

pub fn to_matched(name: &str, s: &str, p: &str) -> Result<Box<dyn Matched>, String> {
  let result: Box<dyn Matched> = match name {
    "identity" => Identity::from_params(s, p)?,
    "spaces" => Spaces::from_params(s, p)?,
    "attr_key" => AttrKey::from_params(s, p)?,
    "index" => Index::from_params(s, p)?,
    "regexp" => Regexp::from_params(s, p)?,
    _ => no_implemented(name),
  };
  Ok(result)
}

pub fn exec(queues: &mut [Box<dyn Matched>], query: &str) -> (Vec<Vec<char>>, bool) {
  let chars: Vec<char> = query.chars().collect();
  let mut start_index = 0;
  let mut result: Vec<Vec<char>> = Vec::with_capacity(queues.len());
  for item in queues {
    if let Some(matched) = item.matched(&chars[start_index..]) {
      start_index += matched.len();
      result.push(matched);
    } else {
      break;
    }
  }
  (result, start_index == chars.len())
}

pub fn check_params_return<T, F: Fn() -> Box<T>>(params: &[&str], cb: F) -> Result<Box<T>, String> {
  for &p in params {
    if !p.is_empty() {
      let all_params = params.iter().fold(String::from(""), |mut r, &s| {
        r.push_str(s);
        r
      });
      return Err(format!("Unrecognized params '{}'", all_params));
    }
  }
  Ok(cb())
}

/*
*
* all: *
* id: #{identity}
* class: .{identity}
* attribute: [{identity}{rule##"(^|*~$)?=('")"##}]
*/
use crate::utils::{chars_to_int, to_static_str};
use lazy_static::lazy_static;
use regex::Regex;
use std::sync::{Arc, Mutex};
use std::{collections::HashMap, fmt::Debug};

pub type FromParamsFn =
  Box<dyn Fn(&str, &str) -> Result<Box<dyn Pattern>, String> + Send + 'static>;
lazy_static! {
  static ref REGEXS: Mutex<HashMap<&'static str, Arc<Regex>>> = Mutex::new(HashMap::new());
  static ref PATTERNS: Mutex<HashMap<&'static str, FromParamsFn>> = Mutex::new(HashMap::new());
}
fn no_implemented(name: &str) -> ! {
  panic!("No supported Pattern type '{}' found", name);
}
pub trait Pattern: Send + Debug {
  fn matched(&mut self, chars: &[char]) -> Option<Vec<char>>;
  // get a pattern trait object
  fn from_params(s: &str, _p: &str) -> Result<Box<dyn Pattern>, String>
  where
    Self: Sized + Send + 'static,
  {
    no_implemented(s);
  }
}

impl Pattern for &[char] {
  fn matched(&mut self, chars: &[char]) -> Option<Vec<char>> {
    let total = self.len();
    if total > chars.len() {
      return None;
    }
    let mut result: Vec<char> = Vec::with_capacity(total);
    for (index, &ch) in self.iter().enumerate() {
      let cur = unsafe { chars.get_unchecked(index) };
      if ch == *cur {
        result.push(ch);
      } else {
        return None;
      }
    }
    Some(result)
  }
}

impl Pattern for char {
  fn matched(&mut self, chars: &[char]) -> Option<Vec<char>> {
    if *self == chars[0] {
      return Some(vec![*self]);
    }
    None
  }
}

impl Pattern for Vec<char> {
  fn matched(&mut self, chars: &[char]) -> Option<Vec<char>> {
    self.as_slice().matched(chars)
  }
}
/// Identity
#[derive(Debug, Default)]
pub struct Identity(&'static str);

impl Pattern for Identity {
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
    self.0 = to_static_str(result.iter().collect::<String>());
    Some(result)
  }
  // from_str
  fn from_params(s: &str, p: &str) -> Result<Box<dyn Pattern>, String>
  where
    Self: Sized,
  {
    check_params_return(&[s, p], || Box::new(Identity::default()))
  }
}
/// AttrKey
#[derive(Debug, Default)]
pub struct AttrKey(&'static str);

impl Pattern for AttrKey {
  fn matched(&mut self, chars: &[char]) -> Option<Vec<char>> {
    let mut identity = Identity::default();
    let mut start_index: usize = 0;
    let mut result = Vec::with_capacity(5);
    let total_chars = chars.len();
    while let Some(matched) = Pattern::matched(&mut identity, &chars[start_index..]) {
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
      self.0 = to_static_str(result.iter().collect::<String>());
      return Some(result);
    }
    None
  }
  // from_params
  fn from_params(s: &str, p: &str) -> Result<Box<dyn Pattern>, String> {
    check_params_return(&[s, p], || Box::new(AttrKey::default()))
  }
}
/// Spaces
#[derive(Debug, Default)]
pub struct Spaces(usize);

impl Pattern for Spaces {
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
  fn from_params(s: &str, p: &str) -> Result<Box<dyn Pattern>, String> {
    let mut min_count = 0;
    if !p.is_empty() {
      return Err(format!("Spaces not support param '{}'", p));
    }
    if !s.trim().is_empty() {
      let mut rule: [Box<dyn Pattern>; 3] =
        [Box::new('('), Box::new(Index::default()), Box::new(')')];
      let (result, _, match_all) = exec(&mut rule, s);
      if !match_all {
        return Err(format!("Wrong 'Spaces{}'", s));
      }
      min_count = chars_to_int(&result[1]).map_err(|e| e.to_string())?;
    }
    Ok(Box::new(Spaces(min_count)))
  }
}

/// Index
#[derive(Debug, Default)]
pub struct Index(usize);

impl Pattern for Index {
  fn matched(&mut self, chars: &[char]) -> Option<Vec<char>> {
    let first = chars[0];
    let mut result = Vec::with_capacity(2);
    let numbers = '0'..'9';
    println!("调用matched======>{}", first);
    if numbers.contains(&first) {
      result.push(first);
      if first != '0' {
        for ch in &chars[1..] {
          if numbers.contains(ch) {
            result.push(*ch);
          }
        }
      }
      self.0 = chars_to_int(&result).unwrap();
      println!("index======>{}", self.0);
      return Some(result);
    }
    None
  }
  fn from_params(s: &str, p: &str) -> Result<Box<dyn Pattern>, String> {
    check_params_return(&[s, p], || Box::new(Index::default()))
  }
}

/// `Nth`
/// 2n + 1/2n-1/-2n+1/0/1/
#[derive(Debug, Default)]
pub struct Nth {
  pub n: Option<isize>,
  pub index: Option<isize>,
}

impl Pattern for Nth {
  fn matched(&mut self, chars: &[char]) -> Option<Vec<char>> {
    let mut rule: RegExp = RegExp {
      cache: true,
      context: r#"^\s*(?:([-+])?([0-9]|[1-9]\d+)n\s*([+-])\s*)?([0-9]|[1-9]\d+)"#,
      captures: Vec::with_capacity(3),
    };
    if let Some(v) = Pattern::matched(&mut rule, chars) {}
    None
  }
  //
  fn from_params(s: &str, p: &str) -> Result<Box<dyn Pattern>, String> {
    check_params_return(&[s, p], || Box::new(Nth::default()))
  }
}

/// RegExp
#[derive(Debug)]
pub struct RegExp<'a> {
  pub cache: bool,
  pub context: &'a str,
  pub captures: Vec<&'a str>,
}

impl<'a> Pattern for RegExp<'a> {
  fn matched(&mut self, chars: &[char]) -> Option<Vec<char>> {
    let Self { context, cache, .. } = *self;
    let content = to_static_str(chars.iter().collect::<String>());
    let rule = RegExp::get_rule(context, cache);
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
  fn from_params(s: &str, p: &str) -> Result<Box<dyn Pattern>, String> {
    let mut cache = true;
    if !s.is_empty() {
      if s == "!" {
        cache = false;
      } else {
        return Err("Wrong param of Pattern type 'regexp', just allow '!' to generate a regexp with 'cached' field falsely.".into());
      }
    }
    Ok(Box::new(RegExp {
      context: to_static_str(p.to_string()),
      cache,
      captures: vec![],
    }))
  }
}

impl<'a> RegExp<'a> {
  pub fn get_rule(context: &str, cache: bool) -> Arc<Regex> {
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

pub fn add_pattern(name: &'static str, from_handle: FromParamsFn) {
  let mut patterns = PATTERNS.lock().unwrap();
  if patterns.get(name).is_some() {
    panic!("The pattern '{}' is already exist.", name);
  } else {
    patterns.insert(name, from_handle);
  }
}

pub(crate) fn init() {
  // add lib supported patterns
  add_pattern("identity", Box::new(Identity::from_params));
  add_pattern("spaces", Box::new(Spaces::from_params));
  add_pattern("attr_key", Box::new(AttrKey::from_params));
  add_pattern("index", Box::new(Index::from_params));
  add_pattern("regexp", Box::new(RegExp::from_params));
}

pub fn to_pattern(name: &str, s: &str, p: &str) -> Result<Box<dyn Pattern>, String> {
  let patterns = PATTERNS.lock().unwrap();
  if let Some(cb) = patterns.get(name) {
    return cb(s, p);
  }
  no_implemented(name);
}

pub fn exec(queues: &mut [Box<dyn Pattern>], query: &str) -> (Vec<Vec<char>>, usize, bool) {
  let chars: Vec<char> = query.chars().collect();
  let mut start_index = 0;
  let mut result: Vec<Vec<char>> = Vec::with_capacity(queues.len());
  for item in queues {
    if let Some(matched) = item.matched(&chars[start_index..]) {
      start_index += matched.len();
      println!("start_index:{}", start_index);
      result.push(matched);
    } else {
      break;
    }
  }
  (result, start_index, start_index == chars.len())
}

pub fn check_params_return<F: Fn() -> Box<dyn Pattern>>(
  params: &[&str],
  cb: F,
) -> Result<Box<dyn Pattern>, String> {
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

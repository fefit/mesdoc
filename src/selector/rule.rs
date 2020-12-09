use std::collections::HashMap;

use crate::selector::interface::{NodeListTrait, NodeTrait};
use crate::selector::pattern::{self, exec, to_pattern, Pattern};
use crate::utils::vec_char_to_clean_str;

pub struct Rule<'a, T> {
  queues: Vec<Box<dyn Pattern>>,
  handle: Option<Handle<T>>,
  data_handle: Option<DataHandle<'a>>,
}
type MatchedDataMap = HashMap<&'static str, &'static str>;
pub type Handle<T> = Box<dyn Fn(MatchedDataMap, T) -> Result<T, String>>;
pub type DataHandle<'a> = Box<dyn Fn(Vec<MatchedDataMap>) -> MatchedDataMap + 'a>;
// get char vec
const DEF_SIZE: usize = 2;
fn get_char_vec() -> Vec<char> {
  Vec::with_capacity(DEF_SIZE)
}

// unmatched start or end
fn panic_unmatched(ch: char, index: usize) -> ! {
  panic!(
    "Unmatched '{ch}' at index {index},you can escape it using both {ch}{ch}",
    ch = ch,
    index = index
  )
}

struct MatchedStore {
  hashs_num: usize,
  is_wait_end: bool,
  is_in_matched: bool,
  raw_params: Vec<char>,
  suf_params: Vec<char>,
  names: Vec<char>,
}

impl Default for MatchedStore {
  fn default() -> Self {
    MatchedStore {
      hashs_num: 0,
      is_wait_end: false,
      is_in_matched: false,
      raw_params: get_char_vec(),
      suf_params: get_char_vec(),
      names: get_char_vec(),
    }
  }
}
impl MatchedStore {
  fn next(&mut self) -> Result<Box<dyn Pattern>, String> {
    self.hashs_num = 0;
    self.is_in_matched = false;
    self.is_wait_end = false;
    let name = vec_char_to_clean_str(&mut self.names);
    let s = vec_char_to_clean_str(&mut self.suf_params);
    let r = vec_char_to_clean_str(&mut self.raw_params);
    to_pattern(name, s, r)
  }
}

impl<'a, T> From<&str> for Rule<'a, T> {
  /// generate a rule from string.
  fn from(content: &str) -> Self {
    const ANCHOR_CHAR: char = '\0';
    const START_CHAR: char = '{';
    const END_CHAR: char = '}';
    let mut prev_char = ANCHOR_CHAR;
    let mut store: MatchedStore = Default::default();
    let mut raw_chars = get_char_vec();
    let mut queues: Vec<Box<dyn Pattern>> = Vec::with_capacity(DEF_SIZE);
    let mut is_matched_finish = false;
    let mut index: usize = 0;
    for ch in content.chars() {
      index += 1;
      let is_prev_matched_finish = if is_matched_finish {
        is_matched_finish = false;
        true
      } else {
        false
      };
      if store.is_wait_end {
        if ch.is_ascii_whitespace() {
          continue;
        }
        if ch == END_CHAR {
          is_matched_finish = true;
        } else {
          panic!(
            "Unexpect end of Pattern type '{}' at index {}, expect '{}' but found '{}'",
            vec_char_to_clean_str(&mut store.names),
            index - 1,
            END_CHAR,
            ch
          );
        }
      } else if !store.is_in_matched {
        // when not in matched
        if prev_char == START_CHAR {
          // translate '{'
          if ch == START_CHAR {
            prev_char = ANCHOR_CHAR;
            continue;
          } else {
            store.names.push(ch);
            store.is_in_matched = true;
            // remove the '{'
            raw_chars.pop();
            if !raw_chars.is_empty() {
              queues.push(Box::new(raw_chars.clone()));
              raw_chars.clear();
            }
          }
        } else if prev_char == END_CHAR {
          if is_prev_matched_finish {
            // is just end of the Pattern type.
            raw_chars.push(ch);
          } else if ch == END_CHAR {
            // translate end char '}'
            prev_char = ANCHOR_CHAR;
            continue;
          } else {
            // panic no matched
            panic_unmatched(END_CHAR, index - 2);
          }
        } else {
          raw_chars.push(ch);
        }
      } else if !store.raw_params.is_empty() {
        // in Pattern's raw params ##gfh#def##
        if ch == '#' {
          let leave_count = store.hashs_num - 1;
          if leave_count == 0 {
            // only one hash
            store.is_wait_end = true;
          } else {
            let raw_len = store.raw_params.len();
            let last_index = raw_len - leave_count;
            if last_index > 0 {
              store.is_wait_end = store.raw_params[last_index..]
                .iter()
                .filter(|&&ch| ch == '#')
                .count()
                == leave_count;
              if store.is_wait_end {
                store.raw_params.truncate(last_index);
              }
            }
          }
          if !store.is_wait_end {
            store.raw_params.push(ch);
          }
        } else {
          // in raw params
          store.raw_params.push(ch);
        }
      } else {
        // in suf_params or names
        if ch == '}' {
          if store.hashs_num > 0 {
            panic!("Uncomplete raw params: ''");
          }
          is_matched_finish = true;
        } else if ch == '#' {
          // in hashs
          store.hashs_num += 1;
        } else {
          // in names or suf_params
          if prev_char == '#' {
            // in raw params now
            store.raw_params.push(ch);
          } else {
            // check if in suf_params, or character is not a name's character
            if !store.suf_params.is_empty() || !(ch.is_ascii_alphanumeric() || ch == '_') {
              store.suf_params.push(ch);
            } else {
              store.names.push(ch);
            }
          }
        }
      }
      if is_matched_finish {
        match store.next() {
          Ok(queue) => queues.push(queue),
          Err(reason) => panic!(reason),
        };
      }
      prev_char = ch;
    }
    // not end
    if store.is_wait_end || store.is_in_matched {
      panic!(format!(
        "The Mathed type '{}' is not complete",
        store.names.iter().collect::<String>()
      ));
    }
    if prev_char == START_CHAR || (prev_char == END_CHAR && !is_matched_finish) {
      panic_unmatched(prev_char, index - 1);
    }
    if !raw_chars.is_empty() {
      queues.push(Box::new(raw_chars));
    }
    Rule {
      queues,
      handle: None,
      data_handle: None,
    }
  }
}

impl<'a, T: NodeListTrait> Rule<'a, T>
where
  T: NodeListTrait,
{
  pub fn exec(&mut self, query: &str) {
    let (result, matched_len, _) = exec(&self.queues, query);
    println!("result is ==> {:?}", result);
  }
  pub fn set_handle<'b>(&mut self, fields: &'b [&'static str], handle: Handle<T>) -> &mut Self
  where
    'a: 'b,
  {
    let data_handle = Box::new(|data: Vec<MatchedDataMap>| -> MatchedDataMap {
      let mut result = HashMap::with_capacity(5);
      for v in fields {}
      result
    });
    self.data_handle = Some(data_handle);
    self.handle = Some(handle);
    self
  }
}

pub fn init() {
  pattern::init();
}

use crate::parser::matched::{to_matched, Matched};
use crate::utils::vec_char_to_clean_str;

#[derive(Debug)]
pub struct Rule {
  pub queues: Vec<Box<dyn Matched>>,
}
const DEF_SIZE: usize = 2;
fn get_char_vec() -> Vec<char> {
  Vec::with_capacity(DEF_SIZE)
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
  fn next(&mut self) -> Result<Box<dyn Matched>, String> {
    self.hashs_num = 0;
    self.is_in_matched = false;
    self.is_wait_end = false;
    let name = vec_char_to_clean_str(&mut self.names);
    let s = vec_char_to_clean_str(&mut self.suf_params);
    let r = vec_char_to_clean_str(&mut self.raw_params);
    println!("name:{}, s:{}, r:{}", name, s, r);
    to_matched(name, s, r)
  }
}

impl From<&str> for Rule {
  fn from(content: &str) -> Self {
    // :nth-child({spaces}{index}{spaces})
    let mut prev_char = '\0';
    let mut store: MatchedStore = Default::default();
    let mut raw_chars = get_char_vec();
    let mut queues: Vec<Box<dyn Matched>> = Vec::with_capacity(DEF_SIZE);
    let mut is_matched_finish = false;
    for ch in content.chars() {
      if store.is_wait_end {
        if ch.is_ascii_whitespace() {
          continue;
        }
        if ch == '}' {
          is_matched_finish = true;
        } else {
          panic!("wrong end");
        }
      } else if !store.is_in_matched {
        // when not in matched
        if prev_char == '{' {
          // translate '{'
          if ch == '{' {
            prev_char = '\0';
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
        } else {
          // push to raw_chars
          raw_chars.push(ch);
        }
      } else if !store.raw_params.is_empty() {
        // in Matched's raw params ##gfh#def##
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
        println!("finished====>{}", ch);
        match store.next() {
          Ok(queue) => queues.push(queue),
          Err(reason) => panic!(reason),
        };
        is_matched_finish = false;
      }
      prev_char = ch;
      println!("ch ------>{}", ch);
    }
    if store.is_wait_end || store.is_in_matched {
      panic!(format!(
        "The Mathed type '{}' is not complete",
        store.names.iter().collect::<String>()
      ));
    }
    if !raw_chars.is_empty() {
      queues.push(Box::new(raw_chars));
    }
    println!("last===>, queues:{:?}", queues);
    Rule { queues }
  }
}

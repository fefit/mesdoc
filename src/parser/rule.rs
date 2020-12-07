use crate::parser::matched::Matched;
pub struct Rule {
  pub queues: Vec<Box<dyn Matched>>,
}
const DEF_SIZE: usize = 2;
fn get_char_vec() -> Vec<char> {
  Vec::with_capacity(DEF_SIZE)
}

struct ParseStatusData {
  hashs_num: usize,
  is_wait_end: bool,
  is_in_matched: bool,
  raw_params: Vec<char>,
  suf_params: Vec<char>,
  names: Vec<char>,
}

impl Default for ParseStatusData {
  fn default() -> Self {
    ParseStatusData {
      hashs_num: 0,
      is_wait_end: false,
      is_in_matched: false,
      raw_params: get_char_vec(),
      suf_params: get_char_vec(),
      names: get_char_vec(),
    }
  }
}
impl ParseStatusData {
  fn reset(&mut self) {
    self.hashs_num = 0;
    self.is_in_matched = false;
    self.is_wait_end = false;
    self.raw_params.clear();
    self.suf_params.clear();
    self.names.clear();
  }
}

impl From<&str> for Rule {
  fn from(content: &str) -> Self {
    // :nth-child({spaces}{index}{spaces})
    let mut prev_char = '\0';
    let mut status: ParseStatusData = Default::default();
    let mut raw_chars = get_char_vec();
    let mut queues: Vec<Box<dyn Matched>> = Vec::with_capacity(DEF_SIZE);
    let ParseStatusData {
      hashs_num,
      is_in_matched,
      is_wait_end,
      ref mut raw_params,
      ref mut suf_params,
      ref mut names,
    } = status;
    for ch in content.chars() {
      if is_wait_end {
        if ch.is_ascii_whitespace() {
          continue;
        }
        if ch == '}' {
          status.reset();
        } else {
          panic!("wrong end");
        }
      }
      if !is_in_matched {
        // when not in matched
        if prev_char == '{' {
          // translate '{'
          if ch == '{' {
            prev_char = '\0';
            continue;
          } else {
            names.push(ch);
            status.is_in_matched = true;
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
      } else if !raw_params.is_empty() {
        // in Matched's raw params ##gfh#def##
        if ch == '#' {
          let leave_count = hashs_num - 1;
          if leave_count == 0 {
            // only one hash
            status.is_wait_end = true;
          } else {
            let raw_len = raw_params.len();
            let last_index = raw_len - leave_count;
            if last_index > 0 {
              status.is_wait_end = raw_params[last_index..]
                .iter()
                .filter(|&&ch| ch == '#')
                .count()
                == leave_count;
              if status.is_wait_end {
                raw_params.truncate(last_index);
              }
            }
          }
          if !status.is_wait_end {
            raw_params.push(ch);
          }
        } else {
          // in raw params
          raw_params.push(ch);
        }
      } else {
        // in suf_params or names
        if ch == '}' {
          if hashs_num > 0 {
            panic!("Uncomplete raw params: ''");
          }
          continue;
        }
        if ch == '#' {
          // in hashs
          status.hashs_num += 1;
        } else {
          // in names or suf_params
          if prev_char == '#' {
            // in raw params now
            raw_params.push(ch);
          } else {
            // check if in suf_params, or character is not a name's character
            if !suf_params.is_empty() || !ch.is_ascii_alphanumeric() {
              suf_params.push(ch);
            } else {
              names.push(ch);
            }
          }
        }
      }
      prev_char = ch;
    }
    Rule { queues }
  }
}

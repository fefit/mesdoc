/*
*
* all: *
* id: #{identity}
* class: .{identity}
* attribute: [{identity}{rule##"(^|*~$)?=('")"##}]
*/
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::sync::Mutex;
lazy_static! {
    static ref MATCHED_TYPES: Mutex<HashMap<&'static str, Box<dyn Matched>>> =
        Mutex::new(HashMap::new());
    static ref REGEXS: Mutex<HashMap<&'static str, Regex>> = Mutex::new(HashMap::new());
}
pub trait Matched: Send {
    fn matched(&mut self, chars: &[char]) -> Option<Vec<char>>;
}

impl Matched for &[char] {
    fn matched(&mut self, chars: &[char]) -> Option<Vec<char>> {
        self.iter().find(|&&ch| ch == chars[0]).map(|ch| vec![*ch])
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
}
/// AttrKey
pub struct AttrKey;

impl Matched for AttrKey {
    fn matched(&mut self, chars: &[char]) -> Option<Vec<char>> {
        let identity = Identity;
        let mut start_index: usize = 0;
        let mut result = Vec::with_capacity(5);
        let total_chars = chars.len();
        while let Some(matched) = identity.matched(&chars[start_index..]) {
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
}
/// Spaces
pub struct Spaces;

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
        if !result.is_empty() {
            return Some(result);
        }
        None
    }
}
/// Rule
pub struct Rule {
    pub cache: bool,
    pub context: &'static str,
    pub captures: Vec<&'static str>,
}

impl Matched for Rule {
    fn matched(&mut self, chars: &[char]) -> Option<Vec<char>> {
        let Self { context, cache, .. } = *self;
        let wrong_regex = format!("wrong regex context '{}'", context);
        let last_context = (String::from("^") + context).as_str();
        let rule: &Regex = if cache {
            let regexs = REGEXS.lock().unwrap();
            if let Some(rule) = regexs.get(last_context) {
                rule
            } else {
                let rule = Regex::new(last_context).expect(&wrong_regex);
                regexs.insert(last_context, rule);
                &rule
            }
        } else {
            &Regex::new(last_context).expect(&wrong_regex)
        };
        let content = chars.iter().collect::<String>();
        if let Some(caps) = rule.captures(&content) {
            let total_len = caps[0].len();
            for m in caps.iter().skip(1) {
                if let Some(m) = m {
                    self.captures.push(m.as_str());
                }
            }
            return Some(
                chars[..total_len]
                    .iter()
                    .map(|ch| *ch)
                    .collect::<Vec<char>>(),
            );
        }
        None
    }
}
/// Index
pub struct Index(usize);

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
}

pub fn add_matched_type<T: Matched>(name: &'static str, cur_type: T) {}

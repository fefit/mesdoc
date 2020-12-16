pub mod interface;
pub mod pattern;
pub mod rule;

use pattern::Matched;
use rule::{Rule, RULES};
pub struct Selector<'a> {
	pub groups: Vec<Vec<(&'a Rule, Vec<Matched>)>>,
}

impl<'a> Selector<'a> {
	fn new() -> Self {
		Selector {
			groups: Vec::with_capacity(1),
		}
	}
}

impl<'a> From<&str> for Selector<'a> {
	fn from(selector: &str) -> Self {
		let rules = RULES.lock().unwrap();
		let chars: Vec<char> = selector.chars().collect();
		let total_len = chars.len();
		let mut index: usize = 0;
		if total_len > 0 {
			while index < total_len - 1 {
				let mut finded = false;
				for r in rules.iter() {
					let (matched, len) = r.exec(&chars[index..]);
					if len > 0 {
						index += len;
						finded = true;
						break;
					}
				}
				if !finded {
					break;
				}
			}
		}
		Selector::new()
	}
}

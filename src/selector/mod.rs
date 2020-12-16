pub mod interface;
pub mod pattern;
pub mod rule;

use pattern::Matched;
use rule::{Rule, RULES};
pub struct Selector<'a> {
	pub groups: Vec<Vec<(&'a Rule, Vec<Matched>)>>,
}

impl<'a> Selector<'a> {}

impl<'a> From<&str> for Selector<'a> {
	fn from(selector: &str) -> Self {}
}

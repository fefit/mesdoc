#![allow(clippy::or_fun_call)]
use ntree::selector::Selector;
use ntree::{self, rules};

fn main() {
	ntree::init();
	let q: Selector = "p:first-of-type".into();
	println!("q is {:?}", q);
}

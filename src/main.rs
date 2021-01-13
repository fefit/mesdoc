#![allow(clippy::or_fun_call)]
use ntree::selector::Selector;
use ntree::{self, rules};

fn main() {
	ntree::init();
	let q: Selector = "p:nth-child(n - 1)".into();
	println!("q is {:?}", q);
}

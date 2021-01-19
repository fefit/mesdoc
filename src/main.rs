#![allow(clippy::or_fun_call)]
use ntree::selector::{Combinator, Selector};
use ntree::{self, rules};

fn main() {
	ntree::init();
	let mut q: Selector = "p:first-of-type".into();
	// q.head_combinator(Combinator::Children);
	println!("q is {:?}", q);
	let q2: Selector = "> * > p:first-of-type".into();
	println!("{:?}", q2);
}

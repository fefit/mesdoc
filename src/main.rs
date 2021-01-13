#![allow(clippy::or_fun_call)]
use ntree::{self, rules};
use ntree::selector::Selector;

fn main() {
  ntree::init();
  let q: Selector = "p:nth-child(2n+1)".into();
  println!("q is {:?}", q);
}

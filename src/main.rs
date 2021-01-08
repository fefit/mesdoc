#![allow(clippy::or_fun_call)]
use ntree::rules;
use ntree::selector::Selector;

fn main() {
  rules::init();
  let q: Selector = ".haha p > input[name='name'] + :first-child[readonly] #idd > *.a".into();
  println!("q is {:?}", q);
}

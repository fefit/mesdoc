#![allow(clippy::or_fun_call)]
use gotdom::rules;
use gotdom::selector::Selector;

fn main() {
  rules::init();
  let q: Selector = ".haha #abc > input[name='name'] + :first-child[readonly] > *.a".into();
  println!("q is {:?}", q);
}

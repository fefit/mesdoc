#![allow(clippy::or_fun_call)]
use gotdom::rules;
use gotdom::selector::Selector;

fn main() {
  rules::init();
  let q: Selector = "#haha > input[name='name'] + :first-child[readonly]".into();
  println!("q is {:?}", q);
}

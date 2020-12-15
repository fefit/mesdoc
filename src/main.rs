#![allow(clippy::or_fun_call)]
use gotdom::rules;
use gotdom::selector::{
  interface::NodeList,
  rule::{self, Rule},
};
use regex::Regex;

fn main() {
  rules::init();
  let rule = Rule::add(
    r##"[{spaces}{attr_key}{spaces}{regexp#([~|^$*]?)=\s*(?:"((?:\\?+.)*?)"|'((?:\\?+.)*?)'|([^\s'"<>/=`]+))#}{spaces}]"##,
    vec![("attr_key", 0), ("regexp", 0)],
    Box::new(|nodes, params, count| {
      let attr_key = Rule::param(&params, "attr_key").expect("The 'class' selector is not correct");
      let attr_value = Rule::param(&params, ("regexp", 0, "2"))
        .or(Rule::param(&params, ("regexp", 0, "3")))
        .or(Rule::param(&params, ("regexp", 0, "4")));
      println!("key is {},{:?}", attr_key, attr_value);
      Ok(nodes)
    }),
  );
  let node_list = NodeList::new();
  rule.exec(node_list, r#"[class =abc]"#);
  let re =
    Regex::new(r#"([~|^$*]?)=\s*(?:"((?:\\?+.)*?)"|'((?:\\?+.)*?)'|([^\s'"<>/=`]+))"#).unwrap();
  let caps = re.captures(r#"^="aaa""#).unwrap();
  println!("caps is:{:?}", caps);
  // rule::init();
  // let mut rule = "{identity?}:nth-child({spaces}{nth}{spaces})".into();

  // let rule: Rule = "[{{spaces}}{attr_key}{regexp#abc#}]".into();
  // print!("rule {:?}", rule);
}

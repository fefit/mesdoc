#![allow(clippy::or_fun_call)]

use crate::interface::{BoxDynElement, IAttrValue};
use crate::selector::rule::{Matcher, MatcherData};
use crate::selector::rule::{Rule, RuleDefItem, RuleItem};

pub fn init(rules: &mut Vec<RuleItem>) {
	let rule = RuleDefItem(
		"attr",
		r##"[{spaces}{attr_key}{spaces}{regexp#(?:([*^$~|!]?)=\s*(?:'((?:\\?+.)*?)'|([^\s\]'"<>/=`]+)|"((?:\\?+.)*?)"))?#}{spaces}]"##,
		10,
		vec![("attr_key", 0), ("regexp", 0)],
		Box::new(|data: MatcherData| {
			let attr_key =
				Rule::param(&data, "attr_key").expect("The attribute selector's key is not correct");
			let attr_value = Rule::param(&data, ("regexp", 0, "2"))
				.or_else(|| Rule::param(&data, ("regexp", 0, "3")))
				.or_else(|| Rule::param(&data, ("regexp", 0, "4")));
			let handle: Box<dyn Fn(Option<IAttrValue>) -> bool> = if let Some(attr_value) = attr_value {
				if attr_value.is_empty() {
					// empty attribute value
					Box::new(|_val: Option<IAttrValue>| false)
				} else {
					match Rule::param(&data, ("regexp", 0, "1")).unwrap_or("") {
						"^" => Box::new(move |val: Option<IAttrValue>| match val {
							Some(IAttrValue::Value(v, _)) => v.starts_with(attr_value),
							_ => false,
						}),
						"$" => Box::new(move |val: Option<IAttrValue>| match val {
							Some(IAttrValue::Value(v, _)) => v.ends_with(attr_value),
							_ => false,
						}),
						"*" => Box::new(move |val: Option<IAttrValue>| match val {
							Some(IAttrValue::Value(v, _)) => v.contains(attr_value),
							_ => false,
						}),
						"|" => Box::new(move |val: Option<IAttrValue>| match val {
							Some(IAttrValue::Value(v, _)) => {
								if v == attr_value {
									return true;
								}
								let attr_value: String = format!("{}-", attr_value);
								v.starts_with(&attr_value)
							}
							_ => false,
						}),
						"~" => Box::new(move |val: Option<IAttrValue>| match val {
							Some(IAttrValue::Value(v, _)) => {
								let split_v = v.split_ascii_whitespace();
								for v in split_v {
									if v == attr_value {
										return true;
									}
								}
								false
							}
							_ => false,
						}),
						"!" => Box::new(move |val: Option<IAttrValue>| match val {
							Some(IAttrValue::Value(v, _)) => attr_value != v,
							_ => false,
						}),
						_ => Box::new(move |val: Option<IAttrValue>| match val {
							Some(IAttrValue::Value(v, _)) => v == attr_value,
							_ => false,
						}),
					}
				}
			} else {
				Box::new(|val: Option<IAttrValue>| val.is_some())
			};
			Matcher {
				one_handle: Some(Box::new(move |ele: &BoxDynElement, _| {
					let val = ele.get_attribute(attr_key);
					handle(val)
				})),
				..Default::default()
			}
		}),
	);
	rules.push(rule.into());
}

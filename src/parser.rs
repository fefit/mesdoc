/*
*
* all: *
* id: #{identity}
* class: .{identity}
* attribute: [{identity}{rule##"(^|*~$)?=('")"##}]
*/
///
#![feature(min_const_generics)]
trait Matched {
  fn matched(&self, chars: &[char]) -> usize;
}

impl<const N: usize> Matched for [char; N] {
  fn matched(&self, chars: &[char]) -> usize {
    if chars.len() > 0 {
      return self.contains(chars[0]) as usize;
    }
    0
  }
}


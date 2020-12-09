use std::error::Error;

pub fn to_static_str(content: String) -> &'static str {
  Box::leak(content.into_boxed_str())
}

pub fn vec_char_to_clean_str(v: &mut Vec<char>) -> &'static str {
  to_static_str(v.drain(..).collect::<String>())
}

pub fn chars_to_int(v: &[char]) -> Result<usize, Box<dyn Error>> {
  let index = v.iter().collect::<String>();
  let index = index.parse::<usize>()?;
  Ok(index)
}

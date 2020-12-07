pub fn to_static_str(content: String) -> &'static str {
    Box::leak(content.into_boxed_str())
}

pub fn vec_char_to_clean_str(v: &mut Vec<char>) -> &'static str {
    to_static_str(v.drain(..).collect::<String>())
}

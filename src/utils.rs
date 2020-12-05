pub fn to_static_str(content: String) -> &'static str {
    Box::leak(content.into_boxed_str())
}

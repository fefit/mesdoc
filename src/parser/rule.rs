use crate::parser::matched::Matched;
pub struct Rule {
    pub queues: Vec<Box<dyn Matched>>,
}

impl From<&str> for Rule {
    fn from(content: &str) -> Self {}
}

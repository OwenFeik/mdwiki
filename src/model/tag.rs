#[derive(Debug, PartialEq, Eq)]
pub struct Tag(Box<str>);

impl Tag {
    fn new(tag: &str) -> Self {
        Self(tag.into())
    }
}

impl From<&str> for Tag {
    fn from(value: &str) -> Self {
        Tag::new(value)
    }
}

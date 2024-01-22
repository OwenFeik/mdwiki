#[derive(Debug, Hash, PartialEq, Eq)]
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

impl AsRef<str> for Tag {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

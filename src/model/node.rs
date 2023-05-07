#[derive(Debug, Eq, PartialEq)]
pub enum Style {
    Bold,
    Italic,
    Strikethrough,
    Underline
}

#[derive(Debug, Eq, PartialEq)]
pub enum Node {
    Style(Style, Vec<Node>),
    Document(Vec<Node>),
    Heading(Vec<Node>),
    Italic(Vec<Node>),
    Text(String),
}

impl Node {
    pub fn text(text: &str) -> Self {
        Self::Text(text.trim().to_string())
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Style {
    Bold,
    Italic,
    Strikethrough,
    Underline,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Node {
    Empty,
    Document(Vec<Node>),
    Heading(Vec<Node>),
    Item(Vec<Node>),
    List(Vec<Node>),
    Style(Style, Vec<Node>),
    Text(String),
}

impl Node {
    pub fn text(text: &str) -> Self {
        Self::Text(text.trim().to_string())
    }

    pub fn add_text(&mut self, text: &str) {
        match self {
            Node::Empty => (),
            Node::Style(_, children)
            | Node::Document(children)
            | Node::Heading(children)
            | Node::Item(children)
            | Node::List(children) => children.push(Node::text(text)),
            Node::Text(string) => {
                if !string.is_empty() {
                    string.push(' ');
                }
                string.push_str(text.trim());
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Node::Empty => true,
            Node::Style(_, children)
            | Node::Document(children)
            | Node::Heading(children)
            | Node::Item(children)
            | Node::List(children) => !children.iter().any(|n| !n.is_empty()),
            Node::Text(string) => string.trim().is_empty(),
        }
    }
}

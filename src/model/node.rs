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
    Image(String, String),
    Item(Vec<Node>),
    Link(String, String),
    List(Vec<Node>),
    Style(Style, Vec<Node>),
    Text(String),
}

impl Node {
    pub fn image(alt: &str, url: &str) -> Self {
        Self::Image(String::from(alt.trim()), String::from(url.trim()))
    }

    pub fn link(text: &str, url: &str) -> Self {
        Self::Link(String::from(text.trim()), String::from(url.trim()))
    }

    pub fn text(text: &str) -> Self {
        Self::Text(text.trim().to_string())
    }

    pub fn add_text(&mut self, text: &str) {
        match self {
            Node::Empty | Node::Image(..) | Node::Link(..) => (),
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
            Node::Image(_, url) => url.is_empty(),
            Node::Link(text, _) => text.is_empty(),
            Node::Style(_, children)
            | Node::Document(children)
            | Node::Heading(children)
            | Node::Item(children)
            | Node::List(children) => !children.iter().any(|n| !n.is_empty()),
            Node::Text(string) => string.trim().is_empty(),
        }
    }
}

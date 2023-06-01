#[derive(Debug, Eq, PartialEq)]
pub enum Style {
    Bold,
    Italic,
    Strikethrough,
}

pub const HEADING_MAX_LEVEL: u8 = 6;

#[derive(Debug, Eq, PartialEq)]
pub enum Node {
    Empty,
    Code(String),                      // (code)
    Codeblock(Option<String>, String), // (lang, code)
    Document(Vec<Node>),               // (children)
    Heading(u8, Vec<Node>),            // (type, children)
    Image(String, String),             // (text, url)
    Item(Vec<Node>),                   // (children)
    Link(String, String),              // (text, url)
    List(Vec<Node>),                   // (children)
    Style(Style, Vec<Node>),           // (style, children)
    Text(String),                      // (text)
}

impl Node {
    pub fn code(code: &str) -> Self {
        Self::Code(code.trim().to_string())
    }

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
            | Node::Heading(_, children)
            | Node::Item(children)
            | Node::List(children) => children.push(Node::text(text)),
            Node::Code(string) | Node::Codeblock(_, string) | Node::Text(string) => {
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
            | Node::Heading(_, children)
            | Node::Item(children)
            | Node::List(children) => !children.iter().any(|n| !n.is_empty()),
            Node::Code(string) | Node::Codeblock(_, string) | Node::Text(string) => {
                string.trim().is_empty()
            }
        }
    }
}

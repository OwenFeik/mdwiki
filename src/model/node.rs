use std::collections::HashMap;

#[derive(Debug, Eq, PartialEq)]
pub enum Style {
    Bold,
    Italic,
    Strikethrough,
}

pub const HEADING_MAX_LEVEL: u8 = 6;

pub type Attrs = HashMap<String, String>;

#[derive(Debug, Eq, PartialEq)]
pub struct Node {
    attributes: Attrs,
    element: El,
}

impl Node {
    fn new(element: El) -> Self {
        Self {
            attributes: HashMap::new(),
            element,
        }
    }

    pub fn code(code: &str) -> Self {
        Self::new(El::Code(code.trim().to_string()))
    }

    pub fn codeblock(lang: Option<&str>, code: &str) -> Self {
        Self::new(El::Codeblock(
            lang.map(String::from),
            String::from(code.trim()),
        ))
    }

    pub fn image(alt: &str, url: &str) -> Self {
        Self::new(El::Image(
            String::from(alt.trim()),
            String::from(url.trim()),
        ))
    }

    pub fn link(text: &str, url: &str) -> Self {
        Self::new(El::Link(
            String::from(text.trim()),
            String::from(url.trim()),
        ))
    }

    pub fn text(text: &str) -> Self {
        Self::new(El::Text(text.trim().to_string()))
    }

    pub fn style(style: Style, children: Vec<Node>) -> Self {
        Self::new(El::Style(style, children))
    }

    pub fn heading(size: u8, children: Vec<Node>) -> Self {
        Self::new(El::Heading(size, children))
    }

    pub fn item(children: Vec<Node>) -> Self {
        Self::new(El::Item(children))
    }

    /// Create a <ul> from a series of nodes. Each node will be wrapped in a
    /// <li>, if it is not already.
    pub fn list(children: Vec<Node>) -> Self {
        let items: Vec<Node> = children
            .into_iter()
            .map(|n| match n.el() {
                El::Item(_) => n,
                _ => Self::item(vec![n]),
            })
            .collect();
        Self::new(El::List(items))
    }

    pub fn table(children: Vec<Vec<Vec<Node>>>) -> Self {
        Self::new(El::Table(children))
    }

    pub fn span(children: Vec<Node>) -> Self {
        Self::new(El::Span(children))
    }

    pub fn empty() -> Self {
        Self::new(El::Empty)
    }

    pub fn details(summary: Vec<Node>, details: Vec<Node>) -> Self {
        Self::new(El::Details(summary, details))
    }

    pub fn attr(&mut self, key: &str, value: &str) {
        self.attributes
            .insert(key.trim().to_string(), value.trim().to_string());
    }

    pub fn with_attr(mut self, key: &str, value: &str) -> Self {
        self.attr(key, value);
        self
    }

    pub fn attrs(&self) -> &Attrs {
        &self.attributes
    }

    pub fn el(&self) -> &El {
        &self.element
    }

    pub fn el_mut(&mut self) -> &mut El {
        &mut self.element
    }

    pub fn add_text(&mut self, text: &str) {
        self.element.add_text(text);
    }

    pub fn is_empty(&self) -> bool {
        self.element.is_empty()
    }

    pub fn el_text(&self) -> Option<&str> {
        match &self.element {
            El::Empty
            | El::Span(_)
            | El::Details(_, _)
            | El::Item(_)
            | El::List(_)
            | El::Style(_, _)
            | El::Table(_)
            | El::Heading(_, _) => None,
            El::Code(text)
            | El::Codeblock(_, text)
            | El::Image(text, _)
            | El::Link(text, _)
            | El::Text(text) => Some(text),
        }
    }

    pub fn el_url(&self) -> Option<&str> {
        match &self.element {
            El::Image(_, url) | El::Link(_, url) => Some(url),
            _ => None,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum El {
    Empty,
    Span(Vec<Node>),                   // (children)
    Code(String),                      // (code)
    Codeblock(Option<String>, String), // (lang, code)
    Details(Vec<Node>, Vec<Node>),     // (summary, details)
    Heading(u8, Vec<Node>),            // (type, children)
    Image(String, String),             // (text, url)
    Item(Vec<Node>),                   // (children)
    Link(String, String),              // (text, url)
    List(Vec<Node>),                   // (children)
    Style(Style, Vec<Node>),           // (style, children)
    Table(Vec<Vec<Vec<Node>>>),        // (rows(columns(cells)))
    Text(String),                      // (text)
}

impl El {
    fn add_text(&mut self, text: &str) {
        match self {
            El::Empty | El::Image(..) | El::Link(..) | El::Table(..) => (),
            El::Span(children)
            | El::Details(_, children)
            | El::Style(_, children)
            | El::Heading(_, children)
            | El::Item(children)
            | El::List(children) => children.push(Node::text(text)),
            El::Code(string) | El::Codeblock(_, string) | El::Text(string) => {
                if !string.is_empty() {
                    string.push(' ');
                }
                string.push_str(text.trim());
            }
        }
    }

    fn is_empty(&self) -> bool {
        match self {
            El::Empty => true,
            El::Image(text, url) | El::Link(text, url) => text.is_empty() && url.is_empty(),
            El::Span(children)
            | El::Style(_, children)
            | El::Heading(_, children)
            | El::Item(children)
            | El::List(children) => all_empty(children),
            El::Details(summary, details) => all_empty(summary) && all_empty(details),
            El::Code(string) | El::Codeblock(_, string) | El::Text(string) => {
                string.trim().is_empty()
            }
            El::Table(rows) => {
                rows.is_empty()
                    || rows.iter().all(|row| {
                        row.is_empty()
                            || row
                                .iter()
                                .all(|col| col.is_empty() || col.iter().all(|node| node.is_empty()))
                    })
            }
        }
    }
}

fn all_empty(nodes: &[Node]) -> bool {
    nodes.iter().all(Node::is_empty)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_is_empty() {
        assert!(Node::text("").is_empty());
        assert!(!Node::text("hi").is_empty());
        assert!(!Node::table(vec![vec![vec![Node::text("hi")]]]).is_empty());
    }
}

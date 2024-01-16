use super::{El, Node};

#[derive(Debug, PartialEq)]
pub struct Doc(Vec<Node>);

impl Doc {
    pub fn nodes(&self) -> &[Node] {
        &self.0
    }

    pub fn title_from_heading(&self) -> Option<String> {
        for node in self.nodes() {
            if let El::Heading(1, children) = node.el() {
                for node in children {
                    if let El::Text(text) = node.el() {
                        return Some(text.clone());
                    }
                }
            }
        }

        None
    }

    #[cfg(test)]
    pub fn empty() -> Self {
        Self(Vec::new())
    }
}

impl From<Vec<Node>> for Doc {
    fn from(value: Vec<Node>) -> Self {
        Doc(value)
    }
}

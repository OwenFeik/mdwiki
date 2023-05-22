use crate::model::node::Node;

struct Html {
    content: String,
    stack: Vec<&'static str>
}

impl Html {
    fn new() -> Self {
        Self {
            content: String::new(),
            stack: Vec::new(),
        }
    }

    fn open(&mut self, tag: &'static str) {
        self.indent(self.stack.len());
        self.content.push('<');
        self.content.push_str(tag);
        self.content.push('>');
        self.stack.push(tag);
    }

    fn close(&mut self) {
        if let Some(tag) = self.stack.pop() {
            self.indent(self.stack.len());
            self.content.push_str("</");
            self.content.push_str(tag);
            self.content.push('>');
        }
        
    }

    fn indent(&mut self, distance: usize) {
        for _ in 0..distance {
            self.content.push(' ');
        }
    }
}

pub fn render_document(node: Node) -> Result<String, String> {
    if let Node::Document(nodes) = node {
        let mut html = Html::new();
        render(&nodes, &mut html)?;
        Ok(html.content)
    } else {
        Err(String::from("Root node not a document."))
    }
}

fn render(nodes: &[Node], html: &mut Html) -> Result<(), String> {
    for node in nodes {
        match node {
            Node::Empty => (),
            Node::Document(children) => {
                html.open("html");
                html.open("head");
                html.close();
                html.open("body");
                render(children, html)?;
                html.close();
                html.close();
            },
            Node::Heading(children) => {
                html.open("h1");
                render(children, html)?;
                html.close();
            },
            Node::Image(text, url) => todo!(),
            Node::Item(_) => todo!(),
            Node::Link(_, _) => todo!(),
            Node::List(_) => todo!(),
            Node::Style(_, _) => todo!(),
            Node::Text(_) => todo!(),
        }
    }

    Ok(())
}


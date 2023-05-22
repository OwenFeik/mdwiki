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
        self.content.push('<');
        self.content.push_str(tag);
        self.content.push('>');
        self.stack.push(tag);
    }

    fn close(&mut self) {
        if let Some(tag) = self.stack.pop() {
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

    fn push(&mut self, string: &str) {
        self.content.push_str(string);
    }
}

pub fn render_document(node: &Node) -> Result<String, String> {
    let mut html = Html::new();
    render(&node, &mut html)?;
    Ok(html.content)
}

fn render(node: &Node, html: &mut Html) -> Result<(), String> {
    match node {
        Node::Empty => (),
        Node::Document(children) => {
            html.open("html");
            html.open("head");
            html.close();
            html.open("body");
            render_nodes(children, html)?;
            html.close();
            html.close();
        },
        Node::Heading(children) => {
            html.open("h1");
            render_nodes(children, html)?;
            html.close();
        },
        Node::Image(text, url) => todo!(),
        Node::Item(_) => todo!(),
        Node::Link(_, _) => todo!(),
        Node::List(_) => todo!(),
        Node::Style(_, _) => todo!(),
        Node::Text(text) => html.push(text),
    }

    Ok(())
}


fn render_nodes(nodes: &[Node], html: &mut Html) -> Result<(), String> {
    for node in nodes {
        render(node, html)?;
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use crate::model::node::Node;

    #[test]
    fn test_render_heading() {
        assert_eq!(
            &super::render_document(&Node::Document(vec![
                Node::Heading(vec![Node::text("Hello World")])
            ])).unwrap(),
            "<html><head></head><body><h1>Hello World</h1></body></html>"
        )
    }
}

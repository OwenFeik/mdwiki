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
    
    fn _start(&mut self, tag: &str) {
        self.content.push('<');
        self.content.push_str(tag);
    }

    fn start(&mut self, tag: &'static str) {
        self._start(tag);
        self.stack.push(tag);
    }

    fn open(&mut self, tag: &'static str) {
        self.start(tag);
        self.finish();
    }

    fn finish(&mut self) {
        self.content.push('>');
    }

    fn openl(&mut self, tag: &'static str) {
        self.indent(self.stack.len());
        self.open(tag);
    }

    fn singleton(&mut self, tag: &'static str) {
        self._start(tag);
    }

    fn close(&mut self) {
        if let Some(tag) = self.stack.pop() {
            self.content.push_str("</");
            self.content.push_str(tag);
            self.content.push('>');
        }
    }

    fn closel(&mut self) {
        if !self.stack.is_empty() {
            self.indent(self.stack.len() - 1);
            self.close();
        }
    }

    fn indent(&mut self, n: usize) {
        self.content.push('\n');
        for _ in 0..n {
            self.content.push(' ');
        }
    }

    fn push(&mut self, string: &str) {
        self.content.push_str(string);
    }

    fn attr(&mut self, key: &str, value: &str) {
        self.space();
        self.push(key);
        self.content.push('=');
        self.content.push('"');
        self.push(value);
        self.content.push('"');
    }

    fn space(&mut self) {
        self.content.push(' ');
    }
}

pub fn render_document(node: &Node) -> Result<String, String> {
    let mut html = Html::new();
    render(&node, &mut html)?;
    Ok(String::from((&html.content).trim()))
}

fn render(node: &Node, html: &mut Html) -> Result<(), String> {
    match node {
        Node::Empty => (),
        Node::Document(children) => {
            html.open("html");
            html.openl("head");
            html.closel();
            html.openl("body");
            render_nodes(children, html)?;
            html.closel();
            html.closel();
        },
        Node::Heading(children) => {
            html.openl("h1");
            render_nodes(children, html)?;
            html.close();
        },
        Node::Image(text, url) => {
            html.singleton("img");
            html.attr("src", url);
            html.attr("alt", text);
            html.finish();
        },
        Node::Item(children) => {
            html.openl("li");
            render_nodes(children, html)?;
            html.close();
        },
        Node::Link(text, url) => {
            html.start("a");
            html.attr("href", url);
            html.finish();
            html.push(text);
            html.close();
        },
        Node::List(children) => {
            html.openl("ul");
            render_nodes(children, html)?;
            html.closel();
        },
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
            concat!(
                "<html>\n",
                " <head>\n",
                " </head>\n",
                " <body>\n",
                "  <h1>Hello World</h1>\n",
                " </body>\n",
                "</html>"
            )
        );
    }

    #[test]
    fn test_render_links() {
        assert_eq!(
            &super::render_document(&Node::List(vec![
                Node::Item(vec![
                    Node::text("Click here:"),
                    Node::link("Website", "https://owen.feik.xyz")
                ]),
                Node::Item(vec![
                    Node::image("image alt", "https://image.url")
                ])
            ])).unwrap(),
            concat!(
                "<ul>\n",
                " <li>Click here:<a href=\"https://owen.feik.xyz\">Website</a></li>\n",
                " <li><img src=\"https://image.url\" alt=\"image alt\"></li>\n",
                "</ul>"
            )
        );
    }
}

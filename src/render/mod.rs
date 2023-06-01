use crate::model::{Node, Style};

#[cfg(test)]
mod test;

const TABSIZE: usize = 2;

struct Html {
    content: String,
    stack: Vec<String>,
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

    fn start(&mut self, tag: &str) {
        self._start(tag);
        self.stack.push(String::from(tag));
    }

    fn open(&mut self, tag: &str) {
        self.start(tag);
        self.finish();
    }

    fn finish(&mut self) {
        self.content.push('>');
    }

    fn lopen(&mut self, tag: &str) {
        self.indent(self.stack.len());
        self.open(tag);
    }

    fn lopenl(&mut self, tag: &str) {
        self.lopen(tag);
        self.indent(self.stack.len());
    }

    fn singleton(&mut self, tag: &'static str) {
        self._start(tag);
    }

    fn close(&mut self) {
        if let Some(tag) = self.stack.pop() {
            self.trim_spaces();
            self.content.push_str("</");
            self.content.push_str(&tag);
            self.content.push('>');
        }
    }

    fn lclose(&mut self) {
        if !self.stack.is_empty() {
            self.trim_end();
            self.indent(self.stack.len().saturating_sub(1));
            self.close();
        }
    }

    fn lclosel(&mut self) {
        if !self.stack.is_empty() {
            self.lclose();
            self.indent(self.stack.len().saturating_sub(1));
        }
    }

    fn closel(&mut self) {
        if !self.stack.is_empty() {
            self.close();
            self.indent(self.stack.len());
        }
    }

    fn trim_spaces(&mut self) {
        let spaces_trimmed = self.content.trim_end_matches(' ');
        let len = spaces_trimmed.len();
        if !spaces_trimmed.ends_with('\n') {
            self.content.truncate(len);
        }
    }

    fn trim_end(&mut self) {
        self.content.truncate(self.content.trim_end().len());
    }

    fn nl(&mut self) {
        self.trim_end();
        self.content.push('\n');
    }

    fn indent(&mut self, n: usize) {
        self.nl();
        self.push(&" ".repeat(n * TABSIZE));
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

    fn space_if_needed(&mut self) {
        if self
            .content
            .ends_with(|c: char| !c.is_whitespace() && c != '>')
        {
            self.space();
        }
    }
}

fn escape(string: &str) -> String {
    string.replace('"', "&quot;")
}

pub fn indent(string: &str, by: usize) -> String {
    let mut repl = String::from("\n");
    repl.push_str(&" ".repeat(by * TABSIZE));
    String::from(string.replace('\n', &repl).trim())
}

fn render(node: &Node, html: &mut Html) {
    match node {
        Node::Empty => (),
        Node::Code(code) => {
            html.open("code");
            html.push(&escape(code));
            html.close();
        },
        Node::Codeblock(lang, code) => {
            html.lopenl("pre");
            html.push(&indent(&escape(code), html.stack.len()));
            html.lclosel();
        },
        Node::Document(children) => {
            html.open("html");
            html.lopen("head");
            html.lopenl("style");
            html.push(&indent(include_str!("res/style.css"), html.stack.len()));
            html.lclose();
            html.lclose();
            html.lopen("body");
            html.lopen("main");
            render_nodes(children, html);
            html.lclose();
            html.lclose();
            html.lclose();
        }
        Node::Heading(level, children) => {
            html.lopen(&format!("h{level}"));
            render_nodes(children, html);
            html.closel();
        }
        Node::Image(text, url) => {
            html.space_if_needed();
            html.singleton("img");
            html.attr("src", url);
            html.attr("alt", text);
            html.finish();
        }
        Node::Item(children) => {
            html.lopen("li");
            render_nodes(children, html);
            html.close();
        }
        Node::Link(text, url) => {
            html.space_if_needed();
            html.start("a");
            html.attr("href", &escape(url));
            html.finish();
            html.push(text);
            html.close();
            html.space();
        }
        Node::List(children) => {
            html.lopen("ul");
            render_nodes(children, html);
            html.lclosel();
        }
        Node::Style(style, children) => {
            let tag = match style {
                Style::Bold => "b",
                Style::Italic => "i",
                Style::Strikethrough => "s",
            };

            html.space_if_needed();
            html.open(tag);
            render_nodes(children, html);
            html.trim_end();
            html.close();
            html.space();
        }
        Node::Text(text) => {
            html.space_if_needed();
            html.push(text);
        }
    }
}

fn render_nodes(nodes: &[Node], html: &mut Html) {
    for node in nodes {
        render(node, html);
    }
}

pub fn render_document(node: &Node) -> String {
    let mut html = Html::new();
    render(&node, &mut html);
    String::from((&html.content).trim())
}

use crate::{
    config::Config,
    log::warning,
    model::{Node, Style},
};

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
        }
        Node::Codeblock(lang, code) => {
            html.lopenl("pre");
            html.push(&indent(&escape(code), html.stack.len()));
            html.lclosel();
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
        Node::Table(rows) => {}
        Node::Text(text) => {
            html.space_if_needed();
            if html.content.ends_with('>') && text.starts_with(char::is_alphanumeric) {
                html.space();
            }
            html.push(text);
        }
    }
}

fn render_nodes(nodes: &[Node], html: &mut Html) {
    for node in nodes {
        render(node, html);
    }
}

fn add_page_heading(path: &[String], html: &mut Html) {
    if let Some(title) = path.last() {
        render(&Node::Heading(1, vec![Node::text(title)]), html);
    } else {
        warning("add_page_heading=true but no path present.");
    }
}

fn add_page_path(path: &[String], html: &mut Html) {
    let n = path.len();
    if n > 1 {
        let mut nodes = Vec::new();
        for i in 0..=(n - 1) {
            let url = "../".repeat(n - 1 - i).to_string();
            nodes.push(Node::text("/"));
            nodes.push(Node::Link(path[i].to_string(), url));
        }
        render(&Node::Heading(3, nodes), html);
    }
}

pub fn render_document(config: &Config, path: &[String], nodes: &[Node]) -> String {
    let mut html = Html::new();
    let mut paragraph_open = false;

    html.open("html");
    html.lopen("head");
    html.lopenl("style");
    html.push(&indent(include_str!("res/style.css"), html.stack.len()));
    html.lclose();
    html.lclose();
    html.lopen("body");
    html.lopen("main");

    if config.path {
        add_page_path(path, &mut html);
    }

    if config.page_heading {
        add_page_heading(path, &mut html);
    }

    let mut prev = None;
    for node in nodes {
        match node {
            Node::Code(..) | Node::Link(..) | Node::Style(..) | Node::Text(..) => {
                if paragraph_open && matches!(prev, Some(&Node::Text(..))) {
                    html.lclosel();
                    paragraph_open = false;
                }

                if !paragraph_open {
                    html.lopenl("p");
                    paragraph_open = true;
                }
            }
            Node::Codeblock(..)
            | Node::Heading(..)
            | Node::Image(..)
            | Node::List(..)
            | Node::Table(..) => {
                if paragraph_open {
                    html.lclosel();
                }
                paragraph_open = false;
            }
            Node::Item(..) => warning("List item at root level."),
            Node::Empty => {}
        }

        render(node, &mut html);
        prev = Some(node);
    }

    html.lclose();
    html.lclose();
    html.lclose();

    String::from(html.content.trim())
}

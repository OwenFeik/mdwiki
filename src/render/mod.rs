use crate::{
    config::Config,
    fstree::FsTree,
    log::warning,
    model::{El, Node, Style},
};

#[cfg(test)]
mod test;

const TABSIZE: usize = 2;

struct Html {
    content: String,
    stack: Vec<String>,
    just_closed: Option<String>,
}

impl Html {
    fn new() -> Self {
        Self {
            content: String::new(),
            stack: Vec::new(),
            just_closed: None,
        }
    }

    fn _start(&mut self, tag: &str) {
        self.push('<');
        self.push_str(tag);
    }

    fn start(&mut self, tag: &str) {
        self._start(tag);
        self.stack.push(String::from(tag));
    }

    fn lstart(&mut self, tag: &str) {
        self.indent(self.stack.len());
        self.start(tag);
    }

    fn open(&mut self, tag: &str) {
        self.start(tag);
        self.finish();
    }

    fn finish(&mut self) {
        self.push('>');
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
            self.push_str("</");
            self.push_str(&tag);
            self.push('>');
            self.just_closed = Some(tag);
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
        self.push('\n');
    }

    fn indent(&mut self, n: usize) {
        self.nl();
        self.push_str(&" ".repeat(n * TABSIZE));
    }

    fn push(&mut self, c: char) {
        self.content.push(c);
        self.just_closed = None;
    }

    fn push_str(&mut self, string: &str) {
        self.content.push_str(string);
        self.just_closed = None;
    }

    fn attr(&mut self, key: &str, value: &str) {
        self.space();
        self.push_str(key);
        self.push('=');
        self.push('"');
        self.push_str(value);
        self.push('"');
    }

    fn space(&mut self) {
        self.push(' ');
    }

    fn space_if_needed(&mut self) {
        if self.content.ends_with('>') {
            if matches!(
                self.just_closed.as_deref(),
                Some("a" | "b" | "i" | "s" | "code")
            ) {
                self.space();
            }
        } else if !self.content.ends_with(char::is_whitespace) {
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
    match node.el() {
        El::Empty => (),
        El::Code(code) => {
            html.open("code");
            html.push_str(&escape(code));
            html.close();
        }
        El::Codeblock(_lang, code) => {
            html.lopenl("pre");
            html.push_str(&indent(&escape(code), html.stack.len()));
            html.lclosel();
        }
        El::Heading(level, children) => {
            html.lopen(&format!("h{level}"));
            render_nodes(children, html);
            html.closel();
        }
        El::Image(text, url) => {
            html.space_if_needed();
            html.singleton("img");
            html.attr("src", url);
            html.attr("alt", text);
            html.finish();
        }
        El::Item(children) => {
            html.lopen("li");
            render_nodes(children, html);
            html.close();
        }
        El::Link(text, url) => {
            html.space_if_needed();
            html.start("a");
            html.attr("href", &escape(url));
            html.finish();
            html.push_str(text);
            html.close();
        }
        El::List(children) => {
            html.lopen("ul");
            render_nodes(children, html);
            html.lclosel();
        }
        El::Style(style, children) => {
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
        El::Table(rows) => {}
        El::Text(text) => {
            html.space_if_needed();
            html.push_str(text);
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
        render(&Node::heading(1, vec![Node::text(title)]), html);
    } else {
        warning("add_page_heading=true but no path present.");
    }
}

fn add_page_path(path: &[String], html: &mut Html) {
    let n = path.len();
    if n > 1 {
        let mut nodes = Vec::new();
        for (i, name) in path.iter().enumerate() {
            let url = "../".repeat(n - 1 - i).to_string();
            nodes.push(Node::text("/"));
            nodes.push(Node::link(name, &url));
        }
        render(&Node::heading(3, nodes), html);
    }
}

fn make_nav_subtree(tree: &FsTree, id: usize) -> Vec<Node> {
    let mut entries = Vec::new();
    if let Some(node) = tree.get(id) {
        if let Some(name) = node.name() {
            entries.push(Node::link(name, &node.url()))
        }
    }

    let mut children = Vec::new();
    for child in tree.children(id) {
        let subtree = make_nav_subtree(tree, child);
        children.extend(subtree);
    }

    if !children.is_empty() {
        entries.push(Node::list(children));
    }

    entries
}

fn make_nav_tree(tree: &FsTree) -> Node {
    Node::list(make_nav_subtree(tree, FsTree::ROOT))
}

const FONT: &str = "https://fonts.googleapis.com/css?family=Open%20Sans";

pub fn render_document(config: &Config, tree: &FsTree, page: usize, nodes: &[Node]) -> String {
    let mut html = Html::new();
    let mut paragraph_open = false;

    html.open("html");
    html.lopen("head");
    html.lstart("style");
    html.attr("href", FONT);
    html.attr("rel", "stylesheet");
    html.finish();
    html.close();
    html.lopenl("style");
    html.push_str(&indent(include_str!("res/style.css"), html.stack.len()));
    html.lclose();
    html.lclose();
    html.lopen("body");
    html.lopen("main");

    if let Some(node) = tree.get(page) {
        let path = node.path();
        if config.path {
            add_page_path(path, &mut html);
        }

        if config.page_heading {
            add_page_heading(path, &mut html);
        }
    }

    let mut prev: Option<&Node> = None;
    for node in nodes {
        match node.el() {
            El::Code(..) | El::Link(..) | El::Style(..) | El::Text(..) => {
                if paragraph_open {
                    match prev {
                        Some(n) if matches!(n.el(), El::Text(..)) => {
                            html.lclosel();
                            paragraph_open = false;
                        }
                        _ => {}
                    }
                }

                if !paragraph_open {
                    html.lopenl("p");
                    paragraph_open = true;
                }
            }
            El::Codeblock(..) | El::Heading(..) | El::Image(..) | El::List(..) | El::Table(..) => {
                if paragraph_open {
                    html.lclosel();
                }
                paragraph_open = false;
            }
            El::Item(..) => warning("List item at root level."),
            El::Empty => {}
        }

        render(node, &mut html);
        prev = Some(node);
    }

    html.lclose();
    html.lclose();
    html.lclose();

    String::from(html.content.trim())
}

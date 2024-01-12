use std::collections::HashMap;

use crate::{
    config::Config,
    fstree::FsTree,
    log::warning,
    model::{Attrs, El, Node, Style},
    Document,
};

mod nav;

#[cfg(test)]
mod test;

pub use self::nav::create_index;

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

    fn open(&mut self, tag: &str, attrs: &Attrs) {
        self.start(tag);
        self.finish(attrs);
    }

    fn finish(&mut self, attrs: &Attrs) {
        attrs.iter().for_each(|(k, v)| self.attr(k, v));
        self.push('>');
    }

    fn lopen(&mut self, tag: &str, attrs: &Attrs) {
        self.indent(self.stack.len());
        self.open(tag, attrs);
    }

    fn lopenl(&mut self, tag: &str, attrs: &Attrs) {
        self.lopen(tag, attrs);
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
        El::Div(children) => {
            html.lopenl("div", node.attrs());
            render_nodes(children, html);
            html.lclosel();
        }
        El::Code(code) => {
            html.open("code", node.attrs());
            html.push_str(&escape(code));
            html.close();
        }
        El::Codeblock(_lang, code) => {
            html.lopenl("pre", node.attrs());
            html.push_str(&indent(&escape(code), html.stack.len()));
            html.lclosel();
        }
        El::Heading(level, children) => {
            html.lopen(&format!("h{level}"), node.attrs());
            render_nodes(children, html);
            html.closel();
        }
        El::Image(text, url) => {
            html.space_if_needed();
            html.singleton("img");
            html.attr("src", url);
            html.attr("alt", text);
            html.finish(node.attrs());
        }
        El::Item(children) => {
            html.lopen("li", node.attrs());
            render_nodes(children, html);
            html.close();
        }
        El::Link(text, url) => {
            html.space_if_needed();
            html.start("a");
            html.attr("href", &escape(url));
            html.finish(node.attrs());
            html.push_str(text);
            html.close();
        }
        El::List(children) => {
            html.lopen("ul", node.attrs());
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
            html.open(tag, node.attrs());
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

fn add_page_path(tree: &FsTree, doc: &Document, html: &mut Html) {
    if let Some(fsnode) = tree.get(doc.fsnode) {
        render(&nav::make_nav_breadcrumb(tree, fsnode), html);
    } else {
        warning(format!(
            "Failed to add page path for {}, FsNode not found.",
            doc.output.display()
        ))
    }
}

const FONT: &str = "https://fonts.googleapis.com/css?family=Open%20Sans";

pub fn render_document(config: &Config, tree: &FsTree, doc: &Document) -> String {
    let mut html = Html::new();
    let mut paragraph_open = false;

    let empty = &HashMap::new();

    html.open("html", empty);
    html.lopenl("head", empty);

    if let Some(text) = tree.get(doc.fsnode).map(|n| n.title()) {
        html.open("title", empty);
        render(&Node::text(text), &mut html);
        html.close();
    }

    html.lstart("style");
    html.attr("href", FONT);
    html.attr("rel", "stylesheet");
    html.finish(empty);
    html.close();
    html.lopenl("style", empty);
    html.push_str(&indent(include_str!("res/style.css"), html.stack.len()));
    html.lclose();
    html.lclose();
    html.lopenl("body", empty);

    if config.nav_tree {
        render(&nav::make_nav_tree(tree, doc), &mut html);
    }

    html.start("div");
    html.attr("id", "content");
    html.finish(empty);
    html.lopen("main", empty);

    if let Some(node) = tree.get(doc.fsnode) {
        if config.path {
            add_page_path(tree, doc, &mut html);
        }

        if config.page_heading {
            add_page_heading(node.path(), &mut html);
        }
    }

    let mut prev: Option<&Node> = None;
    for node in &doc.document {
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
                    html.lopenl("p", empty);
                    paragraph_open = true;
                }
            }
            El::Div(..)
            | El::Codeblock(..)
            | El::Heading(..)
            | El::Image(..)
            | El::List(..)
            | El::Table(..) => {
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
    html.lclose();

    String::from(html.content.trim())
}

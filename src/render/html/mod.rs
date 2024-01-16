use std::collections::HashMap;

use crate::{
    config::Config,
    log,
    model::{Attrs, El, Node, Style, WikiPage, WikiTree},
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
                Some("a" | "b" | "i" | "s" | "code" | "span")
            ) {
                self.space();
            }
        } else if !self.content.ends_with(char::is_whitespace) {
            self.space();
        }
    }
}

struct RenderState<'a> {
    tree: &'a WikiTree,
    page: &'a WikiPage,
    config: &'a Config,
    html: &'a mut Html,
}

impl<'a> std::ops::Deref for RenderState<'a> {
    type Target = Html;

    fn deref(&self) -> &Self::Target {
        self.html
    }
}

impl<'a> std::ops::DerefMut for RenderState<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.html
    }
}

fn escape(string: &str) -> String {
    string.replace('"', "&quot;")
}

fn indent(string: &str, by: usize) -> String {
    let mut repl = String::from("\n");
    repl.push_str(&" ".repeat(by * TABSIZE));
    String::from(string.replace('\n', &repl).trim())
}

fn render(state: &mut RenderState, node: &Node) {
    match node.el() {
        El::Empty => (),
        El::Div(children) => {
            state.html.lopenl("div", node.attrs());
            render_nodes(state, children);
            state.lclosel();
        }
        El::Span(children) => {
            state.space_if_needed();
            state.open("span", node.attrs());
            render_nodes(state, children);
            state.close();
        }
        El::Code(code) => {
            state.open("code", node.attrs());
            state.push_str(&escape(code));
            state.close();
        }
        El::Codeblock(_lang, code) => {
            state.lopenl("pre", node.attrs());
            let indent_size = state.stack.len();
            state.push_str(&indent(&escape(code), indent_size));
            state.lclosel();
        }
        El::Details(summary, details) => {
            state.lopenl("details", node.attrs());
            state.lopen("summary", &HashMap::new());
            render_nodes(state, summary);
            state.close();
            render_nodes(state, details);
            state.lclosel();
        }
        El::Heading(level, children) => {
            state.lopen(&format!("h{level}"), node.attrs());
            render_nodes(state, children);
            state.closel();
        }
        El::Image(text, url) => {
            state.space_if_needed();
            state.singleton("img");
            state.attr("src", url);
            state.attr("alt", text);
            state.finish(node.attrs());
        }
        El::Item(children) => {
            state.lopen("li", node.attrs());
            render_nodes(state, children);
            state.close();
        }
        El::Link(text, url) => {
            let mut url: &str = url;
            if url.is_empty() && state.config.empty_links {
                if let Some(target) = state.tree.find_link_target(text, state.page) {
                    url = target.url();
                }
            }

            state.space_if_needed();
            state.start("a");
            state.attr("href", &escape(url));
            state.finish(node.attrs());
            state.push_str(text);
            state.close();
        }
        El::List(children) => {
            state.lopen("ul", node.attrs());
            render_nodes(state, children);
            state.lclosel();
        }
        El::Style(style, children) => {
            let tag = match style {
                Style::Bold => "b",
                Style::Italic => "i",
                Style::Strikethrough => "s",
            };

            state.space_if_needed();
            state.open(tag, node.attrs());
            render_nodes(state, children);
            state.trim_end();
            state.close();
            state.space();
        }
        El::Table(rows) => {}
        El::Text(text) => {
            state.space_if_needed();
            state.push_str(text);
        }
    }
}

fn render_nodes(state: &mut RenderState, nodes: &[Node]) {
    for node in nodes {
        render(state, node);
    }
}

fn add_page_heading(state: &mut RenderState, page: &WikiPage) {
    render(state, &Node::heading(1, vec![Node::text(page.title())]));
}

fn add_page_path(state: &mut RenderState, page: &WikiPage) {
    render(state, &super::nav::make_nav_breadcrumb(state.tree, page));
}

const FONT: &str = "https://fonts.googleapis.com/css?family=Open%20Sans";

#[cfg(test)]
pub fn render_node(node: &Node) -> String {
    use crate::model::Doc;

    let mut tree = WikiTree::new();
    let page = tree.add_doc(WikiTree::ROOT, "document.html", "Document", Doc::empty());
    let page = tree.get(page).unwrap();
    let config = Config::none();
    let mut html = Html::new();
    let mut state = RenderState {
        tree: &tree,
        page,
        config: &config,
        html: &mut html,
    };
    render(&mut state, node);
    html.content.trim().to_string()
}

pub fn render_document(config: &Config, tree: &WikiTree, page: &WikiPage) -> Result<String, ()> {
    let Some(doc) = page.document() else {
        log::error(format!(
            "Can't render page with no document: {}",
            page.url()
        ));
        return Err(());
    };

    let mut html = Html::new();
    let mut state = RenderState {
        tree,
        page,
        config,
        html: &mut html,
    };

    let empty = &HashMap::new();

    state.html.open("html", empty);
    state.html.lopenl("head", empty);

    state.html.open("title", empty);
    render(&mut state, &Node::text(page.title()));
    state.html.close();

    state.html.lstart("style");
    state.html.attr("href", FONT);
    state.html.attr("rel", "stylesheet");
    state.html.finish(empty);
    state.html.close();
    state.html.lopenl("style", empty);
    state.html.push_str(&indent(
        include_str!("res/style.css"),
        state.html.stack.len(),
    ));
    state.html.lclose();
    state.html.lclose();
    state.html.lopenl("body", empty);

    if config.nav_tree {
        render(&mut state, &super::nav::make_nav_tree(tree, page));
    }

    state.html.start("div");
    state.html.attr("id", "content");
    state.html.finish(empty);
    state.html.lopen("main", empty);

    if config.page_heading {
        add_page_heading(&mut state, page);
    }

    if config.add_breadcrumbs {
        add_page_path(&mut state, page);
    }

    let mut paragraph_open = false;
    let mut prev: Option<&Node> = None;
    for node in doc.nodes() {
        match node.el() {
            El::Code(..) | El::Link(..) | El::Style(..) | El::Text(..) => {
                if paragraph_open {
                    match prev {
                        Some(n) if matches!(n.el(), El::Text(..)) => {
                            state.html.lclosel();
                            paragraph_open = false;
                        }
                        _ => {}
                    }
                }

                if !paragraph_open {
                    state.html.lopenl("p", empty);
                    paragraph_open = true;
                }
            }
            El::Div(..)
            | El::Span(..)
            | El::Codeblock(..)
            | El::Details(..)
            | El::Heading(..)
            | El::Image(..)
            | El::List(..)
            | El::Table(..) => {
                if paragraph_open {
                    state.html.lclosel();
                }
                paragraph_open = false;
            }
            El::Item(..) => log::warning("List item at root level."),
            El::Empty => {}
        }

        render(&mut state, node);
        prev = Some(node);
    }

    html.lclose();
    html.lclose();
    html.lclose();
    html.lclose();

    Ok(String::from(html.content.trim()))
}

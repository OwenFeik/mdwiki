use core::slice;
use std::collections::HashMap;

use crate::{
    config::Config,
    log,
    model::{Attrs, El, Node, Style, Tag, WikiPage, WikiTree},
    render::css::{floating_menu, with_class, with_id},
};

use super::{OUTPUT_EXT, RenderState, encryption_pairs};

pub const TABSIZE: usize = 2;

pub struct Html {
    content: String,
    stack: Vec<String>,
    just_closed: Option<String>,
    indent_adjust: usize,
}

impl Html {
    pub fn new() -> Self {
        Self {
            content: String::new(),
            stack: Vec::new(),
            just_closed: None,
            indent_adjust: 0,
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
        self.push_str(&" ".repeat((n + self.indent_adjust) * TABSIZE));
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

pub fn escape(string: &str) -> String {
    string.replace('"', "&quot;")
}

pub fn indent(string: &str, by: usize) -> String {
    let mut repl = String::from("\n");
    repl.push_str(&" ".repeat(by * TABSIZE));
    String::from(string.replace('\n', &repl).trim())
}

fn handle_empty_url(state: &RenderState, text: &str, ext: &str, url: &str) -> String {
    if url.is_empty() && state.config.empty_links {
        if let Some(target) = state.tree.find_link_target(text, ext, state.page) {
            return target.url().to_string();
        }
    }

    url.to_string()
}

fn render(state: &mut RenderState, node: &Node, skip_encryption: bool) {
    if !skip_encryption && handle_encryption_node(state, node) {
        return;
    }

    match node.el() {
        El::Empty => (),
        El::Block(tag, children) => {
            state.lopenl(tag, node.attrs());
            render_nodes(state, children, false);
            state.lclose();
        }
        El::Inline(tag, children) => {
            state.space_if_needed();
            state.open(tag, node.attrs());
            render_nodes(state, children, false);
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
            render_nodes(state, summary, false);
            state.close();
            render_nodes(state, details, false);
            state.lclosel();
        }
        El::Heading(level, children) => {
            state.lopen(&format!("h{level}"), node.attrs());
            render_nodes(state, children, false);
            state.closel();
        }
        El::Image(text, url) => {
            let mut url: String = url.clone();
            for ext in crate::parse::IMAGE_EXTS {
                if url.is_empty() {
                    url = handle_empty_url(state, text, ext, &url);
                } else {
                    break;
                }
            }

            if url.is_empty() {
                log::warning(format!(
                    "Failed to find URL for image \"{text}\" on {}",
                    state.page.url()
                ))
            }

            state.space_if_needed();
            state.singleton("img");
            state.attr("src", &escape(&url));
            state.attr("alt", text);
            state.finish(node.attrs());
        }
        El::Item(children) => {
            state.lopen("li", node.attrs());
            render_nodes(state, children, false);
            state.close();
        }
        El::Link(text, url) => {
            let url = handle_empty_url(state, text, OUTPUT_EXT, url);

            if url.is_empty() {
                log::warning(format!(
                    "Failed to find URL for link \"{text}\" on {}",
                    state.page.url()
                ))
            }

            state.space_if_needed();
            state.start("a");
            state.attr("href", &escape(&url));
            state.finish(node.attrs());
            state.push_str(text);
            state.close();
        }
        El::List(children) => {
            state.lopen("ul", node.attrs());
            render_nodes(state, children, false);
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
            render_nodes(state, children, false);
            state.trim_end();
            state.close();
            state.space();
        }
        El::Table(_rows) => {}
        El::Text(text) => {
            if text.starts_with(char::is_alphanumeric) || text.starts_with('/') {
                state.space_if_needed();
            }
            state.push_str(text);
        }
    }
}

fn encrypt_text(pairs: &[(&Tag, &String)], plaintext: &str) -> Node {
    const CSS_CLASS: &str = "secret";
    const SEP: &str = ";";

    let mut tags = Vec::new();
    let mut nonces = Vec::new();

    let mut ciphertext = String::new();
    for (tag, password) in pairs {
        if let Ok((nonce, encrypted)) = super::aes::encrypt(plaintext, password) {
            tags.push(tag.as_ref());
            nonces.push(nonce);
            ciphertext = encrypted;
        }
    }

    with_class(Node::span(vec![Node::text(&ciphertext)]), CSS_CLASS)
        .with_attr("tags", &tags.join(SEP))
        .with_attr("nonces", &nonces.join(SEP))
}

pub fn encrypt_nodes(
    state: &RenderState,
    pairs: &[(&Tag, &String)],
    nodes: &[Node],
    at_root: bool,
) -> Node {
    let plaintext = if at_root {
        render_root_range(state, nodes, true)
    } else {
        render_nodes_only(state.config, state.tree, state.page, nodes, true)
    };

    encrypt_text(pairs, &plaintext)
}

fn render_nodes(state: &mut RenderState, nodes: &[Node], skip_encryption: bool) {
    let mut skip = 0;
    for (i, node) in nodes.iter().enumerate() {
        if skip > 0 {
            skip -= 1;
            continue;
        }

        if skip_encryption {
            render(state, node, skip_encryption);
        } else if let Some(n) = handle_encryption_section(state, &nodes[i..], false) {
            skip = n;
        } else {
            render(state, node, skip_encryption);
        }
    }
}

fn add_page_heading(state: &mut RenderState, page: &WikiPage) {
    render(
        state,
        &Node::heading(1, vec![Node::text(page.title())]),
        false,
    );
}

fn add_page_path(state: &mut RenderState) {
    render(state, &super::nav::make_nav_breadcrumb(state), false);
}

pub fn render_nodes_only(
    config: &Config,
    tree: &WikiTree,
    page: &WikiPage,
    nodes: &[Node],
    skip_encryption: bool,
) -> String {
    let mut html = Html::new();
    let mut state = RenderState {
        tree,
        page,
        config,
        html: &mut html,
    };
    render_nodes(&mut state, nodes, skip_encryption);
    html.content.trim().to_string()
}

#[cfg(test)]
pub fn render_node_only(config: &Config, tree: &WikiTree, page: &WikiPage, node: &Node) -> String {
    let mut html = Html::new();
    let mut state = RenderState {
        tree,
        page,
        config,
        html: &mut html,
    };
    render(&mut state, node, false);
    html.content.trim().to_string()
}

fn handle_encryption_node(state: &mut RenderState, node: &Node) -> bool {
    if let Some(pairs) = encryption_pairs(state, node.tags()) {
        let nodes = slice::from_ref(node);
        render(state, &encrypt_nodes(state, &pairs, nodes, false), true);
        true
    } else {
        false
    }
}

fn handle_encryption_section(
    state: &mut RenderState,
    nodes: &[Node],
    at_root: bool,
) -> Option<usize> {
    let node = nodes.first()?;
    let pairs = encryption_pairs(state, node.tags())?;
    let (skip, nodes) = if let El::Heading(nt, _) = node.el() {
        let idx = if let Some(next_heading_idx) = nodes[1..]
            .iter()
            .position(|n| matches!(n.el(), El::Heading(t, _) if *nt == *t))
        {
            next_heading_idx
        } else {
            nodes.len() - 1
        };
        (idx, &nodes[..idx])
    } else {
        (0, &nodes[..1])
    };

    // If any of this nodes tags are password protected, render out an
    // encrypted node instead.
    render(state, &encrypt_nodes(state, &pairs, nodes, at_root), false);
    Some(skip)
}

fn header(title: &str) -> Node {
    Node::block(
        "head",
        vec![
            Node::inline("title", vec![Node::text(title)]),
            Node::block(
                "style",
                vec![Node::text(&indent(include_str!("res/style.css"), 3))],
            ),
            Node::block(
                "script",
                vec![Node::text(&indent(include_str!("res/decrypt.js"), 3))],
            ),
        ],
    )
}

fn render_root_range(state: &RenderState, range: &[Node], skip_encryption: bool) -> String {
    let mut html = Html::new();
    html.indent_adjust = state.stack.len();
    let mut state = RenderState {
        tree: state.tree,
        page: state.page,
        config: state.config,
        html: &mut html,
    };

    let empty = &HashMap::new();
    let mut skip = 0;
    let mut paragraph_open = false;
    let mut prev: Option<&Node> = None;
    for (i, node) in range.iter().enumerate() {
        if skip > 0 {
            skip -= 1;
            continue;
        }

        if !skip_encryption {
            if let Some(n) = handle_encryption_section(&mut state, &range[i..], true) {
                skip = n;
                continue;
            }
        }

        let mut paragraph_needed = false;
        match node.el() {
            El::Text(..) => {
                paragraph_needed = true;

                if paragraph_open {
                    match prev {
                        Some(n) if matches!(n.el(), El::Text(..)) => {
                            state.lclosel();
                            paragraph_open = false;
                        }
                        _ => {}
                    }
                }
            }
            El::Code(..) | El::Link(..) | El::Style(..) => {
                paragraph_needed = true;
            }
            El::Block(..)
            | El::Inline(..)
            | El::Codeblock(..)
            | El::Details(..)
            | El::Heading(..)
            | El::Image(..)
            | El::List(..)
            | El::Table(..) => {
                if paragraph_open {
                    state.lclosel();
                }
                paragraph_open = false;
            }
            El::Item(..) => log::warning("List item at root level."),
            El::Empty => {}
        }

        if !paragraph_open && paragraph_needed {
            state.lopenl("p", empty);
            paragraph_open = true;
        }

        render(&mut state, node, skip_encryption);
        prev = Some(node);
    }

    if paragraph_open {
        state.lclose();
    }

    html.content
}

fn make_tag_key_menu(config: &Config) -> Node {
    const ID: &str = "tag-keys-menu";
    const TITLE_CLASS: &str = "title";
    const LABEL_CLASS: &str = "tag-keys-label";
    const TEST_CLASS: &str = "tag-keys-test";

    let mut tag_entrys = Vec::new();
    for (tag, key) in config.tag_keys.iter() {
        tag_entrys.push(Node::item(vec![
            with_class(Node::span(vec![Node::text(tag.as_ref())]), LABEL_CLASS),
            Node::inline("input", Vec::new())
                .with_attr("type", "password")
                .with_attr("autocomplete", "off"),
            Node::inline("button", vec![Node::text("Clear")]),
            encrypt_text(&[(tag, key)], "correct").with_attr("class", TEST_CLASS),
        ]));
    }

    let node = Node::div(vec![Node::details(
        vec![with_class(
            Node::span(vec![Node::text("Keys")]),
            TITLE_CLASS,
        )],
        vec![Node::list(tag_entrys)],
    )]);
    with_id(floating_menu(node), ID)
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

    state.lopenl("html", empty);
    render(&mut state, &header(page.title()), true);

    state.lopenl("body", empty);

    if config.nav_tree {
        let nav_tree = super::nav::make_nav_tree(&state);
        render(&mut state, &nav_tree, false);
    }

    if !config.tag_keys.is_empty() {
        render(&mut state, &make_tag_key_menu(config), true);
    }

    state.start("div");
    state.attr("id", "content");
    state.finish(empty);
    state.lopen("main", empty);

    if config.page_heading {
        add_page_heading(&mut state, page);
    }

    if config.add_breadcrumbs {
        add_page_path(&mut state);
    }

    let content = render_root_range(&state, doc.nodes(), config.tag_keys.is_empty());
    state.push_str(&content);

    html.lclose();
    html.lclose();
    html.lclose();
    html.lclose();

    Ok(String::from(html.content.trim()))
}

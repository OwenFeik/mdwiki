use crate::model::{Doc, Node, WikiPage, WikiTree};

use super::{encryption_pairs, html::encrypt_nodes, RenderState, CSS_CLASS_ATTR, CSS_ID_ATTR};

pub const CSS_CLASS_THIS_PAGE: &str = "nav-tree-selected";
pub const CSS_CLASS_BULLET: &str = "nav-tree-bullet";
pub const CSS_ID_NAV_TREE: &str = "nav-tree";
const CSS_ID_NAV_BREADCRUMB: &str = "nav-breadcrumb";

fn make_page_link<'a>(state: &'a RenderState, page: &'a WikiPage) -> Node {
    let node = Node::link(page.title(), page.url());
    if let Some(pairs) = encryption_pairs(state, page.tags()) {
        encrypt_nodes(state, &pairs, &[node], false)
    } else {
        node
    }
}

fn make_nav_subtree<'a>(state: &'a RenderState, mut current: &'a WikiPage) -> Node {
    if current.is_media() {
        return Node::empty();
    }

    let mut children = Vec::new();
    for child in state.tree.children(current.id()) {
        if child.is_index() {
            current = child;
        } else {
            let subtree = make_nav_subtree(state, child);
            if !subtree.is_empty() {
                children.push(subtree);
            }
        }
    }

    let mut link = make_page_link(state, current);
    if current.id() == state.page.id() {
        link.attr(CSS_CLASS_ATTR, CSS_CLASS_THIS_PAGE);
    }

    if !children.is_empty() {
        let mut node = Node::details(vec![link], vec![Node::list(children)]);

        if state.page.is_descendent_of(current.id())
            || (current.is_index()
                && current
                    .parent()
                    .map(|p| state.page.is_descendent_of(p))
                    .unwrap_or(false))
        {
            node.attr("open", "");
        }
        node
    } else if !current.is_dir() {
        Node::item(vec![
            Node::span(Vec::new()).with_class(CSS_CLASS_BULLET),
            link,
        ])
    } else {
        Node::empty()
    }
}

pub fn make_nav_tree(state: &RenderState) -> Node {
    let mut items = Vec::new();
    for child in state.tree.children(WikiTree::ROOT) {
        let subtree = make_nav_subtree(state, child);
        if !subtree.is_empty() {
            items.push(subtree);
        }
    }
    Node::list(items).with_attr(CSS_ID_ATTR, CSS_ID_NAV_TREE)
}

fn next_breadcrumb<'a>(tree: &'a WikiTree, current: &'a WikiPage) -> Option<&'a WikiPage> {
    if current.is_index() {
        tree.get_parent(current)
            .and_then(|parent| tree.get_parent(parent))
    } else {
        tree.get_parent(current)
    }
}

pub fn make_nav_breadcrumb(state: &RenderState) -> Node {
    let mut nodes = Vec::new();
    let mut current = next_breadcrumb(state.tree, state.page);
    while let Some(entry) = current
        && !entry.is_root()
    {
        nodes.push(Node::text("/"));
        nodes.push(make_page_link(state, entry));
        current = next_breadcrumb(state.tree, entry);
    }
    nodes.reverse();

    if nodes.is_empty() {
        Node::empty()
    } else {
        Node::heading(3, nodes).with_attr("id", CSS_ID_NAV_BREADCRUMB)
    }
}

pub fn create_index(page: &WikiPage, children: &[&WikiPage]) -> Doc {
    Doc::from(vec![
        Node::heading(1, vec![Node::text(page.title())]),
        Node::list(
            children
                .iter()
                .map(|n| Node::link(n.title(), n.url()))
                .collect(),
        ),
    ])
}

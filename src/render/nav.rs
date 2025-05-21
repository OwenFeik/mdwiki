use crate::{
    model::{Doc, Node, WikiPage, WikiTree},
    render::css::{floating_menu, title, with_class},
};

use super::{RenderState, css::with_id, encryption_pairs, html::encrypt_nodes};

fn make_page_link(page: &WikiPage) -> Node {
    Node::link(page.title(), page.url())
}

fn page_encryption<'a>(state: &'a RenderState, page: &'a WikiPage, node: Node) -> Node {
    if let Some(pairs) = encryption_pairs(state, page.tags()) {
        encrypt_nodes(state, &pairs, &[node], false)
    } else {
        node
    }
}

fn make_nav_subtree<'a>(state: &'a RenderState, mut current: &'a WikiPage) -> Node {
    const THIS_PAGE_CLASS: &str = "nav-tree-selected";
    const CLASS_BULLET: &str = "nav-tree-bullet";

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

    let mut link = make_page_link(current);
    if current.id() == state.page.id() {
        link = with_class(link, THIS_PAGE_CLASS);
    }

    let node = if !children.is_empty() {
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
        Node::item(vec![with_class(Node::span(Vec::new()), CLASS_BULLET), link])
    } else {
        Node::empty()
    };

    page_encryption(state, current, node)
}

pub fn make_nav_tree(state: &RenderState) -> Node {
    const NAV_TREE_ID: &str = "nav-tree";

    let mut items = Vec::new();
    for child in state.tree.children(WikiTree::ROOT) {
        let subtree = make_nav_subtree(state, child);
        if !subtree.is_empty() {
            items.push(subtree);
        }
    }
    with_id(
        floating_menu(Node::div(vec![
            title(Node::span(vec![Node::text("Pages")])),
            Node::list(items),
        ])),
        NAV_TREE_ID,
    )
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
    const NAV_BREADCRUMB_ID: &str = "nav-breadcrumb";

    let mut nodes = Vec::new();
    let mut current = next_breadcrumb(state.tree, state.page);
    while let Some(entry) = current {
        if entry.is_root() {
            break;
        }

        let node = Node::span(vec![make_page_link(entry), Node::text("/")]);
        nodes.push(page_encryption(state, entry, node));
        current = next_breadcrumb(state.tree, entry);
    }
    nodes.reverse();

    if nodes.is_empty() {
        Node::empty()
    } else {
        with_id(Node::heading(3, nodes), NAV_BREADCRUMB_ID)
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

use crate::{
    fstree::{FsNode, FsTree},
    model::Node,
    Document,
};

const CSS_CLASS_ATTR: &str = "class";
const CSS_ID_ATTR: &str = "id";
const THIS_PAGE_CSS_CLASS: &str = "this-page";
const NAV_TREE_CSS_ID: &str = "nav-tree";

fn make_node_link(fsnode: &FsNode) -> Node {
    Node::link(fsnode.title(), &fsnode.url())
}

fn is_index_file(fsnode: &FsNode) -> bool {
    const INDEX_FILE: &str = "index.html";
    !fsnode.is_dir() && fsnode.name().map(String::as_str) == Some(INDEX_FILE)
}

fn make_nav_subtree<'a>(tree: &'a FsTree, mut fsnode: &'a FsNode, doc: &Document) -> Node {
    let mut entries = Vec::new();

    let mut children = Vec::new();
    for child in tree.children(fsnode.id) {
        if is_index_file(child) {
            fsnode = child;
        } else {
            let subtree = make_nav_subtree(tree, child, doc);
            if !subtree.is_empty() {
                children.push(subtree);
            }
        }
    }

    let mut node = make_node_link(fsnode);
    if fsnode.id == doc.fsnode {
        node.attr(CSS_CLASS_ATTR, THIS_PAGE_CSS_CLASS);
    }
    entries.push(node);

    if !children.is_empty() {
        entries.push(Node::list(children));
    } else if fsnode.is_dir() {
        return Node::empty();
    }

    Node::item(entries)
}

pub fn make_nav_tree(tree: &FsTree, doc: &Document) -> Node {
    let mut items = Vec::new();
    for child in tree.children(FsTree::ROOT) {
        let subtree = make_nav_subtree(tree, child, doc);
        if !subtree.is_empty() {
            items.push(subtree);
        }
    }
    Node::list(items).with_attr(CSS_ID_ATTR, NAV_TREE_CSS_ID)
}

pub fn make_page_path(tree: &FsTree, doc: &Document) -> Node {
    let mut nodes = Vec::new();
    let mut fsnode = tree.get(doc.fsnode);
    while let Some(entry) = fsnode
        && !entry.is_root()
    {
        nodes.push(make_node_link(entry));
        nodes.push(Node::text("/"));

        if is_index_file(entry) {
            // Skip indexed directory.
            fsnode = entry
                .parent
                .and_then(|id| tree.get(id))
                .and_then(|parent| parent.parent)
                .and_then(|id| tree.get(id));
        } else {
            fsnode = entry.parent.and_then(|id| tree.get(id));
        }
    }
    nodes.truncate(nodes.len() - 1);
    nodes.reverse();

    if nodes.len() > 1 {
        Node::heading(3, nodes)
    } else {
        Node::empty()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use render::test::{concat, test_render};

    fn make_doc(node: usize, title: &str) -> Document {
        Document {
            fsnode: node,
            document: vec![Node::heading(1, vec![Node::text(title)])],
            output: std::path::PathBuf::new(),
        }
    }

    #[test]
    fn test_nav_tree() {
        let mut tree = FsTree::new();
        let root = tree.add("index", FsTree::ROOT, None);
        let country = tree.add("country", root, Some("Country!".into()));
        tree.add("citya", country, None);
        tree.add("cityb", country, None);
        let node = super::make_nav_tree(&tree, &make_doc(country, "Country!"));

        assert_eq!(
            node,
            Node::list(vec![Node::item(vec![
                Node::link("index", "/index"),
                Node::list(vec![Node::item(vec![
                    Node::link("Country!", "/index/country")
                        .with_attr(CSS_CLASS_ATTR, THIS_PAGE_CSS_CLASS),
                    Node::list(vec![
                        Node::link("citya", "/index/country/citya"),
                        Node::link("cityb", "/index/country/cityb")
                    ])
                ])])
            ])])
            .with_attr(CSS_ID_ATTR, NAV_TREE_CSS_ID)
        );
    }

    #[test]
    fn test_nav_tree_render() {
        let mut tree = FsTree::new();
        let idx = tree.add("index", FsTree::ROOT, None);
        let page = tree.add("page", idx, Some("Page Title".into()));
        tree.add("child", page, None);

        assert_eq!(
            test_render(super::make_nav_tree(&tree, &make_doc(page, "Page Title"))),
            concat(&[
                &format!("<ul {CSS_ID_ATTR}=\"{NAV_TREE_CSS_ID}\">"),
                "  <li><a href=\"/index\">index</a>",
                "    <ul>",
                &format!("      <li><a href=\"/index/page\" {CSS_CLASS_ATTR}=\"{THIS_PAGE_CSS_CLASS}\">Page Title</a>"),
                "        <ul>",
                "          <li><a href=\"/index/page/child\">child</a></li>",
                "        </ul>",
                "      </li>",
                "    </ul>",
                "  </li>",
                "</ul>"
            ])
        )
    }

    #[test]
    fn test_empty_dir_excluded() {
        let mut tree = FsTree::new();
        let node = tree.add_dir("dir", FsTree::ROOT);
        assert_eq!(
            test_render(super::make_nav_tree(&tree, &make_doc(node, "dir"))),
            "<ul id=\"nav-tree\">\n</ul>"
        );
    }

    #[test]
    fn test_index_replaces_dir() {
        let mut tree = FsTree::new();
        let dir = tree.add_dir("dir", FsTree::ROOT);
        let idx = tree.add("index.html", dir, Some("Index".to_string()));
        assert_eq!(
            test_render(super::make_nav_tree(&tree, &make_doc(idx, "Index"))),
            concat(&[
                "<ul id=\"nav-tree\">",
                "  <li><a href=\"/dir/index.html\" class=\"this-page\">Index</a></li>",
                "</ul>"
            ])
        );
    }
}

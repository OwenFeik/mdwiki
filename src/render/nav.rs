use crate::{
    fstree::{FsNode, FsTree},
    model::Node,
    Document,
};

const CSS_CLASS_ATTR: &str = "class";
const CSS_ID_ATTR: &str = "id";
const THIS_PAGE_CSS_CLASS: &str = "this-page";
const NAV_TREE_CSS_ID: &str = "nav-tree";

fn make_nav_subtree<'a>(tree: &'a FsTree, mut fsnode: &'a FsNode, doc: &Document) -> Node {
    const INDEX_FILE: &str = "index.html";

    let mut entries = Vec::new();

    let mut children = Vec::new();
    for child in tree.children(fsnode.id()) {
        if !child.is_dir() && child.name().map(String::as_str) == Some(INDEX_FILE) {
            fsnode = child;
        } else {
            let subtree = make_nav_subtree(tree, child, doc);
            if !subtree.is_empty() {
                children.push(subtree);
            }
        }
    }

    let mut node = Node::link(fsnode.title(), &fsnode.url());
    if fsnode.id() == doc.fsnode {
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
        )
    }
}
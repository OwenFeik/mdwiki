use crate::{
    fstree::{FsNode, FsTree},
    model::Node,
    Document,
};

const CSS_CLASS_ATTR: &str = "class";
const CSS_ID_ATTR: &str = "id";
const THIS_PAGE_CSS_CLASS: &str = "this-page";
const CSS_ID_NAV_TREE: &str = "nav-tree";
const CSS_ID_NAV_BREADCRUMB: &str = "nav-breadcrumb";

fn capitalise_word(word: &str) -> String {
    match word {
        "a" | "and" | "at" | "is" | "of" | "to" => word.to_string(),
        _ if !word.is_empty() => format!(
            "{}{}",
            word.chars().next().unwrap().to_uppercase(),
            &word[1..]
        ),
        _ => word.to_string(),
    }
}

fn capitalise(title: &str) -> String {
    title
        .split(|c| c == ' ' || c == '-' || c == '_')
        .map(capitalise_word)
        .collect::<Vec<String>>()
        .join(" ")
}

fn make_node_link(fsnode: &FsNode) -> Node {
    Node::link(&capitalise(fsnode.title()), &fsnode.url())
}

fn make_nav_subtree<'a>(tree: &'a FsTree, mut fsnode: &'a FsNode, doc: &Document) -> Node {
    let mut entries = Vec::new();

    let mut children = Vec::new();
    for child in tree.children(fsnode.id) {
        if child.is_index_file() {
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
    Node::list(items).with_attr(CSS_ID_ATTR, CSS_ID_NAV_TREE)
}

pub fn make_nav_breadcrumb(tree: &FsTree, fsnode: &FsNode) -> Node {
    let mut nodes = Vec::new();
    let mut fsnode = fsnode.parent.and_then(|id| tree.get(id));
    while let Some(entry) = fsnode
        && !entry.is_root()
    {
        nodes.push(Node::text("/"));
        nodes.push(make_node_link(entry));

        if entry.is_index_file() {
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
    nodes.reverse();

    if nodes.len() > 2 {
        Node::heading(3, nodes).with_attr("id", CSS_ID_NAV_BREADCRUMB)
    } else {
        Node::empty()
    }
}

pub fn create_index(fsnode: &FsNode, children: &[&FsNode]) -> Vec<Node> {
    vec![
        Node::heading(1, vec![Node::text(&capitalise(fsnode.title()))]),
        Node::list(children.iter().map(|n| make_node_link(n)).collect()),
    ]
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
                Node::link("Index", "/index"),
                Node::list(vec![Node::item(vec![
                    Node::link("Country!", "/index/country")
                        .with_attr(CSS_CLASS_ATTR, THIS_PAGE_CSS_CLASS),
                    Node::list(vec![
                        Node::link("Citya", "/index/country/citya"),
                        Node::link("Cityb", "/index/country/cityb")
                    ])
                ])])
            ])])
            .with_attr(CSS_ID_ATTR, CSS_ID_NAV_TREE)
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
                &format!("<ul {CSS_ID_ATTR}=\"{CSS_ID_NAV_TREE}\">"),
                "  <li><a href=\"/index\">Index</a>",
                "    <ul>",
                &format!("      <li><a href=\"/index/page\" {CSS_CLASS_ATTR}=\"{THIS_PAGE_CSS_CLASS}\">Page Title</a>"),
                "        <ul>",
                "          <li><a href=\"/index/page/child\">Child</a></li>",
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

    #[test]
    fn test_capitalise() {
        assert_eq!(capitalise("tree at hill"), "Tree at Hill");
        assert_eq!(capitalise("sword of killing"), "Sword of Killing");
        assert_eq!(capitalise("the big town"), "The Big Town");
        assert_eq!(capitalise("magic is a resource"), "Magic is a Resource");
    }
}

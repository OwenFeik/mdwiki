use crate::{
    fstree::{FsNode, FsTree},
    model::Node,
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
    Node::link(&capitalise(fsnode.title()), fsnode.url())
}

fn make_nav_subtree<'a>(tree: &'a FsTree, mut fsnode: &'a FsNode, page: usize) -> Node {
    let mut entries = Vec::new();

    let mut children = Vec::new();
    for child in tree.children(fsnode.id()) {
        if child.is_index() {
            fsnode = child;
        } else {
            let subtree = make_nav_subtree(tree, child, page);
            if !subtree.is_empty() {
                children.push(subtree);
            }
        }
    }

    let mut node = make_node_link(fsnode);
    if fsnode.id() == page {
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

pub fn make_nav_tree(tree: &FsTree, page: usize) -> Node {
    let mut items = Vec::new();
    for child in tree.children(FsTree::ROOT) {
        let subtree = make_nav_subtree(tree, child, page);
        if !subtree.is_empty() {
            items.push(subtree);
        }
    }
    Node::list(items).with_attr(CSS_ID_ATTR, CSS_ID_NAV_TREE)
}

fn next_breadcrumb<'a>(tree: &'a FsTree, fsnode: &'a FsNode) -> Option<&'a FsNode> {
    if fsnode.is_index() {
        tree.get_parent(fsnode)
            .and_then(|parent| tree.get_parent(parent))
    } else {
        tree.get_parent(fsnode)
    }
}

pub fn make_nav_breadcrumb(tree: &FsTree, fsnode: &FsNode) -> Node {
    let mut nodes = Vec::new();
    let mut fsnode = next_breadcrumb(tree, fsnode);
    while let Some(entry) = fsnode
        && !entry.is_root()
    {
        nodes.push(Node::text("/"));
        nodes.push(make_node_link(entry));
        fsnode = next_breadcrumb(tree, entry);
    }
    nodes.reverse();

    if nodes.is_empty() {
        Node::empty()
    } else {
        Node::heading(3, nodes).with_attr("id", CSS_ID_NAV_BREADCRUMB)
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
    use crate::{model::File, render::test::assert_eq_lines};

    use super::*;
    use render::test::{concat, test_render};

    #[test]
    fn test_nav_tree() {
        let mut tree = FsTree::new();
        let dir = tree.add_dir(FsTree::ROOT, "index");
        let country = File::new(&mut tree, dir, "country", "Country!", Vec::new());
        File::new(&mut tree, country.fsnode(), "citya", "citya", Vec::new());
        File::new(&mut tree, country.fsnode(), "cityb", "cityb", Vec::new());
        let node = super::make_nav_tree(&tree, country.fsnode());

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
        let dir = tree.add_dir(FsTree::ROOT, "index");
        let page = File::new(&mut tree, dir, "page", "Page Title", Vec::new());
        File::new(&mut tree, page.fsnode(), "child", "child", Vec::new());

        assert_eq_lines(
            test_render(super::make_nav_tree(&tree, page.fsnode())),
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
        tree.add_dir(FsTree::ROOT, "dir");
        assert_eq!(
            test_render(super::make_nav_tree(&tree, 2)),
            "<ul id=\"nav-tree\">\n</ul>"
        );
    }

    #[test]
    fn test_index_replaces_dir() {
        let mut tree = FsTree::new();
        let dir = tree.add_dir(FsTree::ROOT, "dir");
        let idx = tree.add_index(dir, "index.html", "Index");
        assert_eq!(
            test_render(super::make_nav_tree(&tree, idx)),
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

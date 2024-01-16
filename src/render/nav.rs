use crate::model::{Doc, Node, WikiPage, WikiTree};

const CSS_CLASS_ATTR: &str = "class";
const CSS_ID_ATTR: &str = "id";
const CSS_CLASS_THIS_PAGE: &str = "nav-tree-selected";
const CSS_CLASS_BULLET: &str = "nav-tree-bullet";
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

fn make_page_link(page: &WikiPage) -> Node {
    Node::link(&capitalise(page.title()), page.url())
}

fn make_nav_subtree<'a>(tree: &'a WikiTree, mut current: &'a WikiPage, page: &'a WikiPage) -> Node {
    if current.is_media() {
        return Node::empty();
    }

    let mut children = Vec::new();
    for child in tree.children(current.id()) {
        if child.is_index() {
            current = child;
        } else {
            let subtree = make_nav_subtree(tree, child, page);
            if !subtree.is_empty() {
                children.push(subtree);
            }
        }
    }

    let mut link = make_page_link(current);
    if current.id() == page.id() {
        link.attr(CSS_CLASS_ATTR, CSS_CLASS_THIS_PAGE);
    }

    if !children.is_empty() {
        let mut node = Node::details(vec![link], vec![Node::list(children)]);

        if page.is_descendent_of(current.id())
            || (current.is_index()
                && current
                    .parent()
                    .map(|p| page.is_descendent_of(p))
                    .unwrap_or(false))
        {
            node.attr("open", "");
        }
        node
    } else if !current.is_dir() {
        Node::item(vec![
            Node::span(Vec::new()).with_attr(CSS_CLASS_ATTR, CSS_CLASS_BULLET),
            link,
        ])
    } else {
        Node::empty()
    }
}

pub fn make_nav_tree(tree: &WikiTree, page: &WikiPage) -> Node {
    let mut items = Vec::new();
    for child in tree.children(WikiTree::ROOT) {
        let subtree = make_nav_subtree(tree, child, page);
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

pub fn make_nav_breadcrumb(tree: &WikiTree, page: &WikiPage) -> Node {
    let mut nodes = Vec::new();
    let mut current = next_breadcrumb(tree, page);
    while let Some(entry) = current
        && !entry.is_root()
    {
        nodes.push(Node::text("/"));
        nodes.push(make_page_link(entry));
        current = next_breadcrumb(tree, entry);
    }
    nodes.reverse();

    if nodes.is_empty() {
        Node::empty()
    } else {
        Node::heading(3, nodes).with_attr("id", CSS_ID_NAV_BREADCRUMB)
    }
}

pub fn create_index(fsnode: &WikiPage, children: &[&WikiPage]) -> Doc {
    Doc::from(vec![
        Node::heading(1, vec![Node::text(&capitalise(fsnode.title()))]),
        Node::list(children.iter().map(|n| make_page_link(n)).collect()),
    ])
}

mod test {
    use crate::render::test::assert_eq_lines;

    use super::*;
    use render::test::{concat, test_render};

    #[test]
    fn test_nav_tree() {
        let mut tree = WikiTree::new();
        let dir = tree.add_dir(WikiTree::ROOT, "index");
        let country = tree.add_doc(dir, "country", "Country!", Doc::empty());
        tree.add_doc(country, "citya", "citya", Doc::empty());
        tree.add_doc(country, "cityb", "cityb", Doc::empty());
        let node = super::make_nav_tree(&tree, tree.get(country).unwrap());

        assert_eq!(
            node,
            Node::list(vec![Node::item(vec![Node::details(
                vec![Node::link("Index", "/index")],
                vec![Node::list(vec![Node::item(vec![Node::details(
                    vec![Node::link("Country!", "/index/country")
                        .with_attr(CSS_CLASS_ATTR, CSS_CLASS_THIS_PAGE)],
                    vec![Node::list(vec![
                        Node::item(vec![
                            Node::span(Vec::new()).with_attr(CSS_CLASS_ATTR, CSS_CLASS_BULLET),
                            Node::link("Citya", "/index/country/citya")
                        ]),
                        Node::item(vec![
                            Node::span(Vec::new()).with_attr(CSS_CLASS_ATTR, CSS_CLASS_BULLET),
                            Node::link("Cityb", "/index/country/cityb")
                        ]),
                    ])]
                )
                .with_attr("open", "")])])]
            )
            .with_attr("open", "")])])
            .with_attr(CSS_ID_ATTR, CSS_ID_NAV_TREE)
        );
    }

    #[test]
    fn test_nav_tree_render() {
        let mut tree = WikiTree::new();
        let dir = tree.add_dir(WikiTree::ROOT, "index");
        let page = tree.add_doc(dir, "page", "Page Title", Doc::empty());
        tree.add_doc(page, "child", "child", Doc::empty());

        assert_eq_lines(
            test_render(super::make_nav_tree(&tree, tree.get(page).unwrap())),
            concat(&[
                &format!("<ul {CSS_ID_ATTR}=\"{CSS_ID_NAV_TREE}\">"),
                "  <li>",
                "    <details open=\"\">",
                "      <summary><a href=\"/index\">Index</a></summary>",
                "      <ul>",
                "        <li>",
                "          <details open=\"\">",
                &format!("            <summary><a href=\"/index/page\" {CSS_CLASS_ATTR}=\"{CSS_CLASS_THIS_PAGE}\">Page Title</a></summary>"),
                "            <ul>",
                "              <li><span class=\"nav-tree-bullet\"></span> <a href=\"/index/page/child\">Child</a></li>",
                "            </ul>",
                "          </details>",
                "        </li>",
                "      </ul>",
                "    </details>",
                "  </li>",
                "</ul>"
            ])
        )
    }

    #[test]
    fn test_empty_dir_excluded() {
        let mut tree = WikiTree::new();
        let dir = tree.add_dir(WikiTree::ROOT, "dir");
        assert_eq!(
            test_render(super::make_nav_tree(&tree, tree.get(dir).unwrap())),
            "<ul id=\"nav-tree\">\n</ul>"
        );
    }

    #[test]
    fn test_index_replaces_dir() {
        let mut tree = WikiTree::new();
        let dir = tree.add_dir(WikiTree::ROOT, "dir");
        let idx = tree.add_index(dir, "index.html", "Index", Doc::empty());
        assert_eq!(
            test_render(super::make_nav_tree(&tree, tree.get(idx).unwrap())),
            concat(&[
                "<ul id=\"nav-tree\">",
                &format!(
                    "  <li><span {}=\"{}\"></span> <a href=\"/dir/index.html\" {}=\"{}\">Index</a></li>",
                    CSS_CLASS_ATTR,
                    CSS_CLASS_BULLET,
                    CSS_CLASS_ATTR,
                    CSS_CLASS_THIS_PAGE
                ),
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

    #[test]
    fn test_media_excluded_from_nav_tree() {
        let mut tree = WikiTree::new();
        let dir = tree.add_dir(WikiTree::ROOT, "dir");
        let page = tree.add_doc(dir, "doc.html", "Doc", Doc::empty());
        let images = tree.add_dir(dir, "images");
        tree.add_media(
            images,
            "image.png",
            "Image",
            std::path::PathBuf::from("./image.png"),
        );

        assert_eq_lines(
            test_render(super::make_nav_tree(&tree, tree.get(page).unwrap())),
            concat(&[
                "<ul id=\"nav-tree\">",
                "  <li>",
                "    <details open=\"\">",
                "      <summary><a href=\"/dir\">Dir</a></summary>",
                "      <ul>",
                "        <li><span class=\"nav-tree-bullet\"></span> <a href=\"/dir/doc.html\" class=\"nav-tree-selected\">Doc</a></li>",
                "      </ul>",
                "    </details>",
                "  </li>",
                "</ul>",
            ]),
        );
    }
}

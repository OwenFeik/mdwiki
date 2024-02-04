use test::css::{with_class, with_id};

use crate::{
    config::Config,
    model::{Doc, Id, Node, Style, WikiTree},
};

use super::html::*;
use super::nav::*;
use super::*;

fn assert_eq_lines<S1: AsRef<str>, S2: AsRef<str>>(actual: S1, expected: S2) {
    let (real, goal) = (actual.as_ref(), expected.as_ref());
    println!("=== Actual:");
    println!("{real}\n");
    println!("=== Expected:");
    println!("{goal}\n");
    for (la, lb) in actual.as_ref().lines().zip(expected.as_ref().lines()) {
        assert_eq!(la, lb, "Expected {la} to be {lb}.")
    }
}

fn concat(strings: &[&str]) -> String {
    let mut string = String::new();
    for s in strings {
        if !string.is_empty() {
            string.push('\n');
        }
        string.push_str(s.as_ref());
    }
    string
}

fn style() -> String {
    let mut style = String::new();
    style.push_str("    <style>\n      ");
    style.push_str(&indent(include_str!("res/style.css"), 3));
    style.push_str("\n    </style>\n");
    style.push_str("    <script>\n      ");
    style.push_str(&indent(include_str!("res/decrypt.js"), 3));
    style.push_str("\n    </script>");
    style
}

fn make_file(document: Doc, title: &str) -> (WikiTree, Id) {
    let mut tree = WikiTree::new();
    let file = tree.add_doc(WikiTree::ROOT, "file.html", title, document);
    (tree, file)
}

fn wrap_main(title: &str, main: &[&str]) -> String {
    let title = format!("    <title>{title}</title>");
    let css = style();
    let mut lines: Vec<String> = [
        "<html>",
        "  <head>",
        &title,
        &css,
        "  </head>",
        "  <body>",
        "    <div id=\"content\">",
        "      <main>",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();

    for line in main {
        lines.push(format!("{}{}", " ".repeat(4 * TABSIZE), line));
    }

    for line in &["      </main>", "    </div>", "  </body>", "</html>"] {
        lines.push(line.to_string());
    }

    concat(&lines.iter().map(|s| s.as_str()).collect::<Vec<&str>>())
}

fn test_render_document(doc: impl Into<Doc>, main: &[&str]) {
    let title = "Page Title";
    let (tree, page) = make_file(doc.into(), title);
    assert_eq_lines(
        render_document(&Config::none(), &tree, tree.get(page).unwrap()).unwrap(),
        wrap_main(title, main),
    );
}

#[test]
fn test_capitalise() {
    assert_eq!(capitalise("tree at hill"), "Tree at Hill");
    assert_eq!(capitalise("sword of killing"), "Sword of Killing");
    assert_eq!(capitalise("the big town"), "The Big Town");
    assert_eq!(capitalise("magic is a resource"), "Magic is a Resource");
}

const MD: &str = r#"
# Test Markdown File

This is a test markdown file. It should

* Parse lists
    * Including sub lists
    * And **bold** and *italics*
    * And [links](https://owen.feik.xyz) and ![images](https://example.org/example.jpg)

## Handle Subheadings
"#;

#[test]
fn test_render_heading() {
    test_render_document(
        vec![Node::heading(1, vec![Node::text("Hello World")])],
        &["<h1>Hello World</h1>"],
    );
}

#[test]
fn test_render_links() {
    assert_eq!(
        render_node(&Node::list(vec![
            Node::item(vec![
                Node::text("Click here:"),
                Node::link("Website", "https://owen.feik.xyz")
            ]),
            Node::image("image alt", "https://image.url")
        ])),
        concat!(
            "<ul>\n",
            "  <li>Click here: <a href=\"https://owen.feik.xyz\">Website</a></li>\n",
            "  <li><img src=\"https://image.url\" alt=\"image alt\"></li>\n",
            "</ul>"
        )
    )
}

#[test]
fn test_render_style() {
    assert_eq!(
        render_node(&Node::style(
            Style::Italic,
            vec![Node::style(Style::Bold, vec![Node::text("italic bold")])]
        )),
        concat!("<i><b>italic bold</b></i>")
    )
}

#[test]
fn test_escaping() {
    assert_eq!(
        escape("\"hello world\""),
        String::from("&quot;hello world&quot;")
    )
}

#[test]
fn test_full_document() {
    test_render_document(
        crate::parse::parse_document(MD.trim()),
        &[
            "<h1>Test Markdown File</h1>",
            "<p>",
            "  This is a test markdown file. It should",
            "</p>",
            "<ul>",
            "  <li>Parse lists",
            "    <ul>",
            "      <li>Including sub lists</li>",
            "      <li>And <b>bold</b> and <i>italics</i></li>",
            r#"      <li>And <a href="https://owen.feik.xyz">links</a> and <img src="https://example.org/example.jpg" alt="images"></li>"#,
            "    </ul>",
            "  </li>",
            "</ul>",
            "<h2>Handle Subheadings</h2>",
        ],
    );
}

#[test]
fn test_code() {
    assert_eq!(
        render_node(&Node::code("print(\"Hello World\")")),
        "<code>print(&quot;Hello World&quot;)</code>"
    );
}

#[test]
fn test_code_block() {
    assert_eq!(
        render_node(&Node::codeblock(Some("py"), "print(\"Hello World\")")),
        "<pre>\n  print(&quot;Hello World&quot;)\n</pre>"
    );
}

#[test]
fn test_details() {
    assert_eq_lines(
        render_node(&Node::details(
            vec![Node::link("index.html", "/")],
            vec![Node::list(vec![Node::link("child.html", "/child.html")])],
        )),
        concat(&[
            "<details>",
            "  <summary><a href=\"/\">index.html</a></summary>",
            "  <ul>",
            "    <li><a href=\"/child.html\">child.html</a></li>",
            "  </ul>",
            "</details>",
        ]),
    );
}

#[test]
fn test_link_doesnt_break_paragraph() {
    test_render_document(
        vec![
            Node::text("Some text"),
            Node::link("Link text", "http://url"),
        ],
        &[
            "<p>",
            "  Some text <a href=\"http://url\">Link text</a>",
            "</p>",
        ],
    );
}

#[test]
fn test_link_included_in_paragraph() {
    test_render_document(
        vec![
            Node::link("Link text", "http://url"),
            Node::text("Some text"),
        ],
        &[
            "<p>",
            "  <a href=\"http://url\">Link text</a> Some text",
            "</p>",
        ],
    )
}

#[test]
fn test_encrypted_nodes() {
    let config = &Config::default();
    let mut tree = WikiTree::new();
    let page = tree.add_doc(WikiTree::ROOT, "myfile.html", "My File", Doc::empty());
    let page = tree.get(page).unwrap();
    let heading = Node::heading(1, vec![Node::text("abcd")]).with_tags(vec!["dm".into()]);
    let nodes = vec![
        heading,
        Node::text("Some body text !!!"),
        Node::heading(1, vec![Node::text("efgh")]),
    ];
    let html = render_nodes_only(config, &tree, page, &nodes, false);
    assert!(!html.contains("<h1>abcd</h1>"));
    assert!(!html.contains("Some body text !!!"));
    assert!(html.contains("<h1>efgh</h1>"));
    assert!(html.starts_with("<span"));
    assert!(html.contains("class=\"secret\""));
    assert!(html.contains("nonces=\""));
    assert!(html.contains("tags=\"dm\""));
}

fn make_state<'a>(
    tree: &'a WikiTree,
    page: usize,
    html: &'a mut Html,
    config: &'a Config,
) -> RenderState<'a> {
    RenderState {
        tree,
        page: tree.get(page).unwrap(),
        config,
        html,
    }
}

#[test]
fn test_nav_tree() {
    let mut tree = WikiTree::new();
    let dir = tree.add_dir(WikiTree::ROOT, "index");
    let country = tree.add_doc(dir, "country", "Country!", Doc::empty());
    tree.add_doc(country, "citya", "Citya", Doc::empty());
    tree.add_doc(country, "cityb", "Cityb", Doc::empty());

    assert_eq!(
        make_nav_tree(&make_state(
            &tree,
            country,
            &mut Html::new(),
            &Config::none()
        )),
        with_id(
            Node::list(vec![Node::item(vec![Node::details(
                vec![Node::link("Index", "/index")],
                vec![Node::list(vec![Node::item(vec![Node::details(
                    vec![with_class(
                        Node::link("Country!", "/index/country"),
                        "nav-tree-selected"
                    )],
                    vec![Node::list(vec![
                        Node::item(vec![
                            with_class(Node::span(Vec::new()), "nav-tree-bullet"),
                            Node::link("Citya", "/index/country/citya")
                        ]),
                        Node::item(vec![
                            with_class(Node::span(Vec::new()), "nav-tree-bullet"),
                            Node::link("Cityb", "/index/country/cityb")
                        ]),
                    ])]
                )
                .with_attr("open", "")])])]
            )
            .with_attr("open", "")])]),
            "nav-tree"
        )
    );
}

#[test]
fn test_nav_tree_render() {
    let mut tree = WikiTree::new();
    let dir = tree.add_dir(WikiTree::ROOT, "index");
    let page = tree.add_doc(dir, "page", "Page Title", Doc::empty());
    tree.add_doc(page, "child", "Child", Doc::empty());

    assert_eq_lines(
        render_node(&make_nav_tree(&make_state(&tree, page, &mut Html::new(), &Config::none()))),
        concat(&[
            "<ul id=\"nav-tree\">",
            "  <li>",
            "    <details open=\"\">",
            "      <summary><a href=\"/index\">Index</a></summary>",
            "      <ul>",
            "        <li>",
            "          <details open=\"\">",
            "            <summary><a href=\"/index/page\" class=\"nav-tree-selected\">Page Title</a></summary>",
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
        render_node(&make_nav_tree(&make_state(
            &tree,
            dir,
            &mut Html::new(),
            &Config::none()
        ))),
        "<ul id=\"nav-tree\">\n</ul>"
    );
}

#[test]
fn test_index_replaces_dir() {
    let mut tree = WikiTree::new();
    let dir = tree.add_dir(WikiTree::ROOT, "dir");
    let idx = tree.add_index(dir, "index.html", "Index", Doc::empty());
    assert_eq!(
        render_node(&make_nav_tree(&make_state(&tree, idx, &mut Html::new(), &Config::none()))),
        concat(&[
            "<ul id=\"nav-tree\">",
            "  <li><span class=\"nav-tree-bullet\"></span> <a href=\"/dir/index.html\" class=\"nav-tree-selected\">Index</a></li>",
            "</ul>"
        ])
    );
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
        render_node(&make_nav_tree(&make_state(&tree,page, &mut Html::new(), &Config::none()))),
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

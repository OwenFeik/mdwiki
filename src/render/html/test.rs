use crate::{
    model::{Doc, Id, Node},
    render::test::{assert_eq_lines, concat},
};

use super::*;

fn style() -> String {
    let mut style = String::new();
    style.push_str("    <style>\n      ");
    style.push_str(&indent(include_str!("res/style.css"), 3));
    style.push_str("\n    </style>");
    style
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
        super::escape("\"hello world\""),
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

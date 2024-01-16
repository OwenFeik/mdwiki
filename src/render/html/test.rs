use crate::{
    model::{Doc, Id, Node},
    render::test::{assert_eq_lines, concat},
};

use super::*;

fn style() -> String {
    let mut style = String::new();
    style.push_str("    <style href=\"");
    style.push_str(FONT);
    style.push_str("\" rel=\"stylesheet\"></style>\n");
    style.push_str("    <style>\n      ");
    style.push_str(&super::indent(include_str!("res/style.css"), 3));
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

#[test]
fn test_render_heading() {
    let (tree, page) = make_file(
        vec![Node::heading(1, vec![Node::text("Hello World")])].into(),
        "Hello World",
    );
    assert_eq_lines(
        super::render_document(&Config::none(), Some(&tree), tree.get(page).unwrap()).unwrap(),
        concat(&[
            "<html>",
            "  <head>",
            "    <title>Hello World</title>",
            &style(),
            "  </head>",
            "  <body>",
            "    <div id=\"content\">",
            "      <main>",
            "        <h1>Hello World</h1>",
            "      </main>",
            "    </div>",
            "  </body>",
            "</html>",
        ]),
    )
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
fn test_integration() {
    let (tree, page) = make_file(
        crate::parse::parse_document(MD.trim()),
        "Test Markdown File",
    );
    assert_eq_lines(
        super::render_document(&Config::none(), Some(&tree), tree.get(page).unwrap()).unwrap(),
        concat(&[
            "<html>",
            "  <head>",
            "    <title>Test Markdown File</title>",
            &style(),
            "  </head>",
            "  <body>",
            "    <div id=\"content\">",
            "      <main>",
            "        <h1>Test Markdown File</h1>",
            "        <p>",
            "          This is a test markdown file. It should",
            "        </p>",
            "        <ul>",
            "          <li>Parse lists",
            "            <ul>",
            "              <li>Including sub lists</li>",
            "              <li>And <b>bold</b> and <i>italics</i></li>",
            r#"              <li>And <a href="https://owen.feik.xyz">links</a> and <img src="https://example.org/example.jpg" alt="images"></li>"#,
            "            </ul>",
            "          </li>",
            "        </ul>",
            "        <h2>Handle Subheadings</h2>",
            "      </main>",
            "    </div>",
            "  </body>",
            "</html>",
        ]),
    )
}

#[test]
fn test_code() {
    let mut html = super::Html::new();
    super::render(&Node::code("print(\"Hello World\")"), &mut html);
    assert_eq!(&html.content, "<code>print(&quot;Hello World&quot;)</code>");
}

#[test]
fn test_code_block() {
    let mut html = Html::new();
    render(
        &Node::codeblock(Some("py"), "print(\"Hello World\")"),
        &mut html,
    );
    assert_eq!(
        html.content.trim(),
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
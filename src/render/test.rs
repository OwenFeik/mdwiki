use crate::{
    config::Config,
    fstree::FsTree,
    model::{Node, Style},
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

pub fn assert_eq_lines<S1: AsRef<str>, S2: AsRef<str>>(actual: S1, expected: S2) {
    for (la, lb) in actual.as_ref().lines().zip(expected.as_ref().lines()) {
        assert_eq!(la, lb, "Expected {la} to be {lb}.")
    }
}

pub fn concat(strings: &[&str]) -> String {
    let mut string = String::new();
    for s in strings {
        if !string.is_empty() {
            string.push('\n');
        }
        string.push_str(s);
    }
    string
}

pub fn test_render(node: Node) -> String {
    let mut html = super::Html::new();
    render(&node, &mut html);
    html.content.trim().to_string()
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

fn make_file(document: Vec<Node>, title: &str) -> (FsTree, File) {
    let mut tree = FsTree::new();
    let file = File::new(&mut tree, FsTree::ROOT, "file.html", title, document);
    (tree, file)
}

#[test]
fn test_render_heading() {
    let (tree, doc) = make_file(
        vec![Node::heading(1, vec![Node::text("Hello World")])],
        "Hello World",
    );
    assert_eq_lines(
        super::render_document(&Config::none(), &tree, &doc),
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
        test_render(Node::list(vec![
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
        test_render(Node::style(
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
    let (tree, doc) = make_file(
        crate::parse::parse_document(MD.trim()),
        "Test Markdown File",
    );
    assert_eq_lines(
        super::render_document(&Config::none(), &tree, &doc),
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
        test_render(Node::details(
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

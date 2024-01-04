use crate::{
    config::Config,
    fstree::FsTree,
    model::{Node, Style},
};

fn style() -> String {
    let mut style = String::new();
    style.push_str("    <style>\n      ");
    style.push_str(&super::indent(include_str!("res/style.css"), 3));
    style.push_str("\n    </style>");
    style
}

fn concat(strings: &[&str]) -> String {
    let mut string = String::new();
    for s in strings {
        if !string.is_empty() {
            string.push('\n');
        }
        string.push_str(s);
    }
    string
}

fn test_render(node: Node) -> String {
    let mut html = super::Html::new();
    super::render(&node, &mut html);
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

#[test]
fn test_render_heading() {
    assert_eq!(
        super::render_document(
            &Config::none(),
            &FsTree::new(),
            0,
            &[Node::Heading(1, vec![Node::text("Hello World")])]
        ),
        concat(&[
            "<html>",
            "  <head>",
            &style(),
            "  </head>",
            "  <body>",
            "    <main>",
            "      <h1>Hello World</h1>",
            "    </main>",
            "  </body>",
            "</html>"
        ])
    )
}

#[test]
fn test_render_links() {
    assert_eq!(
        test_render(Node::List(vec![
            Node::Item(vec![
                Node::text("Click here:"),
                Node::link("Website", "https://owen.feik.xyz")
            ]),
            Node::Item(vec![Node::image("image alt", "https://image.url")])
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
        test_render(Node::Style(
            Style::Italic,
            vec![Node::Style(Style::Bold, vec![Node::text("italic bold")])]
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
    assert_eq!(
        super::render_document(
            &Config::none(),
            &FsTree::new(),
            0,
            &crate::parse::parse_document(MD.trim())
        ),
        concat(&[
            "<html>",
            "  <head>",
            &style(),
            "  </head>",
            "  <body>",
            "    <main>",
            "      <h1>Test Markdown File</h1>",
            "      <p>",
            "        This is a test markdown file. It should",
            "      </p>",
            "      <ul>",
            "        <li>Parse lists",
            "          <ul>",
            "            <li>Including sub lists</li>",
            "            <li>And <b>bold</b> and <i>italics</i></li>",
            r#"            <li>And <a href="https://owen.feik.xyz">links</a>and <img src="https://example.org/example.jpg" alt="images"></li>"#,
            "          </ul>",
            "        </li>",
            "      </ul>",
            "      <h2>Handle Subheadings</h2>",
            "    </main>",
            "  </body>",
            "</html>"
        ])
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
    let mut html = super::Html::new();
    super::render(
        &Node::Codeblock(
            Some(String::from("py")),
            String::from("print(\"Hello World\")"),
        ),
        &mut html,
    );
    assert_eq!(
        html.content.trim(),
        "<pre>\n  print(&quot;Hello World&quot;)\n</pre>"
    );
}

#[test]
fn test_nav_tree() {
    let mut tree = FsTree::new();
    let root = tree.add("index", FsTree::ROOT);
    let country = tree.add("country", root);
    tree.add("citya", country);
    tree.add("cityb", country);
    let node = super::make_nav_tree(&tree);

    dbg!(&node);
    assert_eq!(
        node,
        Node::List(vec![
            Node::Item(vec![Node::link("index", "/index")]),
            Node::Item(vec![Node::List(vec![
                Node::Item(vec![Node::link("country", "/index/country")]),
                Node::Item(vec![Node::List(vec![
                    Node::Item(vec![Node::link("citya", "/index/country/citya")]),
                    Node::Item(vec![Node::link("cityb", "/index/country/cityb")])
                ])])
            ])])
        ])
    );
}

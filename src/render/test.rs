use crate::model::{Node, Style};

#[test]
fn test_render_heading() {
    assert_eq!(
        &super::render_document(&Node::Document(vec![
            Node::Heading(vec![Node::text("Hello World")])
        ])),
        concat!(
            "<html>\n",
            " <head>\n",
            " </head>\n",
            " <body>\n",
            "  <h1>Hello World</h1>\n",
            " </body>\n",
            "</html>"
        )
    );
}

#[test]
fn test_render_links() {
    assert_eq!(
        &super::render_document(&Node::List(vec![
            Node::Item(vec![
                Node::text("Click here:"),
                Node::link("Website", "https://owen.feik.xyz")
            ]),
            Node::Item(vec![
                Node::image("image alt", "https://image.url")
            ])
        ])),
        concat!(
            "<ul>\n",
            " <li>Click here:<a href=\"https://owen.feik.xyz\">Website</a></li>\n",
            " <li><img src=\"https://image.url\" alt=\"image alt\"></li>\n",
            "</ul>"
        )
    );
}

#[test]
fn test_render_style() {
    assert_eq!(
        &super::render_document(&Node::Style(
            Style::Italic, vec![
                Node::Style(Style::Bold, vec![Node::text("italic bold")])
            ])
        ),
        concat!(
            "<i><b>italic bold</b></i>"
        )
    );
}

#[test]
fn test_escaping() {
    assert_eq!(
        super::escape("\"hello world\""),
        String::from("&quot;hello world&quot;")
    );
}

use crate::model::{Node, Style};

#[test]
fn test_parse_heading() {
    assert_eq!(
        super::parse_document("# Title of Document"),
        Node::Document(vec![Node::Heading(vec![Node::text("Title of Document")])])
    );
}

#[test]
fn test_parse_heading_text() {
    assert_eq!(
        super::parse_document(
            r#"
                some text
                # Heading one
                Some other text
                more text
            "#
        ),
        Node::Document(vec![
            Node::text("some text"),
            Node::Heading(vec![Node::text("Heading one")]),
            Node::text("Some other text more text")
        ])
    )
}

#[test]
fn test_parse_bold() {
    assert_eq!(
        super::parse_document("Some text **bolded text** end"),
        Node::Document(vec![
            Node::text("Some text"),
            Node::Style(Style::Bold, vec![Node::text("bolded text")]),
            Node::text("end")
        ])
    )
}

#[test]
fn test_parse_bold_italic() {
    assert_eq!(
        super::parse_document("# Normal *italic* **bold** *italic* normal"),
        Node::Document(vec![Node::Heading(vec![
            Node::text("Normal"),
            Node::Style(Style::Italic, vec![Node::text("italic")]),
            Node::Style(Style::Bold, vec![Node::text("bold")]),
            Node::Style(Style::Italic, vec![Node::text("italic")]),
            Node::text("normal")
        ])])
    )
}

#[test]
fn test_line_break() {
    assert_eq!(
        super::parse_document("line\nno break\n  \nbroken"),
        Node::Document(vec![Node::text("line no break"), Node::text("broken")])
    )
}

#[test]
fn test_list() {
    assert_eq!(
        super::parse_document(
            r#"
                # Heading
                * List item
                    * Sub list item
                    * Another sub item
                * Another list item
            "#
        ),
        Node::Document(vec![
            Node::Heading(vec![Node::text("Heading")]),
            Node::List(vec![
                Node::Item(vec![
                    Node::text("List item"),
                    Node::List(vec![
                        Node::Item(vec![Node::text("Sub list item")]),
                        Node::Item(vec![Node::text("Another sub item")])
                    ]),
                ]),
                Node::Item(vec![Node::text("Another list item")])
            ])
        ])
    )
}

#[test]
fn test_end_list() {
    assert_eq!(
        super::parse_document("* list item\ncontinued\n\nended"),
        Node::Document(vec![
            Node::List(vec![Node::Item(vec![Node::text("list item continued")])]),
            Node::text("ended")
        ])
    )
}

#[test]
fn test_parse_list_item() {
    assert_eq!(
        super::parse_list_item("* list item\n* next item").1,
        Node::Item(vec![Node::text("list item")])
    )
}

#[test]
fn test_parse_list_sub_item() {
    assert_eq!(
        super::parse_list_item("* list item\n  * sub item").1,
        Node::Item(vec![
            Node::text("list item"),
            Node::List(vec![Node::Item(vec![Node::text("sub item")])])
        ])
    )
}

#[test]
fn test_parse_list() {
    assert_eq!(
        super::parse_list("* list item\n  * sub item\n* next item").1,
        Node::List(vec![
            Node::Item(vec![
                Node::text("list item"),
                Node::List(vec![Node::Item(vec![Node::text("sub item")])])
            ]),
            Node::Item(vec![Node::text("next item")])
        ])
    )
}

#[test]
fn test_parse_link() {
    assert_eq!(
        super::parse_document("[My Website](https://owen.feik.xyz)"),
        Node::Document(vec![Node::Link(
            String::from("My Website"),
            String::from("https://owen.feik.xyz")
        )])
    )
}

#[test]
fn test_parse_not_link() {
    assert_eq!(
        super::parse_node("[Text in brackets] other text").1,
        Node::text("[Text in brackets] other text")
    )
}

#[test]
fn test_parse_image() {
    assert_eq!(
        super::parse_node("![Image caption](https://image.url)").1,
        Node::image("Image caption", "https://image.url")
    )
}

#[test]
fn test_strikethrough() {
    assert_eq!(
        super::parse_node("~struckthrough~").1,
        Node::Style(Style::Strikethrough, vec![Node::text("struckthrough")])
    )
}

#[test]
fn test_code() {
    assert_eq!(
        super::parse_document(concat!(
            "# Title With `Inline Code`\n",
            "```lang\ncode block line 1\nline 2\n\nline 3```\n"
        )),
        Node::Document(vec![
            Node::Heading(vec![
                Node::text("Title With"),
                Node::code("Inline Code")
            ]),
            Node::Codeblock(
                Some(String::from("lang")),
                String::from("code block line 1\nline 2\n\nline 3")
            )
        ])
    )
}

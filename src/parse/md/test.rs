use crate::model::{Doc, Node, Style};

#[test]
fn test_parse_heading() {
    assert_eq!(
        super::parse_document("# Title of Document"),
        Doc::from(vec![Node::heading(
            1,
            vec![Node::text("Title of Document")]
        )])
    );
}

#[test]
fn test_parse_heading_text() {
    assert_eq!(
        super::parse_document(
            r#"
                some text
                ## Heading two
                Some other text
                more text
            "#
        ),
        Doc::from(vec![
            Node::text("some text"),
            Node::heading(2, vec![Node::text("Heading two")]),
            Node::text("Some other text more text")
        ])
    )
}

#[test]
fn test_parse_bold() {
    assert_eq!(
        super::parse_document("Some text **bolded text** end"),
        Doc::from(vec![
            Node::text("Some text"),
            Node::style(Style::Bold, vec![Node::text("bolded text")]),
            Node::text("end")
        ])
    )
}

#[test]
fn test_parse_bold_italic() {
    assert_eq!(
        super::parse_document("# Normal *italic* **bold** *italic* normal"),
        Doc::from(vec![Node::heading(
            1,
            vec![
                Node::text("Normal"),
                Node::style(Style::Italic, vec![Node::text("italic")]),
                Node::style(Style::Bold, vec![Node::text("bold")]),
                Node::style(Style::Italic, vec![Node::text("italic")]),
                Node::text("normal")
            ]
        )])
    )
}

#[test]
fn test_line_break() {
    assert_eq!(
        super::parse_document("line\nno break\n  \nbroken"),
        Doc::from(vec![Node::text("line no break"), Node::text("broken")])
    )
}

#[test]
fn test_list() {
    assert_eq!(
        super::parse_document(
            r#"
                ### Heading
                * List item
                    * Sub list item
                    * Another sub item
                * Another list item
            "#
        ),
        Doc::from(vec![
            Node::heading(3, vec![Node::text("Heading")]),
            Node::list(vec![
                Node::item(vec![
                    Node::text("List item"),
                    Node::list(vec![
                        Node::text("Sub list item"),
                        Node::text("Another sub item")
                    ]),
                ]),
                Node::text("Another list item")
            ])
        ])
    )
}

#[test]
fn test_end_list() {
    assert_eq!(
        super::parse_document("* list item\ncontinued\n\nended"),
        Doc::from(vec![
            Node::list(vec![Node::text("list item continued")]),
            Node::text("ended")
        ])
    )
}

#[test]
fn test_parse_list_item() {
    assert_eq!(
        super::parse_list_item("* list item\n* next item").1,
        Node::item(vec![Node::text("list item")])
    )
}

#[test]
fn test_parse_list_sub_item() {
    assert_eq!(
        super::parse_list_item("* list item\n  * sub item").1,
        Node::item(vec![
            Node::text("list item"),
            Node::list(vec![Node::text("sub item")])
        ])
    )
}

#[test]
fn test_parse_list() {
    assert_eq!(
        super::parse_list("* list item\n  * sub item\n* next item").1,
        Node::list(vec![
            Node::item(vec![
                Node::text("list item"),
                Node::list(vec![Node::text("sub item")])
            ]),
            Node::text("next item")
        ])
    )
}

#[test]
fn test_parse_link() {
    assert_eq!(
        super::parse_document("[My Website](https://owen.feik.xyz)"),
        Doc::from(vec![Node::link("My Website", "https://owen.feik.xyz")])
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
fn test_parse_image_no_url() {
    assert_eq!(
        super::parse_node("![World map]()").1,
        Node::image("World map", "")
    )
}

#[test]
fn test_strikethrough() {
    assert_eq!(
        super::parse_node("~struckthrough~").1,
        Node::style(Style::Strikethrough, vec![Node::text("struckthrough")])
    )
}

#[test]
fn test_code() {
    assert_eq!(
        super::parse_document(concat!(
            "## Title With `Inline Code`\n",
            "```lang\ncode block line 1\nline 2\n\nline 3```\n"
        )),
        Doc::from(vec![
            Node::heading(2, vec![Node::text("Title With"), Node::code("Inline Code")]),
            Node::codeblock(Some("lang"), "code block line 1\nline 2\n\nline 3")
        ])
    )
}

#[test]
fn test_parse_table() {
    assert_eq!(
        super::parse_document("| Column 1 | Column 2 |\n| some | **bold** |"),
        Doc::from(vec![Node::table(vec![
            vec![vec![Node::text("Column 1")], vec![Node::text("Column 2")]],
            vec![
                vec![Node::text("some")],
                vec![Node::style(Style::Bold, vec![Node::text("bold")])]
            ]
        ])])
    )
}

#[test]
fn test_last_el_image() {
    assert_eq!(
        super::parse_document("# Nations\n\n![World map]()\n"),
        vec![
            Node::heading(1, vec![Node::text("Nations")]),
            Node::image("World map", "")
        ]
        .into()
    )
}

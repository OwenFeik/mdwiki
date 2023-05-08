#![allow(dead_code)]

use std::str::pattern::Pattern;

use crate::model::node::{Node, Style};

const CONTROL: &str = "#*\n";

fn drop_n<'a>(input: &'a str, n: usize) -> &'a str {
    if input.len() <= n {
        ""
    } else {
        &input[n..]
    }
}

fn drop_first<'a>(input: &'a str) -> &'a str {
    drop_n(input, 1)
}

fn consume<'a, P>(input: &'a str, condition: P) -> (&'a str, &'a str)
where
    P: Pattern<'a>,
{
    if let Some(i) = input.find(condition) {
        (&input[i..], &input[0..i])
    } else {
        ("", input)
    }
}

fn consume_line<'a>(input: &'a str) -> (&'a str, &'a str) {
    let (rest, line) = consume(input, '\n');
    (drop_first(rest), line)
}

fn consume_whitespace<'a>(input: &'a str) -> &'a str {
    let rest = consume(input, char::is_whitespace).0;
    if rest.is_empty() {
        ""
    } else {
        &rest[1..]
    }
}

fn consume_until_any<'a>(input: &'a str, chars: &str) -> (&'a str, &'a str) {
    consume(input, |c| chars.contains(c))
}

fn consume_chars<'a>(input: &'a str, chars: &str) -> (&'a str, &'a str) {
    consume(input, |c| !chars.contains(c))
}

fn parse_heading<'a>(input: &'a str) -> (&'a str, Node) {
    let (rest, _) = consume_chars(input, "#");
    let (rest, text) = consume_line(rest);
    (rest, Node::Heading(parse(text, false)))
}

fn parse_style<'a>(input: &'a str) -> (&'a str, Node) {
    match input {
        input if input.starts_with("**") => {
            let (rest, text) = consume(drop_n(input, 2), "**");
            let nodes = parse(text, false);
            (drop_n(rest, 2), Node::Style(Style::Bold, nodes))
        }
        input if input.starts_with('*') => {
            let (rest, text) = consume(drop_first(input), '*');
            (drop_first(rest), Node::Style(Style::Italic, parse(text, false)))
        }
        _ => ("", Node::Document(vec![])),
    }
}

fn parse_text<'a>(input: &'a str) -> (&'a str, Vec<Node>) {
    let mut node = Node::Text(String::new());
    let mut nodes = Vec::new();
    let mut rest = input;
    while {
        let text;
        (rest, text) = consume_until_any(rest, CONTROL);

        // Empty line, new text element.
        if text.trim().is_empty() && !node.is_empty() {
            nodes.push(node);
            node = Node::Text(String::new());
        }
        node.add_text(text);

        let text;
        (rest, text) = consume_chars(rest, "\n");
        !text.is_empty()
    } {}

    if !node.is_empty() {
        nodes.push(node);
    }

    (rest, nodes)
}

fn parse_list(input: &str) {
    
}

fn parse(input: &str, at_line_start: bool) -> Vec<Node> {
    let mut nodes = Vec::new();
    let mut rest = input;
    while !rest.is_empty() {
        match rest.chars().nth(0) {
            None => break,
            Some('#') => {
                let heading;
                (rest, heading) = parse_heading(rest);
                nodes.push(heading);
            }
            Some('*') if at_line_start => {}
            Some('*') => {
                    let styled;
                (rest, styled) = parse_style(rest);
                nodes.push(styled);
            }
            _ => {
                let text;
                (rest, text) = parse_text(rest);
                nodes.extend(text)
            }
        }
    }

    nodes
}

fn parse_document(input: &str) -> Node {
    Node::Document(parse(input, true))
}

#[cfg(test)]
mod test {
    use std::vec;

    use crate::model::node::{Node, Style};

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
                    Node::Item(vec![Node::text("Another sub item")])
                ])
            ])
        )
    }
}

#![allow(dead_code)]

use std::str::pattern::Pattern;

use crate::model::node::{Node, Style};

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
    (rest, Node::Heading(parse(text)))
}

fn parse_style<'a>(input: &'a str) -> (&'a str, Node) {
    match input {
        input if input.starts_with("**") => {
            let (rest, text) = consume(drop_n(input, 2), "**");
            let nodes = parse(text);
            (drop_n(rest, 2), Node::Style(Style::Bold, nodes))
        },
        input if input.starts_with('*') => {
            let (rest, text) = consume(drop_first(input), '*');
            (drop_first(rest), Node::Style(Style::Italic, parse(text)))
        }
        _ => ("", Node::Document(vec![]))
    }
}

fn parse(input: &str) -> Vec<Node> {
    let mut nodes = Vec::new();
    let mut rest = input;
    let mut node = String::new();
    while !rest.is_empty() {
        let text;
        (rest, text) = consume_until_any(rest, "*#\n");
        
        if !node.is_empty() {
            node.push('\n');
        }
        node.push_str(text.trim());

        if rest.starts_with('#') {
            if !node.is_empty() {
                nodes.push(Node::text(&node));
                node = String::new();
            }

            let heading;
            (rest, heading) = parse_heading(rest);
            nodes.push(heading);
        } else if rest.starts_with('*') {
            if !node.is_empty() {
                nodes.push(Node::text(&node));
                node = String::new();
            }

            let styled;
            (rest, styled) = parse_style(rest);
            nodes.push(styled);
        }

        rest = consume_whitespace(rest);
    }


    if !node.is_empty() {
        nodes.push(Node::text(&node));
    }

    nodes
}

fn parse_document(input: &str) -> Node {
    Node::Document(parse(input))
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
                Node::text("Some other text\nmore text")
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
            Node::Document(vec![
                Node::Heading(vec![
                    Node::text("Normal"),
                    Node::Style(Style::Italic, vec![Node::text("italic")]),
                    Node::Style(Style::Bold, vec![Node::text("bold")]),
                    Node::Style(Style::Italic, vec![Node::text("italic")]),
                    Node::text("normal")
                ])
            ])
        )
    }
}

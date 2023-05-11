#![allow(dead_code)]

use std::str::pattern::Pattern;

use crate::model::node::{Node, Style};

const CONTROL: &str = "*#";

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

fn first_char(input: &str) -> Option<char> {
    input.chars().nth(0)
}

fn first_non_whitespace(input: &str) -> Option<char> {
    input.trim_start().chars().nth(0)
}


fn starts_with_any(input: &str, chars: &str) -> bool {
    if let Some(c) = first_char(input) {
        chars.contains(c)  
    } else {
        false
    }
}

fn starts_with_empty_line(input: &str) -> bool {
    let mut line_passed = false;
    for c in input.chars() {
        if c == '\n' {
            if line_passed {
                return true;
            }
            line_passed = true;
        } else if !c.is_whitespace() {
            break;
        }
    }
    return false;
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
    let rest = consume(input, |c: char| !c.is_whitespace()).0;
    if rest.is_empty() {
        ""
    } else {
        rest
    }
}

fn consume_until_any<'a>(input: &'a str, chars: &str) -> (&'a str, &'a str) {
    consume(input, |c| chars.contains(c))
}

fn consume_chars<'a>(input: &'a str, chars: &str) -> (&'a str, &'a str) {
    consume(input, |c| !chars.contains(c))
}

fn parse_heading<'a>(input: &'a str) -> (&'a str, Node) {
    let (rest, _) = consume_chars(input.trim_start(), "#");
    let (rest, text) = consume_line(rest.trim_start());
    (rest, Node::Heading(parse(text)))
}

fn parse_style<'a>(input: &'a str) -> (&'a str, Node) {
    match consume_whitespace(input) {
        input if input.starts_with("**") => {
            let (rest, text) = consume(drop_n(input, 2), "**");
            (drop_n(rest, 2), Node::Style(Style::Bold, parse(text)))
        }
        input if input.starts_with('*') => {
            let (rest, text) = consume(drop_first(input), '*');
            (drop_first(rest), Node::Style(Style::Italic, parse(text)))
        }
        _ => parse_node(input),
    }
}

fn parse_text<'a>(input: &'a str) -> (&'a str, Node) {
    let mut node = Node::Text(String::new());
    let mut rest = input;
    while !starts_with_any(rest.trim_start(), CONTROL) {
        if first_char(rest) == Some('\n') {
            rest = drop_first(rest);
        }

        let text;
        (rest, text) = consume(rest, |c| CONTROL.contains(c) || c == '\n');

        // Empty line, new text element.
        if text.trim().is_empty() {
            break;
        }
        node.add_text(text);
    }

    (rest, node)
}

fn list_prefix_size(input: &str) -> Option<usize> {
    let mut n = 0;
    for c in input.chars() {
        match c {
            '*' => return Some(n),
            '\n' => n = 0,
            _ if c.is_whitespace() => n += 1,
            _ => break
        }
    }
    None
}

fn parse_list_item<'a>(input: &'a str) -> (&'a str, Node) {
    let prefix_size = list_prefix_size(input).expect(
        &format!("Invalid list item start: {}", &input[..16])
    );
    let mut nodes = Vec::new();
    let mut rest = input;
    while !starts_with_empty_line(rest) {
        if first_non_whitespace(rest) == Some('*') {
            let before = rest.len();
            rest = consume_whitespace(rest);
            let n = before - rest.len();
    
            // A new list item.        
            if n == prefix_size {
                return (rest, Node::Item(nodes))
            }
        }

        while first_char(input) != Some('\n') {
            let node;
            (rest, node) = parse_node(rest);
            nodes.push(node);    
        }
    }
    (rest, Node::Item(nodes))
}

fn parse_list<'a>(input: &'a str) -> (&'a str, Node) {
    let mut nodes = Vec::new();

    let prefix_size = list_prefix_size(input);

    let mut rest = input;
    while list_prefix_size(rest) == prefix_size {
        let node;
        (rest, node) = parse_list_item(rest);
        nodes.push(node);
    }

    (rest, Node::List(nodes))
}

fn parse_node<'a>(input: &'a str) -> (&'a str, Node) {
    let mut at_line_start = true;
    for c in input.chars() {
        match c {
            '\n' => break,
            _ if c.is_whitespace() => (),
            _ => {
                at_line_start = false;
                break;
            }
        }
    }

    match first_non_whitespace(input) {
        None => ("", Node::Empty),
        Some('#') => parse_heading(input),
        Some('*') if at_line_start => parse_list(input),
        Some('*') => parse_style(input),
        _ => parse_text(input)
    }
}

fn parse<'a>(input: &'a str) -> Vec<Node> {
    let mut nodes = Vec::new();
    let mut rest = input;
    while !rest.is_empty() {
        let node;
        (rest, node) = parse_node(rest);
        nodes.push(node);
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

    #[test]
    fn test_end_list() {
        assert_eq!(
            super::parse_document("* list item\ncontinued\n\nended"),
            Node::Document(vec![
                Node::List(vec![
                    Node::Item(vec![Node::text("list item"), Node::text("continued")])
                ]),
                Node::text("ended")
            ])
        )
    }
}

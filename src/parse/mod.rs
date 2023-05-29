#![allow(dead_code)]

use std::str::pattern::Pattern;

use crate::model::{Node, Style};

#[cfg(test)]
mod test;

const CONTROL: &str = "*#[!~";

fn add_node(to: &mut Vec<Node>, node: Node) {
    if !node.is_empty() {
        to.push(node);
    }
}

fn is_empty(input: &str) -> bool {
    return input.trim().is_empty();
}

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

fn nth_solid(input: &str, n: u32) -> Option<char> {
    let mut count = 0;
    for c in input.chars() {
        if !c.is_whitespace() {
            count += 1;

            if count >= n {
                return Some(c);
            }
        }
    }
    None
}

fn first_solid(input: &str) -> Option<char> {
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

fn consume_whitespace<'a>(input: &'a str) -> &'a str {
    consume(input, |c: char| !c.is_whitespace()).0
}

fn consume_until_any<'a>(input: &'a str, chars: &str) -> (&'a str, &'a str) {
    consume(input, |c| chars.contains(c))
}

fn consume_chars<'a>(input: &'a str, chars: &str) -> (&'a str, &'a str) {
    consume(input, |c| !chars.contains(c))
}

fn parse_heading<'a>(input: &'a str) -> (&'a str, Node) {
    let (rest, _) = consume_chars(input.trim_start(), "#");
    let (rest, text) = consume(rest.trim_start(), '\n');
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
        input if input.starts_with('~') => {
            let (rest, text) = consume(drop_first(input), '~');
            (drop_first(rest), Node::Style(Style::Strikethrough, parse(text)))
        }
        _ => parse_node(input),
    }
}

fn parse_text<'a>(input: &'a str) -> (&'a str, Node) {
    let mut node = Node::Text(String::new());
    let mut rest = input.trim_start();

    loop {
        let text;
        (rest, text) = consume(rest.trim_start(), |c| CONTROL.contains(c) || c == '\n');
        node.add_text(text);

        // Control character, parse separately.
        if starts_with_any(rest.trim_start(), CONTROL) {
            if first_solid(rest) == Some('!') {
                if nth_solid(rest, 2) == Some('[') {
                    break;
                } else if let Node::Text(text) = &mut node {
                    text.push('!');
                    rest = drop_first(rest);
                }
            } else {
                break;
            }
        }

        // Empty line, new text node.
        if starts_with_empty_line(rest) {
            break;
        }

        if is_empty(rest) {
            break;
        }
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
            _ => break,
        }
    }
    None
}

fn parse_list_item<'a>(input: &'a str) -> (&'a str, Node) {
    let prefix_size = list_prefix_size(input);
    let mut nodes = Vec::new();
    let mut rest = drop_first(consume(input, '*').0);

    while !is_empty(rest) && !starts_with_empty_line(rest) {
        while !is_empty(rest) && first_char(rest) != Some('\n') {
            let node;
            (rest, node) = parse_node(rest);
            add_node(&mut nodes, node);
        }

        if first_solid(rest) == Some('*') {
            if list_prefix_size(rest) <= prefix_size {
                return (rest, Node::Item(nodes));
            }
        }

        if starts_with_empty_line(rest) {
            break;
        }

        let node;
        (rest, node) = parse_node_line_start(rest);
        add_node(&mut nodes, node);
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
        add_node(&mut nodes, node);
    }

    (rest, Node::List(nodes))
}

fn parse_link<'a>(input: &'a str) -> (&'a str, Node) {
    let mut rest = drop_first(input.trim_start());
    let text;
    (rest, text) = consume(rest, ']');

    if first_char(rest) == Some(']') {
        rest = drop_first(rest);
    }

    if first_solid(rest) == Some('(') {
        rest = drop_first(rest.trim());

        let url;
        (rest, url) = consume(rest, ')');

        if first_solid(rest) == Some(')') {
            rest = drop_first(rest.trim());
            return (rest, Node::link(text.trim(), url));
        }
    }

    let consumed = input.len() - rest.len();
    let mut node;
    (rest, node) = parse_text(rest);

    if let Node::Text(text) = &mut node {
        node = Node::Text(format!("{} {}", &input[..consumed].trim(), &text.trim()));
    }
    (rest, node)
}

fn parse_image<'a>(input: &'a str) -> (&'a str, Node) {
    let result = parse_link(drop_first(input.trim_start()));
    if let (rest, Node::Link(text, url)) = result {
        (rest, Node::Image(text, url))
    } else {
        result
    }
}

fn starts_with_new_line(input: &str) -> bool {
    for c in input.chars() {
        match c {
            '\n' => return true,
            _ if c.is_whitespace() => (),
            _ => return false,
        }
    }

    false
}

fn _parse_node<'a>(input: &'a str, at_line_start: bool) -> (&'a str, Node) {
    let mut rest = input;
    while starts_with_new_line(rest) {
        rest = drop_first(consume(rest, '\n').0);
    }

    match first_solid(rest) {
        None => ("", Node::Empty),
        Some('#') => parse_heading(rest),
        Some('*') if at_line_start => parse_list(rest),
        Some('*') | Some('~') => parse_style(rest),
        Some('[') => parse_link(rest),
        Some('!') if nth_solid(rest, 2) == Some('[') => parse_image(rest),
        _ => parse_text(rest),
    }
}

fn parse_node_line_start<'a>(input: &'a str) -> (&'a str, Node) {
    _parse_node(input, true)
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

    _parse_node(input, at_line_start)
}

fn parse<'a>(input: &'a str) -> Vec<Node> {
    let mut nodes = Vec::new();
    let (mut rest, node) = parse_node_line_start(input);
    add_node(&mut nodes, node);

    while !is_empty(rest) {
        let node;
        (rest, node) = parse_node(rest);
        add_node(&mut nodes, node);
    }

    nodes
}

pub fn parse_document(input: &str) -> Node {
    Node::Document(parse(input))
}

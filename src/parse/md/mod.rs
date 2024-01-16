#![allow(dead_code)]

use std::str::pattern::Pattern;

use crate::model::{Doc, El, Node, Style, HEADING_MAX_LEVEL};

#[cfg(test)]
mod test;

const CONTROL: &str = "*#[!~`";

fn add_node(to: &mut Vec<Node>, node: Node) {
    if !node.is_empty() {
        to.push(node);
    }
}

fn is_empty(input: &str) -> bool {
    return input.trim().is_empty();
}

fn drop_n(input: &str, n: usize) -> &str {
    if input.len() <= n {
        ""
    } else {
        &input[n..]
    }
}

fn drop_first(input: &str) -> &str {
    drop_n(input, 1)
}

fn first_char(input: &str) -> Option<char> {
    input.chars().next()
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
    input.trim_start().chars().next()
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
    false
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

fn consume_whitespace(input: &str) -> &str {
    consume(input, |c: char| !c.is_whitespace()).0
}

fn consume_until_any<'a>(input: &'a str, chars: &str) -> (&'a str, &'a str) {
    consume(input, |c| chars.contains(c))
}

fn consume_chars<'a>(input: &'a str, chars: &str) -> (&'a str, &'a str) {
    consume(input, |c| !chars.contains(c))
}

fn parse_heading(input: &str) -> (&str, Node) {
    let (rest, hashes) = consume_chars(input.trim_start(), "#");
    let level = hashes.len().min(HEADING_MAX_LEVEL.into()) as u8;
    let (rest, text) = consume(rest.trim_start(), '\n');
    (rest, Node::heading(level, parse(text, false)))
}

fn parse_style(input: &str) -> (&str, Node) {
    match consume_whitespace(input) {
        input if input.starts_with("**") => {
            let (rest, text) = consume(drop_n(input, 2), "**");
            (
                drop_n(rest, 2),
                Node::style(Style::Bold, parse(text, false)),
            )
        }
        input if input.starts_with('*') => {
            let (rest, text) = consume(drop_first(input), '*');
            (
                drop_first(rest),
                Node::style(Style::Italic, parse(text, false)),
            )
        }
        input if input.starts_with('~') => {
            let (rest, text) = consume(drop_first(input), '~');
            (
                drop_first(rest),
                Node::style(Style::Strikethrough, parse(text, false)),
            )
        }
        _ => parse_node(input),
    }
}

fn parse_text(input: &str) -> (&str, Node) {
    let mut node = Node::text("");
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
                } else if let El::Text(text) = node.el_mut() {
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

fn parse_list_item(input: &str) -> (&str, Node) {
    let prefix_size = list_prefix_size(input);
    let mut nodes = Vec::new();
    let mut rest = drop_first(consume(input, '*').0);

    while !is_empty(rest) && !starts_with_empty_line(rest) {
        while !is_empty(rest) && first_char(rest) != Some('\n') {
            let node;
            (rest, node) = parse_node(rest);
            add_node(&mut nodes, node);
        }

        if first_solid(rest) == Some('*') && list_prefix_size(rest) <= prefix_size {
            return (rest, Node::item(nodes));
        }

        if starts_with_empty_line(rest) {
            break;
        }

        let node;
        (rest, node) = parse_node_line_start(rest);
        add_node(&mut nodes, node);
    }
    (rest, Node::item(nodes))
}

fn parse_list(input: &str) -> (&str, Node) {
    let mut nodes = Vec::new();

    let prefix_size = list_prefix_size(input);

    let mut rest = input;
    while list_prefix_size(rest) == prefix_size {
        let node;
        (rest, node) = parse_list_item(rest);
        add_node(&mut nodes, node);
    }

    (rest, Node::list(nodes))
}

fn parse_link(input: &str) -> (&str, Node) {
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

    if let El::Text(text) = node.el_mut() {
        node = Node::text(&format!("{} {}", &input[..consumed].trim(), &text.trim()));
    }
    (rest, node)
}

fn parse_image(input: &str) -> (&str, Node) {
    let (rest, result) = parse_link(drop_first(input.trim_start()));
    if let (Some(text), Some(url)) = (result.el_text(), result.el_url()) {
        (rest, Node::image(text, url))
    } else {
        (rest, result)
    }
}

fn parse_code(input: &str) -> (&str, Node) {
    if input.starts_with("```") {
        let (rest, code) = consume(drop_n(input, 3), "```");
        let code = code.trim();
        if code.contains('\n') {
            let (rest_code, lang) = consume(code, |c: char| !c.is_alphanumeric());

            if !rest_code.is_empty() {
                return (rest, Node::codeblock(Some(lang), rest_code));
            }
        }

        (rest, Node::codeblock(None, code))
    } else {
        let (rest, code) = consume(drop_first(input), '`');
        (rest, Node::code(code))
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

fn parse_row(input: &str) -> (&str, Vec<Vec<Node>>) {
    let (mut rest, _) = consume(input, '|');
    rest = drop_first(rest); // drop '|'
    let mut cols = Vec::new();
    while !rest.is_empty() && !rest.starts_with('\n') {
        let text;
        (rest, text) = consume_until_any(rest, "|\n");
        cols.push(parse(text, false));
        if rest.starts_with('|') {
            rest = drop_first(rest);
        }
    }

    (rest, cols)
}

fn parse_table(input: &str) -> (&str, Node) {
    let mut rest = input;
    let mut rows = Vec::new();
    while {
        let row;
        (rest, row) = parse_row(rest);
        rows.push(row);
        rest.starts_with('\n') && first_solid(rest) == Some('|')
    } {}

    (rest, Node::table(rows))
}

fn _parse_node(input: &str, at_line_start: bool) -> (&str, Node) {
    let mut rest = input;
    while starts_with_new_line(rest) {
        rest = drop_first(consume(rest, '\n').0);
    }

    match first_solid(rest) {
        None => ("", Node::empty()),
        Some('`') => parse_code(rest),
        Some('#') => parse_heading(rest),
        Some('*') if at_line_start => parse_list(rest),
        Some('*') | Some('~') => parse_style(rest),
        Some('[') => parse_link(rest),
        Some('!') if nth_solid(rest, 2) == Some('[') => parse_image(rest),
        Some('|') if at_line_start => parse_table(rest),
        _ => parse_text(rest),
    }
}

fn parse_node_line_start(input: &str) -> (&str, Node) {
    _parse_node(input, true)
}

fn parse_node(input: &str) -> (&str, Node) {
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

fn parse(input: &str, at_line_start: bool) -> Vec<Node> {
    let mut rest = input;
    let mut nodes = Vec::new();
    if at_line_start {
        let node;
        (rest, node) = parse_node_line_start(input);
        add_node(&mut nodes, node);
    }

    while !is_empty(rest) {
        let node;
        (rest, node) = parse_node(rest);
        add_node(&mut nodes, node);
    }

    nodes
}

pub fn parse_document(input: &str) -> Doc {
    Doc::from(parse(input, true))
}
